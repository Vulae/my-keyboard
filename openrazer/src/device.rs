use std::path::PathBuf;

use crate::{DeviceMatrixEffectManager, OpenRazerError};

#[derive(Debug)]
pub struct Device {
    #[allow(unused)]
    path: PathBuf,
    device_type: String,
    matrix_manager: DeviceMatrixEffectManager,
}

impl Device {
    fn new(path: PathBuf, device_type: String) -> Self {
        Self {
            path: path.clone(),
            device_type,
            matrix_manager: DeviceMatrixEffectManager::new(path),
        }
    }

    pub fn device_type(&self) -> &str {
        &self.device_type
    }

    pub fn matrix_manager(&mut self) -> &mut DeviceMatrixEffectManager {
        &mut self.matrix_manager
    }
}

pub fn query_razer_devices() -> Result<Box<[Device]>, OpenRazerError> {
    let mut devices = Vec::new();

    std::fs::read_dir("/sys/bus/hid/drivers/razerkbd/")?.try_for_each(|maybe_device| {
        let maybe_device = maybe_device?;

        let mut device_type_path = maybe_device.path();
        device_type_path.push("device_type");
        match std::fs::exists(&device_type_path) {
            Err(err) if err.kind() == std::io::ErrorKind::NotADirectory => return Ok(()),
            Ok(false) => return Ok(()),
            _ => {}
        }
        let device_type = std::fs::read_to_string(&device_type_path)?;

        devices.push(Device::new(
            maybe_device.path(),
            device_type.trim().to_owned(),
        ));

        Ok::<(), OpenRazerError>(())
    })?;

    Ok(devices.into_boxed_slice())
}
