pub mod cmd {
    use crate::config::{auto_translation, voice_engine};
    use crate::config::auto_translation::AutoTranslationConfig;
    use crate::config::voice_engine::VoiceEngineConfig;
    use crate::controller::generator;
    use crate::controller::generator::AudioCacheIndex;

    #[tauri::command]
    pub fn get_voice_engine_config() -> Option<VoiceEngineConfig> {
        let manager = voice_engine::VOICE_ENGINE_CONFIG_MANAGER
            .lock().unwrap();
        Some(manager.get_config())
    }

    #[tauri::command]
    pub fn save_voice_engine_config(config: VoiceEngineConfig) -> Option<bool> {
        let mut manager = voice_engine::VOICE_ENGINE_CONFIG_MANAGER
            .lock().unwrap();
        Some(manager.save_config(config))
    }

    #[tauri::command]
    pub fn get_auto_translation_config() -> Option<AutoTranslationConfig> {
        let manager = auto_translation::AUTO_TRANS_CONFIG_MANAGER
            .lock().unwrap();
        Some(manager.get_config())
    }

    #[tauri::command]
    pub fn save_auto_translation_config(config: AutoTranslationConfig) -> Option<bool> {
        let mut manager = auto_translation::AUTO_TRANS_CONFIG_MANAGER
            .lock().unwrap();
        Some(manager.save_config(config))
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
    pub async fn generate_audio(text: String) -> Option<AudioCacheIndex> {
        let audio_index = generator::generate_audio(text).await;
        match audio_index {
            None => None,
            Some(index) => {
                match generator::play_audio(index.name.clone()) {
                    Ok(_) => {}
                    Err(err) => {
                        log::error!("Cannot play audio {}, err: {}",
                            index.name.clone(),
                        err)
                    }
                }
                Some(index)
            }
        }
    }

    #[tauri::command]
    pub fn check_audio_caches() -> Option<bool> {
        match generator::check_audio_caches() {
            Ok(_) => {
                return Some(true);
            }
            Err(err) => {
                log::error!("Failed to check audio caches, err: {}", err);
            }
        }
        None
    }
}
