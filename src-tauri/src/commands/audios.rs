use std::sync::Arc;

use lazy_static::lazy_static;
use tokio::sync::Mutex as AsyncMutex;

use crate::common::{app, constants};
use crate::controller::{audio_manager, audio_recorder, generator};
use crate::controller::audio_manager::{AudioConfigResponseData, AudioSelection};
use crate::controller::generator::{AudioCacheDetail, AudioCacheIndex};

lazy_static! {
    static ref GEN_AUDIO_LOCK: Arc<AsyncMutex<()>> = Arc::new(AsyncMutex::new(()));
}

#[tauri::command]
pub fn list_audios() -> Option<Vec<AudioCacheIndex>> {
    match generator::list_indices() {
        Ok(indices) => {
            return Some(indices);
        }
        Err(err) => {
            log::error!("Cannot list audios, err: {}",
                        err)
        }
    }
    None
}

#[tauri::command]
pub fn get_audio_detail(index: String) -> Option<AudioCacheDetail> {
    match generator::get_index_detail(index) {
        Ok(data) => {
            return Some(data);
        }
        Err(err) => {
            log::error!("Cannot get audio detail, err: {}",
                        err)
        }
    }
    None
}

#[tauri::command]
pub fn delete_audio(index: String) -> Option<bool> {
    match generator::delete_index(index) {
        Ok(_) => {
            return Some(true);
        }
        Err(err) => {
            log::error!("Cannot delete audio cache, err: {}", err)
        }
    }
    None
}


#[tauri::command]
pub fn play_audio(index: String) -> Option<bool> {
    match generator::play_audio(index) {
        Ok(data) => {
            return Some(data);
        }
        Err(err) => {
            log::error!("Cannot play audio, err: {}",
                        err)
        }
    }
    None
}

#[tauri::command]
pub async fn generate_audio(text: String) -> Option<AudioCacheIndex> {
    let lock = GEN_AUDIO_LOCK.try_lock();
    match lock {
        Ok(_) => {}
        Err(_) => {
            log::debug!("Generate audio is executing");
            return None;
        }
    }
    log::debug!("Call cmd generate audio by text: {}", text.clone());
    let mut audio_index = generator::generate_audio(text).await;
    if let Some(index) = audio_index.take() {
        // TODO emit audio list change event
        app::silent_emit_all(constants::event::ON_AUDIO_GENERATED, index.clone());
        match generator::play_audio(index.name.clone()) {
            Ok(_) => {}
            Err(err) => {
                log::error!("Cannot play audio {}, err: {}",
                            index.name.clone(),
                        err)
            }
        }
        log::debug!("Generated index: {:?}", index.clone());
        return Some(index);
    }
    None
}


#[tauri::command]
pub fn change_output_device(selection: AudioSelection) -> Option<AudioConfigResponseData> {
    match audio_manager::change_output_device(selection) {
        Ok(data) => {
            return Some(data);
        }
        Err(err) => {
            log::error!("Failed to change audio output device, err: {}", err);
        }
    }
    None
}

#[tauri::command]
pub fn change_input_device(selection: AudioSelection) -> Option<AudioConfigResponseData> {
    match audio_manager::change_input_device(selection) {
        Ok(data) => {
            return Some(data);
        }
        Err(err) => {
            log::error!("Failed to change audio input device, err: {}", err);
        }
    }
    None
}

#[tauri::command]
pub async fn is_recorder_recording() -> bool {
    audio_recorder::is_recording().await
}