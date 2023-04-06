pub mod cmd {
    use crate::config::{auto_translation, voice_engine, voice_recognition};
    use crate::config::auto_translation::AutoTranslationConfig;
    use crate::config::voice_engine::{VoiceEngineConfig, VoiceVoxEngineConfig};
    use crate::config::voice_recognition::VoiceRecognitionConfig;
    use crate::controller::{audio_manager, audio_recorder, generator};
    use crate::controller::audio_manager::{AudioConfigResponseData, AudioSelection};
    use crate::controller::generator::{AudioCacheDetail, AudioCacheIndex};
    use crate::controller::voice_engine::voice_vox;
    use crate::controller::voice_engine::voice_vox::{VoiceVoxSpeaker, VoiceVoxSpeakerInfo};

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
    pub fn get_voice_recognition_config() -> Option<VoiceRecognitionConfig> {
        let manager = voice_recognition::VOICE_REC_CONFIG_MANAGER
            .lock().unwrap();
        Some(manager.get_config())
    }

    #[tauri::command]
    pub fn save_voice_recognition_config(app_handle: tauri::AppHandle<tauri::Wry>, config: VoiceRecognitionConfig) -> Option<bool> {
        // try to get original config to extract record_key
        let old_config = voice_recognition::load_voice_recognition_config();
        if old_config.is_err() {
            log::error!("Unable to load current voice recognition config, err: {}", old_config.unwrap_err());
            return Some(false);
        }
        let old_config = old_config.unwrap();

        let mut manager = voice_recognition::VOICE_REC_CONFIG_MANAGER
            .lock().unwrap();
        let result = manager.save_config(config.clone());

        // try to update shortcut by original key and new key
        match audio_recorder::update_shortcut(&app_handle, &old_config, &config) {
            Ok(_) => {}
            Err(err) => {
                log::error!("Failed to update shortcut, err: {}", err)
            }
        }

        Some(result)
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
    pub fn get_audio_config() -> Option<AudioConfigResponseData> {
        match audio_manager::get_audio_config() {
            Ok(data) => {
                return Some(data);
            }
            Err(err) => {
                log::error!("Failed to load audio config, err: {}", err);
            }
        }
        None
    }

    async fn get_voice_vox_config() -> Option<VoiceVoxEngineConfig> {
        let config = tauri::async_runtime::spawn_blocking(move || {
            let manager = voice_engine::VOICE_ENGINE_CONFIG_MANAGER.lock().unwrap();
            manager.get_config()
        }).await;
        if config.is_err() {
            log::error!("Failed to retrieve voice engine config");
            return None;
        }
        let config = config.unwrap();
        if !config.is_voice_vox_config() {
            log::error!("Current voice engine is not voice vox");
            return None;
        }
        Some(config.get_voice_vox_config())
    }

    #[tauri::command]
    pub async fn get_voice_vox_speakers() -> Option<Vec<VoiceVoxSpeaker>> {
        let config = get_voice_vox_config().await;
        if config.is_none() {
            return None;
        }
        let result = voice_vox::speakers(&config.unwrap()).await;
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
        if config.is_none() {
            return None;
        }
        let result = voice_vox::speaker_info(&config.unwrap(), speaker_uuid).await;
        match result {
            Ok(res) => Some(res),
            Err(err) => {
                log::error!("Failed to load voice vox speaker info, err: {}", err);
                None
            }
        }
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
    pub fn is_recorder_recording() -> bool {
        audio_recorder::is_recording()
    }
}
