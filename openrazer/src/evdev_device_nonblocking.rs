// FIXME: When dropped it's blocking until a new event is sent.

use std::{
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
    rx: Receiver<evdev::InputEvent>,
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl EvdevDeviceNonblocking {
    pub fn new(mut device: evdev::Device) -> Self {
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
            }
        });
        Self {
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
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
        log::debug!("Successfully dropped EvdevDeviceNonblocking");
    }
}
