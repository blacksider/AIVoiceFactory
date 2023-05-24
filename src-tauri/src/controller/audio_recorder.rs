use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use lazy_static::lazy_static;
use tauri::{AppHandle, GlobalShortcutManager, Manager, Window, WindowBuilder, WindowUrl, Wry};
use tokio::sync::Mutex as AsyncMutex;

use crate::audio::talk;
use crate::audio::talk::TalkParams;
use crate::common::app;
use crate::config::voice_recognition;
use crate::config::voice_recognition::VoiceRecognitionConfig;
use crate::controller::errors::ProgramError;

lazy_static! {
    static ref RECORDER: Arc<AsyncMutex<AudioRecorder>> = Arc::new(AsyncMutex::new(AudioRecorder::new()));
    static ref RECORDING_POPUP: Arc<AsyncMutex<RecordingPopupHandler>> = Arc::new(AsyncMutex::new(RecordingPopupHandler::new()));
}

struct AudioRecorder {
    recording: Arc<AtomicBool>,
}

unsafe impl Send for AudioRecorder {}

unsafe impl Sync for AudioRecorder {}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            recording: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn start(&mut self) -> Result<(), ProgramError> {
        log::debug!("Start recording");

        talk::start(&TalkParams::default()).await?;

        self.recording.clone().store(true, Ordering::Release);

        Ok(())
    }

    // pub async fn stop(&mut self) -> Result<Vec<u8>, ProgramError> {
    pub fn stop(&mut self) -> Result<(), ProgramError> {
        log::debug!("Stop recording");

        talk::stop()?;

        self.recording.clone().store(false, Ordering::Release);

        Ok(())
    }
}

pub async fn is_recording() -> bool {
    let lock = RECORDER.clone();
    let recorder = lock.lock().await;
    return recorder.recording.load(Ordering::Acquire);
}

async fn check_recorder_async(handle: AppHandle<Wry>) -> Result<(), ProgramError> {
    let lock = RECORDER.clone();
    let mut recorder = lock.try_lock()
        .map_err(|_| ProgramError::from("recorder busy, try later"))?;
    if recorder.recording.load(Ordering::Acquire) {
        // if is recording, stop
        recorder.stop()?;
        close_recording_popup().await?;
    } else {
        //
        recorder.start().await?;
        open_recording_popup(&handle).await?;
    }
    Ok(())
}

async fn recorder_handler(app: AppHandle<Wry>) {
    match check_recorder_async(app.app_handle()).await {
        Ok(_) => {}
        Err(err) => {
            log::error!("Unable to check recorder, err: {}", err)
        }
    };
}

pub fn start_shortcut(app: AppHandle<Wry>) -> Result<(), ProgramError> {
    let config = voice_recognition::load_voice_recognition_config()?;
    if config.enable {
        if !config.record_key.is_empty() {
            let key = config.record_key;
            let mut manager = app.global_shortcut_manager();
            if manager.is_registered(&*key)? {
                manager.unregister(&*key)?;
            }
            let app_handle = Box::new(app.app_handle());
            manager.register(&*key, move || {
                let app_handle = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    recorder_handler(*app_handle).await;
                });
            })?;
        }
    }
    Ok(())
}

pub fn update_shortcut(original: &VoiceRecognitionConfig, new_config: &VoiceRecognitionConfig) -> Result<(), ProgramError> {
    let app = app::get_app_handle()?;
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
            let app_handle = Box::new(app.app_handle());
            manager.register(new_key, move || {
                let app_handle = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    recorder_handler(*app_handle).await;
                });
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

    fn display(&self) -> Result<(), ProgramError> {
        if self.window.is_none() {
            return Err(ProgramError::from("Init popup window first"));
        }
        let w = self.window.as_ref().unwrap();
        w.show()?;
        Ok(())
    }

    fn close(&self) -> Result<(), ProgramError> {
        if self.window.is_none() {
            return Err(ProgramError::from("Init popup window first"));
        }
        self.window.as_ref().ok_or("Failed to get window reference")?.hide()?;
        Ok(())
    }
}

async fn open_recording_popup(app: &AppHandle<Wry>) -> Result<(), ProgramError> {
    let lock = RECORDING_POPUP.clone();
    let mut handler = lock.lock().await;
    if handler.window.is_none() {
        // on the left top
        let window = WindowBuilder::new(
            app,
            "recording_popup",
            WindowUrl::App("recording".parse().unwrap()))
            .resizable(false)
            .disable_file_drop_handler()
            .transparent(true)
            .min_inner_size(50.0, 30.0)
            .max_inner_size(250.0, 300.0)
            .position(10.0, 10.0)
            .always_on_top(true)
            .skip_taskbar(true)
            .decorations(false)
            .build()?;
        handler.set_window(window);
    }
    handler.display()
}

async fn close_recording_popup() -> Result<(), ProgramError> {
    let lock = RECORDING_POPUP.clone();
    let handler = lock.lock().await;
    handler.close()
}