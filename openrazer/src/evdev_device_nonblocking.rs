use std::{
    path::Path,
    sync::mpsc::{Receiver, TryRecvError},
};

// FIXME: This definitely doesn't drop correctly.
// The thread keeps running for the whole program, and when the program exits this goes as well.

/// [`evdev::Device::set_nonblocking`] doesn't work, so this is just a workaround for that.
#[derive(Debug)]
pub struct EvdevDeviceNonblocking {
    rx: Receiver<evdev::InputEvent>,
}

impl EvdevDeviceNonblocking {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn({
            let mut device = evdev::Device::open(&path)?;
            move || loop {
                for event in device.fetch_events().unwrap() {
                    tx.send(event).unwrap();
                }
            }
        });
        Ok(Self { rx })
    }

    pub fn try_next(&self) -> Result<Option<evdev::InputEvent>, TryRecvError> {
        match self.rx.try_recv() {
            Ok(event) => Ok(Some(event)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(err) => Err(err),
        }
    }
}
