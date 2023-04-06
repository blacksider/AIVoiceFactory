use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use cpal::{Stream, SupportedStreamConfig};
use cpal::traits::{DeviceTrait, StreamTrait};
use lazy_static::lazy_static;
use tauri::{AppHandle, GlobalShortcutManager, Manager, Window, WindowBuilder, WindowUrl, Wry};

use crate::config::voice_recognition;
use crate::config::voice_recognition::VoiceRecognitionConfig;
use crate::controller::{audio_manager, recognizer};
use crate::controller::errors::{CommonError, ProgramError};

lazy_static! {
    static ref RECORDER: Arc<Mutex<AudioRecorder>> = Arc::new(Mutex::new(AudioRecorder::new()));
    static ref RECORDING_POPUP: Arc<Mutex<RecordingPopupHandler>> = Arc::new(Mutex::new(RecordingPopupHandler::new()));
}

fn sample_format(format: cpal::SampleFormat) -> hound::SampleFormat {
    if format.is_float() {
        hound::SampleFormat::Float
    } else {
        hound::SampleFormat::Int
    }
}

fn wav_spec_from_config(config: &SupportedStreamConfig) -> hound::WavSpec {
    hound::WavSpec {
        channels: config.channels() as _,
        sample_rate: config.sample_rate().0 as _,
        bits_per_sample: (config.sample_format().sample_size() * 8) as _,
        sample_format: sample_format(config.sample_format()),
    }
}

enum BufferType {
    I8,
    I16,
    I32,
    F32,
}

struct BufferWrapper {
    buffer_type: BufferType,
    vi8: Vec<i8>,
    vi16: Vec<i16>,
    vi32: Vec<i32>,
    vf32: Vec<f32>,
}

impl BufferWrapper {
    fn new() -> Self {
        Self {
            buffer_type: BufferType::I8,
            vi8: vec![],
            vi16: vec![],
            vi32: vec![],
            vf32: vec![],
        }
    }

    fn clear(&mut self) {
        self.vi8.clear();
        self.vi16.clear();
        self.vi32.clear();
        self.vf32.clear();
    }

    fn extend_from_i8_slice(&mut self, other: &[i8]) {
        self.vi8.extend_from_slice(other);
    }

    fn extend_from_i16_slice(&mut self, other: &[i16]) {
        self.vi16.extend_from_slice(other);
    }

    fn extend_from_i32_slice(&mut self, other: &[i32]) {
        self.vi32.extend_from_slice(other);
    }

    fn extend_from_f32_slice(&mut self, other: &[f32]) {
        self.vf32.extend_from_slice(other);
    }
}

struct AudioRecorder {
    buffer: Arc<Mutex<BufferWrapper>>,
    stream: Option<Stream>,
    config: Option<SupportedStreamConfig>,
    should_stop: Arc<AtomicBool>,
    recording: Arc<AtomicBool>,
}

unsafe impl Send for AudioRecorder {}

unsafe impl Sync for AudioRecorder {}

fn check_buffer_type(sample_format: &cpal::SampleFormat) -> BufferType {
    match *sample_format {
        cpal::SampleFormat::I8 => BufferType::I8,
        cpal::SampleFormat::I16 => BufferType::I16,
        cpal::SampleFormat::I32 => BufferType::I32,
        cpal::SampleFormat::F32 => BufferType::F32,
        _ => BufferType::I8
    }
}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(BufferWrapper::new())),
            stream: None,
            config: None,
            should_stop: Arc::new(AtomicBool::new(false)),
            recording: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&mut self) -> Result<(), ProgramError> {
        log::debug!("Start recording");
        self.should_stop.store(false, Ordering::SeqCst);

        let device = audio_manager::get_input_device()?;
        let config = device.default_input_config()?;
        self.config = Some(config.clone());

        let err_fn = |err| eprintln!("an error occurred on the stream: {}", err);

        let buffer = self.buffer.clone();
        let mut buffer = buffer.lock().unwrap();
        buffer.buffer_type = check_buffer_type(&config.sample_format());
        buffer.clear();
        drop(buffer);

        let buffer = self.buffer.clone();
        let should_stop = self.should_stop.clone();

        let stream = match config.sample_format() {
            cpal::SampleFormat::I8 => device.build_input_stream(
                &config.into(),
                move |data: &[i8], _: &_| {
                    if !should_stop.load(Ordering::SeqCst) {
                        let mut buffer = buffer.lock().unwrap();
                        buffer.extend_from_i8_slice(data);
                    }
                },
                err_fn,
                None,
            )?,
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config.into(),
                move |data: &[i16], _: &_| {
                    if !should_stop.load(Ordering::SeqCst) {
                        let mut buffer = buffer.lock().unwrap();
                        buffer.extend_from_i16_slice(data);
                    }
                },
                err_fn,
                None,
            )?,
            cpal::SampleFormat::I32 => device.build_input_stream(
                &config.into(),
                move |data: &[i32], _: &_| {
                    if !should_stop.load(Ordering::SeqCst) {
                        let mut buffer = buffer.lock().unwrap();
                        buffer.extend_from_i32_slice(data);
                    }
                },
                err_fn,
                None,
            )?,
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data: &[f32], _: &_| {
                    if !should_stop.load(Ordering::SeqCst) {
                        let mut buffer = buffer.lock().unwrap();
                        buffer.extend_from_f32_slice(data);
                    }
                },
                err_fn,
                None,
            )?,
            sample_format => {
                return Err(ProgramError::wrap(CommonError::new(format!("Unsupported sample format '{}'", sample_format))));
            }
        };

        self.stream = Some(stream);
        self.stream.as_ref().unwrap().play().map_err(|e| format!("Failed to start stream: {}", e))?;
        self.recording.clone().store(true, Ordering::Release);

        Ok(())
    }

    pub fn stop(&mut self) -> Result<Vec<u8>, ProgramError> {
        log::debug!("Stop recording");
        self.should_stop.store(true, Ordering::SeqCst);
        self.stream.as_ref()
            .ok_or_else(|| "Unable to get stream reference")?
            .pause()
            .map_err(|e| format!("Failed to pause stream: {}", e))?;
        self.recording.clone().store(false, Ordering::Release);
        close_recording_popup();

        let buffer = self.buffer.clone();
        let buffer = buffer.lock().unwrap();
        let spec = wav_spec_from_config(self.config.as_ref().unwrap());
        let mut cursor = Cursor::new(Vec::new());
        let cursor_ref = &mut cursor;
        let mut writer = hound::WavWriter::new(cursor_ref, spec)
            .map_err(|e| format!("Failed to create wav writer: {}", e))?;

        match buffer.buffer_type {
            BufferType::I8 => {
                log::debug!("extend data i8, total {}", buffer.vi8.len());
                for &data in buffer.vi8.iter() {
                    writer
                        .write_sample(data)
                        .map_err(|e| format!("Failed to write sample: {}", e))?;
                }
            }
            BufferType::I16 => {
                log::debug!("extend data i16, total {}", buffer.vi16.len());
                for &data in buffer.vi16.iter() {
                    writer
                        .write_sample(data)
                        .map_err(|e| format!("Failed to write sample: {}", e))?;
                }
            }
            BufferType::I32 => {
                log::debug!("extend data i32, total {}", buffer.vi32.len());
                for &data in buffer.vi32.iter() {
                    writer
                        .write_sample(data)
                        .map_err(|e| format!("Failed to write sample: {}", e))?;
                }
            }
            BufferType::F32 => {
                log::debug!("extend data f32, total {}", buffer.vf32.len());
                for &data in buffer.vf32.iter() {
                    writer
                        .write_sample(data)
                        .map_err(|e| format!("Failed to write sample: {}", e))?;
                }
            }
        }
        writer.finalize().map_err(|e| format!("Failed to finalize wav file: {}", e))?;
        log::debug!("Write all buffer, cursor at {}", cursor.position());
        Ok(cursor.into_inner())
    }
}

pub fn is_recording() -> bool {
    let lock = RECORDER.clone();
    let recorder = lock.lock().unwrap();
    return recorder.recording.load(Ordering::Acquire);
}

pub async fn check_recorder(handle: AppHandle<Wry>) -> Result<String, ProgramError> {
    let buffer: Result<Option<Vec<u8>>, ProgramError> = tauri::async_runtime::spawn_blocking(move || {
        let lock = RECORDER.clone();
        let mut recorder = lock.lock().unwrap();
        if recorder.recording.load(Ordering::Acquire) {
            let buffer_res = recorder.stop();
            match buffer_res {
                Ok(buffer) => {
                    Ok(Some(buffer))
                }
                Err(err) => {
                    Err(err)
                }
            }
        } else {
            recorder.start()?;
            open_recording_popup(&handle);
            Ok(None)
        }
    }).await?;
    let buffer = buffer?;

    if let Some(buffer) = buffer {
        // send to recognizer
        let text = recognizer::recognize(buffer).await?;
        return Ok(text);
    }
    Ok("".to_string())
}

async fn check_recorder_async(handle: AppHandle<Wry>) -> Result<(), ProgramError> {
    match check_recorder(handle.app_handle()).await {
        Ok(text) => {
            if !text.is_empty() {
                log::debug!("Handle recorded text: {}", text);
                handle.get_window("main").unwrap()
                    .emit("on_audio_recognize_text", text)
                    .unwrap();
            }
            Ok(())
        }
        Err(err) => {
            Err(err)
        }
    }
}

fn recorder_handler(app: AppHandle<Wry>) {
    let app_handle = Box::new(app.clone());
    tauri::async_runtime::spawn(async move {
        match check_recorder_async( app_handle.app_handle()).await {
            Ok(_) => {}
            Err(err) => {
                log::error!("Unable to check recorder, err: {}", err)
            }
        };
    });
}

pub fn start_shortcut(app: &AppHandle<Wry>) -> Result<(), ProgramError> {
    let config = voice_recognition::load_voice_recognition_config()?;
    if config.enable {
        if !config.record_key.is_empty() {
            let key = config.record_key;
            let mut manager = app.global_shortcut_manager();
            if manager.is_registered(&*key)? {
                manager.unregister(&*key)?;
            }
            let app_handle = Box::new(app.clone());
            manager.register(&*key, move || {
                recorder_handler(app_handle.app_handle());
            })?;
        }
    }
    Ok(())
}

pub fn update_shortcut(app: &AppHandle<Wry>, original: &VoiceRecognitionConfig, new_config: &VoiceRecognitionConfig) -> Result<(), ProgramError> {
    let mut manager = app.global_shortcut_manager();

    // if original registered, unregister it
    if original.enable {
        if !original.record_key.is_empty() {
            let original_key = &*original.record_key.clone();
            if manager.is_registered(original_key)? {
                manager.unregister(original_key)?;
            }
        }
    }

    // if newer is available, register newer key
    if new_config.enable {
        if !new_config.record_key.is_empty() {
            let new_key = &*new_config.record_key.clone();
            let app_handle = Box::new(app.clone());
            manager.register(new_key, move || {
                recorder_handler(app_handle.app_handle());
            })?;
        }
    }
    Ok(())
}

struct RecordingPopupHandler {
    window: Option<Window<Wry>>,
}

impl RecordingPopupHandler {
    fn new() -> Self {
        Self {
            window: None
        }
    }

    fn set_window(&mut self, window: Window<Wry>) {
        self.window = Some(window);
    }

    fn display(&self) {
        if self.window.is_none() {
            return;
        }
        let w = self.window.as_ref().unwrap();
        w.show().expect("Cannot display recording popup");
    }

    fn close(&self) {
        if self.window.is_none() {
            return;
        }
        self.window.as_ref().unwrap().hide()
            .expect("Cannot close recording popup");
    }
}

fn open_recording_popup(app: &AppHandle<Wry>) {
    let lock = RECORDING_POPUP.clone();
    let mut handler = lock.lock().unwrap();
    if handler.window.is_none() {
        // on the left top
        let window = WindowBuilder::new(
            app,
            "recording_popup",
            WindowUrl::App("recording".parse().unwrap()))
            .resizable(false)
            .disable_file_drop_handler()
            .transparent(true)
            .inner_size(150.0, 30.0)
            .position(10.0, 10.0)
            .always_on_top(true)
            .skip_taskbar(true)
            .decorations(false)
            .build()
            .expect("failed to create recording popup window");
        handler.set_window(window);
    } else {
        handler.display();
    }
}

fn close_recording_popup() {
    let lock = RECORDING_POPUP.clone();
    let handler = lock.lock().unwrap();
    handler.close();
}