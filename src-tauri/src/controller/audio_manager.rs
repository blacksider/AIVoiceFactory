use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Device;
use lazy_static::lazy_static;
use rodio::{OutputStream, Sink};
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};

use crate::common::{app, constants};
use crate::config::audio;
use crate::config::audio::{
    AudioSelection, AudioSelectionConfig, AudioStreamConfig, SelectByName, SelectDefault,
};
use crate::utils;

lazy_static! {
    static ref MIC_STREAM: Arc<Mutex<MicStreamWrapper>> =
        Arc::new(Mutex::new(MicStreamWrapper::new()));
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
pub struct AudioConfigResponseData {
    config: AudioSelectionConfig,
    stream: AudioStreamConfig,
    default_output_device: String,
    output_devices: Vec<String>,
    default_input_device: String,
    input_devices: Vec<String>,
}

fn default_output_device() -> Result<String> {
    let host = cpal::default_host();
    let output_device = host
        .default_output_device()
        .ok_or(anyhow!("No default output device found"))?;
    Ok(output_device.name()?)
}

fn available_output_devices() -> Result<Vec<String>> {
    let host = cpal::default_host();
    let output_devices = host.output_devices()?;
    let mut result: Vec<String> = vec![];
    for (_, device) in output_devices.enumerate() {
        result.push(device.name()?);
    }
    Ok(result)
}

fn default_input_device() -> Result<String> {
    let host = cpal::default_host();
    let input_device = host
        .default_input_device()
        .ok_or(anyhow!("No default input device found"))?;
    Ok(input_device.name()?)
}

fn available_input_devices() -> Result<Vec<String>> {
    let host = cpal::default_host();
    let input_devices = host.input_devices()?;
    let mut result: Vec<String> = vec![];
    for (_, device) in input_devices.enumerate() {
        result.push(device.name()?);
    }
    Ok(result)
}

async fn get_audio_sel_config() -> AudioSelectionConfig {
    let manager = audio::AUDIO_SEL_CONFIG_MANAGER.read().await;
    manager.get_config()
}

async fn get_audio_stream_config() -> AudioStreamConfig {
    let manager = audio::AUDIO_STREAM_CONFIG_MANAGER.read().await;
    manager.get_config()
}

pub async fn get_audio_config() -> Result<AudioConfigResponseData> {
    let outputs = available_output_devices()?;
    let inputs = available_input_devices()?;
    Ok(AudioConfigResponseData {
        config: get_audio_sel_config().await,
        stream: get_audio_stream_config().await,
        default_output_device: default_output_device()?,
        output_devices: outputs,
        default_input_device: default_input_device()?,
        input_devices: inputs,
    })
}

async fn save_audio_sel_config(config: AudioSelectionConfig) -> Result<()> {
    let mut manager = audio::AUDIO_SEL_CONFIG_MANAGER.write().await;
    if manager.save_config(config) {
        Ok(())
    } else {
        Err(anyhow!("Failed to change input device to default",))
    }
}

async fn change_input_device_to_default() -> Result<()> {
    let mut config = get_audio_sel_config().await;
    config.input = AudioSelection::Default(SelectDefault {});
    save_audio_sel_config(config).await
}

async fn change_input_device_by_name(new_device: String) -> Result<()> {
    let mut config = get_audio_sel_config().await;
    config.input = AudioSelection::ByName(SelectByName { name: new_device });
    save_audio_sel_config(config).await
}

async fn change_output_device_to_default() -> Result<()> {
    let mut config = get_audio_sel_config().await;
    config.output = AudioSelection::Default(SelectDefault {});
    save_audio_sel_config(config).await
}

async fn change_output_device_by_name(new_device: String) -> Result<()> {
    let mut config = get_audio_sel_config().await;
    config.output = AudioSelection::ByName(SelectByName { name: new_device });
    save_audio_sel_config(config).await
}

async fn get_input_device_name() -> Option<String> {
    let config = get_audio_sel_config().await;
    let AudioSelection::ByName(SelectByName { name }) = config.input else {
        return None;
    };
    Some(name)
}

async fn get_output_device_name() -> Option<String> {
    let config = get_audio_sel_config().await;
    let AudioSelection::ByName(SelectByName { name }) = config.output else {
        return None;
    };
    Some(name)
}

pub async fn start_mic_streaming() -> Result<()> {
    if MIC_STREAMING.load(Ordering::Acquire) {
        return Err(anyhow!("Mic streaming is running"));
    }

    // TODO sink Send?
    {
        let input_device = get_input_device().await?;
        let output_device = get_vb_audio_cable_output()?;
        let (output_stream, stream_handle) = OutputStream::try_from_device(&output_device)
            .map_err(|e| {
                log::error!("Stream to VB audio output, err: {}", e);
                anyhow!("unable to stream to VB audio output")
            })?;
        let sink = Sink::try_new(&stream_handle).map_err(|e| {
            log::error!("Failed to sink to VB audio output, err: {}", e);
            anyhow!("unable to sink to VB audio output")
        })?;

        let input_config = input_device.default_input_config()?;
        let sample_rate = input_config.sample_rate().0;
        let channel = input_config.channels();

        let input_stream = utils::audio::build_input_stream(
            &input_device,
            &input_config,
            move |data: Vec<f32>| {
                let mic_stream_clone = MIC_STREAM.clone();
                let mic_stream = mic_stream_clone.lock().unwrap();
                let samples = rodio::buffer::SamplesBuffer::new(channel, sample_rate, data);
                mic_stream.sink.as_ref().unwrap().append(samples);
            },
        )?;

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
        anyhow!("unable to send mic stream stop accept signal")
    })?;

    Ok(())
}

pub async fn stop_mic_streaming() -> Result<()> {
    if !MIC_STREAMING.load(Ordering::Acquire) {
        log::debug!("Mic streaming doesn't started, no need to stop mic streaming");
        return Ok(());
    }
    log::debug!("Stopping mic streaming");
    let (interrupted_tx, _) = &*MIC_STREAM_STOP;

    interrupted_tx.send(()).map_err(|e| {
        log::error!("Failed to send mic streaming stop signal, error: {}", e);
        anyhow!("unable to send mic streaming stop signal")
    })?;

    let (accept_tx, _) = &*MIC_STREAM_STOP_ACCEPT;
    let mut accept_rx = accept_tx.subscribe();

    // wait for signal
    _ = accept_rx.recv().await;

    Ok(())
}

pub async fn restart_mic_streaming() -> Result<()> {
    log::debug!("Try to restart mic streaming");
    stop_mic_streaming().await?;

    let manager = audio::AUDIO_STREAM_CONFIG_MANAGER.read().await;
    let config = manager.get_config();
    let is_streaming = config.stream_input && config.stream_mic_input;
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

pub async fn change_stream_config(stream: AudioStreamConfig) -> Result<AudioConfigResponseData> {
    let is_streaming = {
        let mut manager = audio::AUDIO_STREAM_CONFIG_MANAGER.write().await;
        let streaming = stream.stream_input && stream.stream_mic_input;
        manager.save_config(stream);
        streaming
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

pub async fn change_output_device(selection: AudioSelection) -> Result<AudioConfigResponseData> {
    match &selection {
        AudioSelection::Default(_) => {
            change_output_device_to_default().await?;
            get_audio_config().await
        }
        AudioSelection::ByName(SelectByName { name }) => {
            let host = cpal::default_host();
            let output_devices = host.output_devices()?;

            for (_, device) in output_devices.enumerate() {
                let device_name = device.name()?;
                if device_name == *name {
                    change_output_device_by_name(device_name.clone()).await?;
                    return get_audio_config().await;
                }
            }
            Err(anyhow!("No such output device of name {}", name))
        }
    }
}

pub async fn change_input_device(selection: AudioSelection) -> Result<AudioConfigResponseData> {
    match &selection {
        AudioSelection::Default(_) => {
            change_input_device_to_default().await?;
            get_audio_config().await
        }
        AudioSelection::ByName(SelectByName { name }) => {
            let host = cpal::default_host();
            let input_devices = host.input_devices()?;

            for (_, device) in input_devices.enumerate() {
                let device_name = device.name()?;
                if device_name == *name {
                    change_input_device_by_name(device_name.clone()).await?;
                    return get_audio_config().await;
                }
            }
            Err(anyhow!("No such input device of name {}", name))
        }
    }
}

pub async fn get_output_device() -> Result<Device> {
    let output = get_output_device_name().await;
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
        .ok_or(anyhow!("No default output device"))
}

pub async fn is_stream_input_enabled() -> bool {
    let manager = audio::AUDIO_STREAM_CONFIG_MANAGER.read().await;
    manager.get_config().stream_input
}

pub fn get_vb_audio_cable_output() -> Result<Device> {
    let host = cpal::default_host();
    let devices = host.output_devices().unwrap();

    for (_, device) in devices.enumerate() {
        let name = device.name().unwrap();
        if name == "CABLE Input (VB-Audio Virtual Cable)" {
            return Ok(device);
        }
    }

    Err(anyhow!("VB audio cable output not found"))
}

pub async fn get_input_device() -> Result<Device> {
    let input = get_input_device_name().await;
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
        .ok_or(anyhow!("No default input device"))
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

async fn check_audio_output_devices(
    previous: &Vec<String>,
    previous_default_output: String,
) -> Result<(bool, Option<(Vec<String>, String)>)> {
    let outputs = available_output_devices();
    let default_output = default_output_device();
    if outputs.is_err() {
        log::error!(
            "Unable to get current outputs, err: {}",
            outputs.unwrap_err()
        );
        return Ok((false, None));
    }
    if default_output.is_err() {
        log::error!(
            "Unable to get current default output, err: {}",
            default_output.unwrap_err()
        );
        return Ok((false, None));
    }
    let outputs = outputs?;
    let default_output = default_output?;
    if let Some(name) = get_output_device_name().await {
        let mut exists = false;
        for device_name in &outputs {
            if *device_name == name {
                exists = true;
                break;
            }
        }
        if !exists {
            log::warn!(
                "Current selected output device {} no longer exists, set to use default device",
                name.clone()
            );
            // if selected device no longer exists, change to use default device
            change_output_device_to_default().await?;
            return Ok((true, Some((outputs, default_output))));
        }
    }

    if !is_vec_equals(previous, &outputs) {
        return Ok((true, Some((outputs, default_output))));
    }

    Ok((
        previous_default_output != default_output,
        Some((outputs, default_output)),
    ))
}

async fn check_audio_input_devices(
    previous: &Vec<String>,
    previous_default_input: String,
) -> Result<(bool, Option<(Vec<String>, String)>)> {
    let inputs = available_input_devices();
    let default_input = default_input_device();
    if inputs.is_err() {
        log::error!("Unable to get current inputs, err: {}", inputs.unwrap_err());
        return Ok((false, None));
    }
    if default_input.is_err() {
        log::error!(
            "Unable to get current default input, err: {}",
            default_input.unwrap_err()
        );
        return Ok((false, None));
    }
    let inputs = inputs.unwrap();
    let default_input = default_input.unwrap();
    if let Some(name) = get_input_device_name().await {
        let mut exists = false;
        for device_name in &inputs {
            if *device_name == name {
                exists = true;
                break;
            }
        }
        if !exists {
            log::warn!(
                "Current selected input device {} no longer exists, set to use default device",
                name.clone()
            );
            // if selected device no longer exists, change to use default device
            change_input_device_to_default().await?;
            return Ok((true, Some((inputs, default_input))));
        }
    }

    if !is_vec_equals(previous, &inputs) {
        return Ok((true, Some((inputs, default_input))));
    }

    Ok((
        previous_default_input != default_input,
        Some((inputs, default_input)),
    ))
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
    Some((
        inputs.unwrap(),
        default_input.unwrap(),
        outputs.unwrap(),
        default_output.unwrap(),
    ))
}

pub async fn watch_audio_devices() {
    log::info!("Start watching audio devices");
    loop {
        let mut inputs: Option<Vec<String>> = None;
        let mut default_input: Option<String> = None;
        let mut outputs: Option<Vec<String>> = None;
        let mut default_output: Option<String> = None;
        if let Some((_inputs, _default_input, _outputs, _default_output)) =
            get_current_inputs_outputs()
        {
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
            default_output.as_ref().unwrap().clone(),
        )
        .await
        {
            Ok((changed, mut new_outputs)) => {
                if changed {
                    log::debug!("Found output devices changed");
                    app::silent_emit_all(constants::event::ON_AUDIO_CONFIG_CHANGE, ());
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
            default_input.as_ref().unwrap().clone(),
        )
        .await
        {
            Ok((changed, mut new_inputs)) => {
                if changed {
                    log::debug!("Found input devices changed");
                    app::silent_emit_all(constants::event::ON_AUDIO_CONFIG_CHANGE, ());
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
