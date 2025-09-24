use std::{collections::HashMap, path::PathBuf};

use regex::Regex;

use crate::{
    query_devices, DeviceMatrixEffectManager, EvdevDeviceNonblocking, OpenRazerError, QueryDevice,
    RAZER_DEVICE_VENDOR_ID,
};

#[derive(Debug)]
pub struct RazerDevice {
    query_devices: Box<[QueryDevice]>,
}

impl RazerDevice {
    pub fn get_matrix_manager(&self) -> Result<Option<DeviceMatrixEffectManager>, OpenRazerError> {
        // Just search all devices if they have the openrazer stuff :)
        for device in self.query_devices.iter() {
            let mut path = PathBuf::from(format!("/sys/{}", device.sys_path));
            path.pop();
            path.pop();
            if std::fs::read_dir(&path)?
                .flat_map(|entry| entry.ok())
                .any(|entry| entry.path().ends_with("matrix_effect_none"))
            {
                return Ok(Some(DeviceMatrixEffectManager::new(path)));
            }
        }
        Ok(None)
    }

    pub fn get_evdev_device(&self) -> Result<Option<EvdevDeviceNonblocking>, OpenRazerError> {
        // FIXME: This is def not a good way to get what one is actually keyboard inputs.
        for device in self.query_devices.iter() {
            if device.handlers.contains(&"kbd".to_owned())
                && !device.handlers.contains(&"leds".to_owned())
            {
                if let Some(event) = device
                    .handlers
                    .iter()
                    .find(|handler| handler.starts_with("event"))
                {
                    let mut path = PathBuf::from("/dev/input/");
                    path.push(event);
                    let file = std::fs::File::open(&path)?;
                    log::info!("Reading keyboard events from {path:?}");
                    return Ok(Some(EvdevDeviceNonblocking::from_fd(file)?));
                }
            }
        }
        Ok(None)
    }
}

pub fn query_razer_devices() -> Result<Box<[RazerDevice]>, OpenRazerError> {
    let mut groups: HashMap<String, Vec<QueryDevice>> = HashMap::new();

    query_devices()?
        .into_iter()
        .filter(|device| device.id_vendor == RAZER_DEVICE_VENDOR_ID)
        .for_each(|device| {
            // NOTE:
            // I have absolutely no idea what I'm doing. this just seems the only actual way to
            // reliably get if they are actually the same(IRL) device.
            //
            // The path looks something like: /devices/pci0000:00/0000:00:02.1/0000:16:00.0/usb1/1-2/1-2:1.2/0003:1532:021E.0008/input/input32
            // And the third to last path segment seems to be consistent between devices if they
            // are the same(IRL) device
            let Some((_, [ident])) = Regex::new(r".*/(.+?)\.[\da-fA-F]+/.+?/.+?$")
                .unwrap()
                .captures(&device.sys_path)
                .map(|i| i.extract())
            else {
                return;
            };
            groups.entry(ident.to_owned()).or_default().push(device);
        });

    Ok(groups
        .into_values()
        .map(|devices| RazerDevice {
            query_devices: devices.into_boxed_slice(),
        })
        .collect())
}
