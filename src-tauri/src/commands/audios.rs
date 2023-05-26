use crate::controller::{audio_recorder, generator};
use crate::controller::generator::{AudioCacheDetail, AudioCacheIndex};

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
pub async fn play_audio(index: String) -> Option<bool> {
    match generator::play_audio(index).await {
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
    log::info!("Call cmd generate audio by text: {}", text.clone());
    let index = generator::generate_audio(text).await;
    if let Some(cache) = index {
        // play generated silently
        generator::play_audio_silently(cache.name.clone());
        Some(cache)
    } else {
        None
    }
}

#[tauri::command]
pub async fn is_recorder_recording() -> bool {
    audio_recorder::is_recording().await
}