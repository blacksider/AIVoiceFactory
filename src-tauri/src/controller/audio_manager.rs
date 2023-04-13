use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use cpal::Device;
use cpal::traits::{DeviceTrait, HostTrait};
use lazy_static::lazy_static;

use crate::common::{app, constants};
use crate::controller::errors::{CommonError, ProgramError};

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
    input: AudioSelection,
}

#[derive(Debug, Clone)]
struct AudioManager {
    config: AudioSelectionConfig,
}

impl AudioManager {
    fn new() -> Self {
        AudioManager {
            config: AudioSelectionConfig {
                output: AudioSelection::default(),
                input: AudioSelection::default(),
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

    fn change_input_device_to_default(&mut self) {
        self.config.input = AudioSelection::Default(SelectDefault {});
    }

    fn change_input_device_by_name(&mut self, new_device: String) {
        self.config.input = AudioSelection::ByName(SelectByName {
            name: new_device
        });
    }

    fn get_input_device_name(&self) -> Option<String> {
        let AudioSelection::ByName(SelectByName { name }) = self.to_owned().config.input else {
            return None;
        };
        Some(name.clone())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioConfigResponseData {
    config: AudioSelectionConfig,
    output_devices: Vec<String>,
    input_devices: Vec<String>,
}

fn available_output_devices() -> Result<Vec<String>, ProgramError> {
    let host = cpal::default_host();
    let output_devices = host.output_devices()?;
    let mut result: Vec<String> = vec![];
    for (_, device) in output_devices.enumerate() {
        result.push(device.name()?);
    }
    Ok(result)
}

fn available_input_devices() -> Result<Vec<String>, ProgramError> {
    let host = cpal::default_host();
    let input_devices = host.input_devices()?;
    let mut result: Vec<String> = vec![];
    for (_, device) in input_devices.enumerate() {
        result.push(device.name()?);
    }
    Ok(result)
}

fn get_audio_config_from_manager(manager: &AudioManager) -> Result<AudioConfigResponseData, ProgramError> {
    let config = manager.config.clone();
    let outputs = available_output_devices()?;
    let inputs = available_input_devices()?;
    Ok(AudioConfigResponseData {
        config,
        output_devices: outputs,
        input_devices: inputs,
    })
}

pub fn get_audio_config() -> Result<AudioConfigResponseData, ProgramError> {
    let lock = AUDIO_MANGER.clone();
    let manager = lock.read().unwrap();
    get_audio_config_from_manager(manager.deref())
}

pub fn change_output_device(selection: AudioSelection) -> Result<AudioConfigResponseData, ProgramError> {
    let lock = AUDIO_MANGER.clone();
    let mut manager = lock.write().unwrap();
    match &selection {
        AudioSelection::Default(_) => {
            manager.change_output_device_to_default();
            return get_audio_config_from_manager(manager.deref());
        }
        AudioSelection::ByName(SelectByName { name }) => {
            let host = cpal::default_host();
            let output_devices = host.output_devices()?;

            for (_, device) in output_devices.enumerate() {
                let device_name = device.name()?;
                if device_name == *name {
                    manager.change_output_device_by_name(device_name.clone());
                    return get_audio_config_from_manager(manager.deref());
                }
            }
            Err(ProgramError::from(CommonError::new(format!("No such output device of name {}", name))))
        }
    }
}

pub fn change_input_device(selection: AudioSelection) -> Result<AudioConfigResponseData, ProgramError> {
    let lock = AUDIO_MANGER.clone();
    let mut manager = lock.write().unwrap();
    match &selection {
        AudioSelection::Default(_) => {
            manager.change_input_device_to_default();
            return get_audio_config_from_manager(manager.deref());
        }
        AudioSelection::ByName(SelectByName { name }) => {
            let host = cpal::default_host();
            let input_devices = host.input_devices()?;

            for (_, device) in input_devices.enumerate() {
                let device_name = device.name()?;
                if device_name == *name {
                    manager.change_input_device_by_name(device_name.clone());
                    return get_audio_config_from_manager(manager.deref());
                }
            }
            Err(ProgramError::from(CommonError::new(format!("No such input device of name {}", name))))
        }
    }
}

pub fn get_output_device() -> Result<Device, ProgramError> {
    let lock = AUDIO_MANGER.clone();
    let audio_manager = lock.write().unwrap();
    let output = audio_manager.get_output_device_name();
    let host = cpal::default_host();
    if let Some(name) = output {
        let output_devices = host.output_devices()?;

        for (_, device) in output_devices.enumerate() {
            let device_name = device.name()?;
            if device_name == name {
                return Ok(device);
            }
        }
    }
    host.default_output_device()
        .ok_or(ProgramError::from(
            CommonError::new(String::from("No default output device"))))
}

pub fn get_input_device() -> Result<Device, ProgramError> {
    let lock = AUDIO_MANGER.clone();
    let audio_manager = lock.write().unwrap();
    let input = audio_manager.get_input_device_name();
    let host = cpal::default_host();
    if let Some(name) = input {
        let input_devices = host.input_devices()?;

        for (_, device) in input_devices.enumerate() {
            let device_name = device.name()?;
            if device_name == name {
                return Ok(device);
            }
        }
    }
    host.default_input_device()
        .ok_or(ProgramError::from(
            CommonError::new(String::from("No default input device"))))
}

fn is_vec_equals(vec1: &Vec<String>, vec2: &Vec<String>) -> bool {
    if vec1.len() != vec2.len() {
        return false;
    }
    for i in 0..vec1.len() {
        if vec1[i] != vec2[i] {
            return false;
        }
    }
    true
}

fn check_audio_output_devices(previous: &Vec<String>) -> Result<(bool, Option<Vec<String>>), ProgramError> {
    let lock = AUDIO_MANGER.clone();
    let mut audio_manager = lock.write().unwrap();
    let outputs = available_output_devices();
    if outputs.is_err() {
        log::error!("Unable to get current outputs, err: {}", outputs.unwrap_err());
        return Ok((false, None));
    }
    let outputs = outputs.unwrap();
    if let Some(name) = audio_manager.get_output_device_name() {
        let mut exists = false;
        for device_name in &outputs {
            if *device_name == name {
                exists = true;
                break;
            }
        }
        if !exists {
            log::warn!("Current selected output device {} no longer exists, set to use default device", name.clone());
            // if selected device no longer exists, change to use default device
            audio_manager.change_output_device_to_default();
            return Ok((true, Some(outputs)));
        }
    }
    Ok((!is_vec_equals(previous, &outputs), Some(outputs)))
}

fn check_audio_input_devices(previous: &Vec<String>) -> Result<(bool, Option<Vec<String>>), ProgramError> {
    let lock = AUDIO_MANGER.clone();
    let mut audio_manager = lock.write().unwrap();
    let inputs = available_input_devices();
    if inputs.is_err() {
        log::error!("Unable to get current inputs, err: {}", inputs.unwrap_err());
        return Ok((false, None));
    }
    let inputs = inputs.unwrap();
    if let Some(name) = audio_manager.get_input_device_name() {
        let mut exists = false;
        for device_name in &inputs {
            if *device_name == name {
                exists = true;
                break;
            }
        }
        if !exists {
            log::warn!("Current selected input device {} no longer exists, set to use default device", name.clone());
            // if selected device no longer exists, change to use default device
            audio_manager.change_input_device_to_default();
            return Ok((true, Some(inputs)));
        }
    }

    // watching for complete device list changes as well
    Ok((!is_vec_equals(previous, &inputs), Some(inputs)))
}

fn get_current_inputs_outputs() -> Option<(Vec<String>, Vec<String>)> {
    let outputs = available_output_devices();
    if outputs.is_err() {
        return None;
    }
    let inputs = available_input_devices();
    if inputs.is_err() {
        return None;
    }
    return Some((inputs.unwrap(), outputs.unwrap()));
}

pub fn watch_audio_devices() {
    log::info!("Start watching audio devices");
    std::thread::spawn(move || {
        loop {
            let mut inputs;
            let mut outputs;
            if let Some((_inputs, _outputs)) = get_current_inputs_outputs() {
                inputs = _inputs;
                outputs = _outputs;
            } else {
                std::thread::sleep(Duration::from_secs(1));
                continue;
            }
            std::thread::sleep(Duration::from_secs(1));
            match check_audio_output_devices(&outputs) {
                Ok((changed, mut new_outputs)) => {
                    if changed {
                        log::debug!("Found output devices changed");
                        app::silent_emit_all(constants::event::ON_AUDIO_CONFIG_CHANGE,
                                             {});
                    }
                    if let Some(new_outputs) = new_outputs.take() {
                        outputs = new_outputs;
                    }
                }
                Err(err) => {
                    log::error!("Unable to check audio output devices, err: {}", err);
                }
            }
            match check_audio_input_devices(&inputs) {
                Ok((changed, mut new_inputs)) => {
                    if changed {
                        log::debug!("Found input devices changed");
                        app::silent_emit_all(constants::event::ON_AUDIO_CONFIG_CHANGE,
                                             {});
                    }
                    if let Some(new_inputs) = new_inputs.take() {
                        inputs = new_inputs;
                    }
                }
                Err(err) => {
                    log::error!("Unable to check audio input devices, err: {}", err);
                }
            }
        }
    });
}
