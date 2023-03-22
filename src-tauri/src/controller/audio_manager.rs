use std::error::Error;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use cpal::Device;
use cpal::traits::{DeviceTrait, HostTrait};
use lazy_static::lazy_static;
use tauri::{Window, Wry};

use crate::controller::errors::CommonError;

lazy_static! {
  static ref AUDIO_MANGER: Arc<RwLock<AudioManager>> = Arc::new(RwLock::new(AudioManager::new()));
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SelectByName {
    name: String,
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct SelectDefault {}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type")]
pub enum AudioSelection {
    Default(SelectDefault),
    ByName(SelectByName),
}

impl Default for AudioSelection {
    fn default() -> Self {
        AudioSelection::Default(SelectDefault::default())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioSelectionConfig {
    output: AudioSelection,
}

#[derive(Debug, Clone)]
struct AudioManager {
    config: AudioSelectionConfig,
}

impl AudioManager {
    fn new() -> Self {
        AudioManager {
            config: AudioSelectionConfig {
                output: AudioSelection::default()
            }
        }
    }

    fn change_output_device_to_default(&mut self) {
        self.config.output = AudioSelection::Default(SelectDefault {});
    }

    fn change_output_device_by_name(&mut self, new_device: String) {
        self.config.output = AudioSelection::ByName(SelectByName {
            name: new_device
        });
    }

    fn get_output_device_name(&self) -> Option<String> {
        let AudioSelection::ByName(SelectByName { name }) = self.to_owned().config.output else {
            return None;
        };
        Some(name.clone())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioConfigResponseData {
    config: AudioSelectionConfig,
    output_devices: Vec<String>,
}

fn available_output_devices() -> Result<Vec<String>, Box<dyn Error>> {
    let host = cpal::default_host();
    let output_devices = host.output_devices()
        .map_err(Box::new)?;
    let mut result: Vec<String> = vec![];
    for (_, device) in output_devices.enumerate() {
        result.push(device.name().map_err(Box::new)?);
    }
    Ok(result)
}

fn get_audio_config_from_manager(manager: &AudioManager) -> Result<AudioConfigResponseData, Box<dyn Error>> {
    let config = manager.config.clone();
    let outputs = available_output_devices()?;
    Ok(AudioConfigResponseData {
        config,
        output_devices: outputs,
    })
}

pub fn get_audio_config() -> Result<AudioConfigResponseData, Box<dyn Error>> {
    let lock = AUDIO_MANGER.clone();
    let manager = lock.read().unwrap();
    get_audio_config_from_manager(manager.deref())
}

pub fn change_output_device(selection: AudioSelection) -> Result<AudioConfigResponseData, Box<dyn Error>> {
    let lock = AUDIO_MANGER.clone();
    let mut manager = lock.write().unwrap();
    match &selection {
        AudioSelection::Default(_) => {
            manager.change_output_device_to_default();
            return get_audio_config_from_manager(manager.deref());
        }
        AudioSelection::ByName(SelectByName { name }) => {
            let host = cpal::default_host();
            let output_devices = host.output_devices()
                .map_err(Box::new)?;

            for (_, device) in output_devices.enumerate() {
                let device_name = device.name().map_err(Box::new)?;
                if device_name == *name {
                    manager.change_output_device_by_name(device_name.clone());
                    log::info!("change device by name {}", device_name.clone());
                    return get_audio_config_from_manager(manager.deref());
                }
            }
            Err(Box::new(CommonError::new(format!("No such device of name {}", name))))
        }
    }
}

pub fn get_output_device() -> Result<Device, Box<dyn Error>> {
    let lock = AUDIO_MANGER.clone();
    let audio_manager = lock.write().unwrap();
    let output = audio_manager.get_output_device_name();
    let host = cpal::default_host();
    if let Some(name) = output {
        let output_devices = host.output_devices()
            .map_err(Box::new)?;

        for (_, device) in output_devices.enumerate() {
            let device_name = device.name().map_err(Box::new)?;
            if device_name == name {
                return Ok(device);
            }
        }
    }
    host.default_output_device()
        .ok_or(Box::new(CommonError::new(String::from("No default output device"))))
}

fn check_audio_output_devices(window: Window<Wry>) -> Result<(), Box<dyn Error>> {
    let lock = AUDIO_MANGER.clone();
    let mut audio_manager = lock.write().unwrap();
    let host = cpal::default_host();
    if let Some(name) = audio_manager.get_output_device_name() {
        let output_devices = host.output_devices()
            .map_err(Box::new)?;

        let mut exists = false;
        for (_, device) in output_devices.enumerate() {
            let device_name = device.name().map_err(Box::new)?;
            if device_name == name {
                exists = true;
                break;
            }
        }
        if !exists {
            log::warn!("Current selected output device {} no longer exists, set to use default device", name.clone());
            // if selected device no longer exists, change to use default device
            audio_manager.change_output_device_to_default();
            // emit change to frontend to let ui change as well
            window.emit("on_audio_config_change", {}).map_err(Box::new)?;
            // TODO add watching for complete device list changes as well
        }
    }
    Ok(())
}

pub fn watch_audio_devices(window: Window<Wry>) {
    log::info!("Start watching audio devices");
    std::thread::spawn(move || {
        loop {
            match check_audio_output_devices(window.to_owned()) {
                Ok(_) => {}
                Err(err) => {
                    log::error!("Unable to check audio output devices, err: {}", err);
                }
            }
            std::thread::sleep(Duration::from_secs(1));
        }
    });
}

#[cfg(test)]
mod tests {
    use cpal::traits::{DeviceTrait, HostTrait};

    #[test]
    fn test_get_devices() {
        let host = cpal::default_host();
        let output_devices = host.output_devices().expect("Failed to get output devices");

        println!("Available output devices:");

        for (i, device) in output_devices.enumerate() {
            println!("{}. {}", i + 1, device.name().unwrap());
        }

        println!("Supported hosts:\n  {:?}", cpal::ALL_HOSTS);
        let available_hosts = cpal::available_hosts();
        println!("Available hosts:\n  {:?}", available_hosts);

        for host_id in available_hosts {
            println!("{}", host_id.name());
            let host = cpal::host_from_id(host_id).unwrap();

            let default_in = host.default_input_device().map(|e| e.name().unwrap());
            let default_out = host.default_output_device().map(|e| e.name().unwrap());
            println!("  Default Input Device:\n    {:?}", default_in);
            println!("  Default Output Device:\n    {:?}", default_out);

            let devices = host.input_devices().unwrap();
            println!("Input  Devices: ");
            for (device_index, device) in devices.enumerate() {
                println!("  {}. \"{}\"", device_index + 1, device.name().unwrap());
            }
            let devices = host.output_devices().unwrap();
            println!("Output Devices: ");
            for (device_index, device) in devices.enumerate() {
                println!("  {}. \"{}\"", device_index + 1, device.name().unwrap());
            }
        }
    }
}
