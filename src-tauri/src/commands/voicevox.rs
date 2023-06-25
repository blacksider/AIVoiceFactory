use crate::config::voice_engine;
use crate::config::voice_engine::VoiceVoxEngineConfig;
use crate::controller::voice_engine::voicevox;
use crate::controller::voice_engine::voicevox::model::{VoiceVoxSpeaker, VoiceVoxSpeakerInfo};

#[tauri::command]
pub fn is_loading_voicevox_engine() -> bool {
    voicevox::is_binary_loading()
}

#[tauri::command]
pub fn is_voicevox_engine_initialized() -> bool {
    voicevox::is_binary_initialized()
}

#[tauri::command]
pub async fn check_voicevox_engine() {
    voice_engine::check_voicevox().await;
}

#[tauri::command]
pub async fn stop_loading_voicevox_engine() {
    voicevox::stop_binary_loading().await
}

#[tauri::command]
pub async fn available_voicevox_binaries() -> Option<Vec<String>> {
    let bins = voicevox::available_binaries();
    match bins {
        Ok(bins) => Some(bins),
        Err(err) => {
            log::error!("Failed to list voicevox available binaries, err: {}", err);
            None
        }
    }
}

#[tauri::command]
pub async fn get_voice_vox_speakers() -> Option<Vec<VoiceVoxSpeaker>> {
    let config = get_voice_vox_config().await;
    config.as_ref()?;
    let result = voicevox::speakers(&config.unwrap()).await;
    match result {
        Ok(res) => Some(res),
        Err(err) => {
            log::error!("Failed to load voice vox speakers, err: {}", err);
            None
        }
    }
}

#[tauri::command]
pub async fn get_voice_vox_speaker_info(speaker_uuid: String) -> Option<VoiceVoxSpeakerInfo> {
    let config = get_voice_vox_config().await;
    config.as_ref()?;
    let result = voicevox::speaker_info(&config.unwrap(), speaker_uuid).await;
    match result {
        Ok(res) => Some(res),
        Err(err) => {
            log::error!("Failed to load voice vox speaker info, err: {}", err);
            None
        }
    }
}

async fn get_voice_vox_config() -> Option<VoiceVoxEngineConfig> {
    let manager = voice_engine::VOICE_ENGINE_CONFIG_MANAGER.lock().await;
    let config = manager.get_config();
    if !config.is_voice_vox_config() {
        log::error!("Current voice engine is not voice vox");
        return None;
    }
    Some(config.get_voice_vox_config().unwrap())
}
