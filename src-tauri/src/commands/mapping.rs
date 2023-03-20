pub mod cmd {
    use crate::config::{auto_translation, voice_engine};
    use crate::config::auto_translation::AutoTranslationConfig;
    use crate::config::voice_engine::VoiceEngineConfig;

    #[tauri::command]
    pub fn get_voice_engine_config() -> Option<VoiceEngineConfig> {
        let config = voice_engine::load_voice_engine_config();
        match config {
            Ok(data) => {
                Some(data)
            }
            Err(err) => {
                log::error!("Failed to load voice engine config, err: {}", err);
                None
            }
        }
    }

    #[tauri::command]
    pub fn save_voice_engine_config(config: VoiceEngineConfig) -> Option<bool> {
        let result = voice_engine::save_voice_engine_config(&config);
        match result {
            Ok(_) => {
                Some(true)
            }
            Err(err) => {
                log::error!("Failed to save voice engine config, err: {}", err);
                None
            }
        }
    }

    #[tauri::command]
    pub fn get_auto_translation_config() -> Option<AutoTranslationConfig> {
        let config = auto_translation::load_auto_translation_config();
        match config {
            Ok(data) => {
                Some(data)
            }
            Err(err) => {
                log::error!("Failed to load auto translation config, err: {}", err);
                None
            }
        }
    }

    #[tauri::command]
    pub fn save_auto_translation_config(config: AutoTranslationConfig) -> Option<bool> {
        let result = auto_translation::save_auto_translation_config(&config);
        match result {
            Ok(_) => {
                Some(true)
            }
            Err(err) => {
                log::error!("Failed to save auto translation config, err: {}", err);
                None
            }
        }
    }
}
