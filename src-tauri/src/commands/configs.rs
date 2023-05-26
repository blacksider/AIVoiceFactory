use crate::common::{app, constants};
use crate::config::{auto_translation, voice_engine, voice_recognition};
use crate::config::auto_translation::AutoTranslationConfig;
use crate::config::voice_engine::VoiceEngineConfig;
use crate::config::voice_recognition::VoiceRecognitionConfig;
use crate::controller::{audio_manager, audio_recorder};
use crate::controller::audio_manager::{AudioConfigResponseData, AudioSelection, StreamConfig};
use crate::controller::voice_recognition::whisper;

#[tauri::command]
pub async fn get_voice_engine_config() -> Option<VoiceEngineConfig> {
    let manager = voice_engine::VOICE_ENGINE_CONFIG_MANAGER
        .lock().await;
    Some(manager.get_config())
}

#[tauri::command]
pub async fn save_voice_engine_config(config: VoiceEngineConfig) -> Option<bool> {
    let mut manager = voice_engine::VOICE_ENGINE_CONFIG_MANAGER
        .lock().await;
    Some(manager.save_config(config))
}

#[tauri::command]
pub async fn get_auto_translation_config() -> Option<AutoTranslationConfig> {
    let manager =
        auto_translation::AUTO_TRANS_CONFIG_MANAGER.lock().await;
    Some(manager.get_config())
}

#[tauri::command]
pub async fn save_auto_translation_config(config: AutoTranslationConfig) -> Option<bool> {
    let mut manager =
        auto_translation::AUTO_TRANS_CONFIG_MANAGER.lock().await;
    Some(manager.save_config(config))
}

#[tauri::command]
pub async fn get_voice_recognition_config() -> Option<VoiceRecognitionConfig> {
    let manager =
        voice_recognition::VOICE_REC_CONFIG_MANAGER.read().await;
    Some(manager.get_config())
}

#[tauri::command]
pub async fn save_voice_recognition_config(config: VoiceRecognitionConfig) -> bool {
    // try to get original config to extract record_key
    let old_config =
        voice_recognition::load_voice_recognition_config();
    if old_config.is_err() {
        log::error!("Unable to load current voice recognition config, err: {}",
                old_config.unwrap_err());
        return false;
    }
    let old_config = old_config.unwrap();

    let mut manager = voice_recognition::VOICE_REC_CONFIG_MANAGER
        .write()
        .await;
    let success = manager.save_config(config.clone());

    if success {
        let old_config_clone = old_config.clone();
        let config_clone = config.clone();
        tauri::async_runtime::spawn(async move {
            let mut updated = whisper::update_model(
                &old_config_clone, &config_clone).await;
            if let Some(model) = updated.take() {
                app::silent_emit_all(constants::event::ON_WHISPER_MODEL_LOADED, model);
            }
        });
    }

    // try to update shortcut by original key and new key
    match audio_recorder::update_shortcut(&old_config, &config) {
        Ok(_) => {}
        Err(err) => {
            log::error!("Failed to update shortcut, err: {}", err)
        }
    }

    success
}

#[tauri::command]
pub async fn get_audio_config() -> Option<AudioConfigResponseData> {
    match audio_manager::get_audio_config().await {
        Ok(data) => {
            return Some(data);
        }
        Err(err) => {
            log::error!("Failed to load audio config, err: {}", err);
        }
    }
    None
}

#[tauri::command]
pub async fn change_output_device(selection: AudioSelection) -> Option<AudioConfigResponseData> {
    match audio_manager::change_output_device(selection).await {
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
pub async fn change_input_device(selection: AudioSelection) -> Option<AudioConfigResponseData> {
    match audio_manager::change_input_device(selection).await {
        Ok(data) => {
            match audio_manager::restart_mic_streaming().await {
                Ok(_) => {}
                Err(err) => {
                    log::error!("Failed to restart mic streaming, err: {}", err);
                    return None;
                }
            }
            return Some(data);
        }
        Err(err) => {
            log::error!("Failed to change audio input device, err: {}", err);
        }
    }
    None
}

#[tauri::command]
pub async fn change_stream_config(stream: StreamConfig) -> Option<AudioConfigResponseData> {
    match audio_manager::change_stream_config(stream).await {
        Ok(data) => {
            return Some(data);
        }
        Err(err) => {
            log::error!("Failed to change stream config, err: {}", err);
        }
    }
    None
}
