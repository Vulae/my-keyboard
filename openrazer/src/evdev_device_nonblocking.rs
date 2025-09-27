use std::{
    os::fd::{AsRawFd, FromRawFd, RawFd},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, TryRecvError},
        Arc,
    },
    thread::JoinHandle,
};

/// [`evdev::Device::set_nonblocking`] doesn't work, so this is just a workaround for that.
#[derive(Debug)]
pub struct EvdevDeviceNonblocking {
    raw_fd: RawFd,
    rx: Receiver<evdev::InputEvent>,
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl EvdevDeviceNonblocking {
    pub fn new(mut device: evdev::Device) -> Self {
        let raw_fd = device.as_raw_fd();
        let (tx, rx) = std::sync::mpsc::channel();
        let running = Arc::new(AtomicBool::new(true));
        let handle = std::thread::spawn({
            let running = Arc::clone(&running);
            move || {
                while running.load(Ordering::Relaxed) {
                    for event in device.fetch_events().unwrap() {
                        tx.send(event).unwrap()
                    }
                }
                // The inner file descriptor is dropped already
                std::mem::forget(device);
            }
        });
        Self {
            raw_fd,
            rx,
            running,
            handle: Some(handle),
        }
    }

    pub fn from_fd<F: Into<std::os::fd::OwnedFd>>(fd: F) -> Result<Self, std::io::Error> {
        Ok(Self::new(evdev::Device::from_fd(fd.into())?))
    }

    pub fn try_next(&self) -> Result<Option<evdev::InputEvent>, TryRecvError> {
        match self.rx.try_recv() {
            Ok(event) => Ok(Some(event)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

impl Drop for EvdevDeviceNonblocking {
    fn drop(&mut self) {
        log::debug!("Dropping EvdevDeviceNonblocking");
        let file = unsafe { std::fs::File::from_raw_fd(self.raw_fd) };
        std::mem::drop(file);
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let start = std::time::Instant::now();
            while !handle.is_finished() {
                std::thread::sleep(std::time::Duration::from_nanos(10));
            }
            let time = std::time::Instant::now().duration_since(start);
            if time >= std::time::Duration::from_millis(50) {
                log::warn!(
                    "Dropping EvdevDeviceNonblocking took {}ms",
                    time.as_millis(),
                )
            }
        }
        log::debug!("Successfully dropped EvdevDeviceNonblocking");
    }
}
