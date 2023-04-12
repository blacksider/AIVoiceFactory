pub mod cmd {
    use std::sync::Arc;

    use lazy_static::lazy_static;
    use tauri::Manager;
    use tokio::sync::Mutex as AsyncMutex;

    use crate::config::{auto_translation, voice_engine, voice_recognition};
    use crate::config::auto_translation::AutoTranslationConfig;
    use crate::config::voice_engine::{VoiceEngineConfig, VoiceVoxEngineConfig};
    use crate::config::voice_recognition::VoiceRecognitionConfig;
    use crate::controller::{audio_manager, audio_recorder, generator};
    use crate::controller::audio_manager::{AudioConfigResponseData, AudioSelection};
    use crate::controller::generator::{AudioCacheDetail, AudioCacheIndex};
    use crate::controller::voice_engine::voicevox;
    use crate::controller::voice_engine::voicevox::model::{VoiceVoxSpeaker, VoiceVoxSpeakerInfo};
    use crate::controller::voice_recognition::whisper;

    lazy_static! {
        static ref GEN_AUDIO_LOCK: Arc<AsyncMutex<()>> = Arc::new(AsyncMutex::new(()));
    }


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
            voice_recognition::VOICE_REC_CONFIG_MANAGER.lock().await;
        Some(manager.get_config())
    }

    #[tauri::command]
    pub async fn save_voice_recognition_config(app_handle: tauri::AppHandle<tauri::Wry>,
                                               config: VoiceRecognitionConfig) -> bool {
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
            .lock().await;
        let success = manager.save_config(config.clone());

        if success {
            let old_config_clone = old_config.clone();
            let config_clone = config.clone();
            tauri::async_runtime::spawn(async move {
                whisper::update_model(&old_config_clone, &config_clone).await
            });
        }

        // try to update shortcut by original key and new key
        match audio_recorder::update_shortcut(&app_handle, &old_config, &config) {
            Ok(_) => {}
            Err(err) => {
                log::error!("Failed to update shortcut, err: {}", err)
            }
        }

        success
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
    pub async fn generate_audio(app_handle: tauri::AppHandle<tauri::Wry>,
                                text: String) -> Option<AudioCacheIndex> {
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
            app_handle.get_window("main")
                .unwrap()
                .emit("on_audio_generated", index.clone())
                .unwrap();
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
        let manager = voice_engine::VOICE_ENGINE_CONFIG_MANAGER.lock().await;
        let config = manager.get_config();
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
        if config.is_none() {
            return None;
        }
        let result = voicevox::speaker_info(&config.unwrap(), speaker_uuid).await;
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
    pub async fn is_recorder_recording() -> bool {
        audio_recorder::is_recording().await
    }
}
