use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use cpal::Device;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use lazy_static::lazy_static;
use rodio::{OutputStream, Sink};
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::RwLock as AsyncRwLock;

use crate::common::{app, constants};
use crate::controller::errors::{CommonError, ProgramError};

lazy_static! {
    static ref AUDIO_MANGER: Arc<AsyncRwLock<AudioManager>> = Arc::new(AsyncRwLock::new(AudioManager::new()));
    static ref MIC_STREAM: Arc<Mutex<MicStreamWrapper>> = Arc::new(Mutex::new(MicStreamWrapper::new()));
    static ref MIC_STREAM_STOP: (Sender<()>, Receiver<()>) = broadcast::channel(1);
    static ref MIC_STREAM_STOP_ACCEPT: (Sender<()>, Receiver<()>) = broadcast::channel(1);
    static ref MIC_STREAMING: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

pub struct MicStreamWrapper {
    output_stream: Option<OutputStream>,
    sink: Option<Sink>,
    stream: Option<cpal::Stream>,
}

unsafe impl Send for MicStreamWrapper {}

unsafe impl Sync for MicStreamWrapper {}

impl MicStreamWrapper {
    fn new() -> Self {
        MicStreamWrapper {
            output_stream: None,
            sink: None,
            stream: None,
        }
    }
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StreamConfig {
    stream_input: bool,
    stream_mic_input: bool,
}

#[derive(Debug, Clone)]
struct AudioManager {
    config: AudioSelectionConfig,
    stream: StreamConfig,
}

impl AudioManager {
    fn new() -> Self {
        AudioManager {
            config: AudioSelectionConfig {
                output: AudioSelection::default(),
                input: AudioSelection::default(),
            },
            stream: StreamConfig {
                stream_input: false,
                stream_mic_input: false,
            },
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
    stream: StreamConfig,
    default_output_device: String,
    output_devices: Vec<String>,
    default_input_device: String,
    input_devices: Vec<String>,
}

fn default_output_device() -> Result<String, ProgramError> {
    let host = cpal::default_host();
    let output_device = host.default_output_device()
        .ok_or("No default output device found")?;
    Ok(output_device.name()?)
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

fn default_input_device() -> Result<String, ProgramError> {
    let host = cpal::default_host();
    let input_device = host.default_input_device()
        .ok_or("No default input device found")?;
    Ok(input_device.name()?)
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
        stream: manager.stream.clone(),
        default_output_device: default_output_device()?,
        output_devices: outputs,
        default_input_device: default_input_device()?,
        input_devices: inputs,
    })
}

pub async fn get_audio_config() -> Result<AudioConfigResponseData, ProgramError> {
    let lock = AUDIO_MANGER.clone();
    let manager = lock.read().await;
    get_audio_config_from_manager(manager.deref())
}

pub async fn start_mic_streaming() -> Result<(), ProgramError> {
    if MIC_STREAMING.load(Ordering::Acquire) {
        return Err(ProgramError::from("Mic streaming is running"));
    }

    // TODO sink Send?
    {
        let input_device = get_input_device().await?;
        let output_device = get_vb_audio_cable_output()?;
        let (output_stream, stream_handle) = OutputStream::try_from_device(&output_device)
            .map_err(|e| {
                log::error!("Stream to VB audio output, err: {}", e);
                ProgramError::from("unable to stream to VB audio output")
            })?;
        let sink = Sink::try_new(&stream_handle).map_err(|e| {
            log::error!("Failed to sink to VB audio output, err: {}", e);
            ProgramError::from("unable to sink to VB audio output")
        })?;

        let input_config = input_device.default_input_config()?;
        let sample_rate = input_config.sample_rate().0;
        let channel = input_config.channels();

        let input_stream = input_device.build_input_stream(
            &input_config.into(),
            move |data: &[f32], _: &_| {
                let mic_stream_clone = MIC_STREAM.clone();
                let mic_stream = mic_stream_clone.lock().unwrap();
                let samples = rodio::buffer::SamplesBuffer::new(channel, sample_rate, data);
                mic_stream.sink.as_ref().unwrap().append(samples);
            },
            |err| eprintln!("an error occurred on the input stream: {}", err),
            None)?;

        log::debug!("Starting mic streaming");
        input_stream.play()?;

        let mic_stream_clone = MIC_STREAM.clone();
        let mut mic_stream = mic_stream_clone.lock().unwrap();
        mic_stream.stream.replace(input_stream);
        mic_stream.sink.replace(sink);
        mic_stream.output_stream.replace(output_stream);

        log::debug!("Start mic streaming success");

        MIC_STREAMING.store(true, Ordering::Release);
    };

    log::debug!("Wait for mic streaming to be stopped");
    let (interrupted_tx, _) = &*MIC_STREAM_STOP;
    let mut interrupted_rx = interrupted_tx.subscribe();

    // wait for stop signal
    let _ = interrupted_rx.recv().await;

    log::debug!("Mic streaming stop signal accepted");

    MIC_STREAMING.store(false, Ordering::Release);


    let mic_stream_clone = MIC_STREAM.clone();
    let mut mic_stream = mic_stream_clone.lock().unwrap();
    mic_stream.stream.take();
    mic_stream.output_stream.take();

    // stop sink
    let sink = mic_stream.sink.as_ref().unwrap();
    sink.stop();

    mic_stream.sink.take();

    log::debug!("Mic streaming stopped");

    let (accept_tx, _) = &*MIC_STREAM_STOP_ACCEPT;
    accept_tx.send(()).map_err(|e| {
        log::error!("Failed to send mic stream stop accept signal, err: {}", e);
        ProgramError::from("unable to send mic stream stop accept signal")
    })?;

    Ok(())
}

pub async fn stop_mic_streaming() -> Result<(), ProgramError> {
    if !MIC_STREAMING.load(Ordering::Acquire) {
        log::debug!("Mic streaming doesn't started, no need to stop mic streaming");
        return Ok(());
    }
    log::debug!("Stopping mic streaming");
    let (interrupted_tx, _) = &*MIC_STREAM_STOP;

    interrupted_tx.send(()).map_err(|e| {
        log::error!("Failed to send mic streaming stop signal, error: {}", e);
        ProgramError::from("unable to send mic streaming stop signal")
    })?;

    let (accept_tx, _) = &*MIC_STREAM_STOP_ACCEPT;
    let mut accept_rx = accept_tx.subscribe();

    // wait for accept signal
    _ = accept_rx.recv().await;

    Ok(())
}

pub async fn restart_mic_streaming() -> Result<(), ProgramError> {
    log::debug!("Try to restart mic streaming");
    stop_mic_streaming().await?;

    let lock = AUDIO_MANGER.clone();
    let manager = lock.read().await;
    let is_streaming = manager.stream.stream_input && manager.stream.stream_mic_input;
    log::debug!("Restart is_streaming: {}", is_streaming);
    if is_streaming {
        start_mic_streaming_async();
    }
    Ok(())
}

fn start_mic_streaming_async() {
    // start mic streaming
    tauri::async_runtime::spawn(async move {
        match start_mic_streaming().await {
            Ok(_) => {}
            Err(err) => {
                log::error!("Start mic streaming failed with error: {}", err);
            }
        }
    });
}

pub async fn change_stream_config(stream: StreamConfig) -> Result<AudioConfigResponseData, ProgramError> {
    let is_streaming = {
        let lock = AUDIO_MANGER.clone();
        let mut manager = lock.write().await;
        manager.stream = stream;
        manager.stream.stream_input && manager.stream.stream_mic_input
    };

    log::debug!("is streaming: {}", is_streaming);
    if is_streaming {
        start_mic_streaming_async();
    } else {
        // stop mic streaming
        stop_mic_streaming().await?;
    }

    get_audio_config().await
}

pub async fn change_output_device(selection: AudioSelection) -> Result<AudioConfigResponseData, ProgramError> {
    let lock = AUDIO_MANGER.clone();
    let mut manager = lock.write().await;
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

pub async fn change_input_device(selection: AudioSelection) -> Result<AudioConfigResponseData, ProgramError> {
    let lock = AUDIO_MANGER.clone();
    let mut manager = lock.write().await;
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

pub async fn get_output_device() -> Result<Device, ProgramError> {
    let lock = AUDIO_MANGER.clone();
    let audio_manager = lock.read().await;
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

pub async fn is_stream_input_enabled() -> bool {
    let lock = AUDIO_MANGER.clone();
    let audio_manager = lock.read().await;
    audio_manager.stream.stream_input
}

pub fn get_vb_audio_cable_output() -> Result<Device, ProgramError> {
    let host = cpal::default_host();
    let devices = host.output_devices().unwrap();

    for (_, device) in devices.enumerate() {
        let name = device.name().unwrap();
        if name == "CABLE Input (VB-Audio Virtual Cable)" {
            return Ok(device);
        }
    }

    Err(ProgramError::from("VB audio cable output not found"))
}

pub async fn get_input_device() -> Result<Device, ProgramError> {
    let lock = AUDIO_MANGER.clone();
    let audio_manager = lock.read().await;
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

async fn check_audio_output_devices(previous: &Vec<String>,
                                    previous_default_output: String)
                                    -> Result<(bool, Option<(Vec<String>, String)>), ProgramError> {
    let lock = AUDIO_MANGER.clone();
    let mut audio_manager = lock.write().await;
    let outputs = available_output_devices();
    let default_output = default_output_device();
    if outputs.is_err() {
        log::error!("Unable to get current outputs, err: {}", outputs.unwrap_err());
        return Ok((false, None));
    }
    if default_output.is_err() {
        log::error!("Unable to get current default output, err: {}", default_output.unwrap_err());
        return Ok((false, None));
    }
    let outputs = outputs.unwrap();
    let default_output = default_output.unwrap();
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
            return Ok((true, Some((outputs, default_output))));
        }
    }

    if !is_vec_equals(previous, &outputs) {
        return Ok((true, Some((outputs, default_output))));
    }

    Ok((previous_default_output != default_output, Some((outputs, default_output))))
}

async fn check_audio_input_devices(previous: &Vec<String>,
                                   previous_default_input: String) -> Result<(bool, Option<(Vec<String>, String)>), ProgramError> {
    let lock = AUDIO_MANGER.clone();
    let mut audio_manager = lock.write().await;
    let inputs = available_input_devices();
    let default_input = default_input_device();
    if inputs.is_err() {
        log::error!("Unable to get current inputs, err: {}", inputs.unwrap_err());
        return Ok((false, None));
    }
    if default_input.is_err() {
        log::error!("Unable to get current default input, err: {}", default_input.unwrap_err());
        return Ok((false, None));
    }
    let inputs = inputs.unwrap();
    let default_input = default_input.unwrap();
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
            return Ok((true, Some((inputs, default_input))));
        }
    }

    if !is_vec_equals(previous, &inputs) {
        return Ok((true, Some((inputs, default_input))));
    }

    Ok((previous_default_input != default_input, Some((inputs, default_input))))
}

fn get_current_inputs_outputs() -> Option<(Vec<String>, String, Vec<String>, String)> {
    let outputs = available_output_devices();
    let default_output = default_output_device();
    if outputs.is_err() || default_output.is_err() {
        return None;
    }
    let inputs = available_input_devices();
    let default_input = default_input_device();
    if inputs.is_err() || default_input.is_err() {
        return None;
    }
    return Some((inputs.unwrap(), default_input.unwrap(),
                 outputs.unwrap(), default_output.unwrap()));
}

pub async fn watch_audio_devices() {
    log::info!("Start watching audio devices");
    loop {
        let mut inputs: Option<Vec<String>> = None;
        let mut default_input: Option<String> = None;
        let mut outputs: Option<Vec<String>> = None;
        let mut default_output: Option<String> = None;
        if let Some((_inputs, _default_input, _outputs, _default_output)) = get_current_inputs_outputs() {
            inputs.replace(_inputs);
            default_input.replace(_default_input);
            outputs.replace(_outputs);
            default_output.replace(_default_output);
        } else {
            std::thread::sleep(Duration::from_secs(1));
            continue;
        }
        std::thread::sleep(Duration::from_secs(1));
        match check_audio_output_devices(
            outputs.as_ref().unwrap(),
            default_output.as_ref().unwrap().clone()).await {
            Ok((changed, mut new_outputs)) => {
                if changed {
                    log::debug!("Found output devices changed");
                    app::silent_emit_all(constants::event::ON_AUDIO_CONFIG_CHANGE,
                                         {});
                }
                if let Some((new_outputs, new_default)) = new_outputs.take() {
                    outputs.replace(new_outputs);
                    default_output.replace(new_default);
                }
            }
            Err(err) => {
                log::error!("Unable to check audio output devices, err: {}", err);
            }
        }
        match check_audio_input_devices(
            inputs.as_ref().unwrap(),
            default_input.as_ref().unwrap().clone()).await {
            Ok((changed, mut new_inputs)) => {
                if changed {
                    log::debug!("Found input devices changed");
                    app::silent_emit_all(constants::event::ON_AUDIO_CONFIG_CHANGE,
                                         {});
                }
                if let Some((new_inputs, new_default)) = new_inputs.take() {
                    inputs.replace(new_inputs);
                    default_input.replace(new_default);
                }
            }
            Err(err) => {
                log::error!("Unable to check audio input devices, err: {}", err);
            }
        }
    }
}
