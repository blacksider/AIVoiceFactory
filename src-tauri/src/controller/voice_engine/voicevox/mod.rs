use bytes::Bytes;

use model::{VoiceVoxSpeaker, VoiceVoxSpeakerInfo};

use crate::config::voice_engine::{VoiceVoxConfigType, VoiceVoxEngineConfig};
use crate::controller::errors::ProgramError;

mod binary;
mod http;
pub mod model;

pub fn is_binary_loading() -> bool {
    binary::is_loading()
}

pub fn is_binary_initialized() -> bool {
    binary::is_initialized()
}

pub async fn stop_binary_loading() {
    binary::stop_loading().await
}

pub fn check_and_load_binary(config: VoiceVoxEngineConfig) {
    if config.config_type != VoiceVoxConfigType::Binary {
        return;
    }
    tauri::async_runtime::spawn(async {
        match binary::check_and_load(config).await {
            Ok(_) => {
                log::debug!("Check and load voicevox engine success");
            }
            Err(err) => {
                log::debug!("Check and load voicevox engine failed, err: {}", err);
            }
        }
    });
}

pub fn try_stop_engine_exe() {
    match binary::try_stop_engine_exe() {
        Ok(_) => {
            log::debug!("Stop voicevox engine success");
        }
        Err(err) => {
            log::error!("Stop voicevox engine failed, err: {}", err);
        }
    }
}

pub fn check_and_unload_binary() {
    tauri::async_runtime::spawn(async {
        match binary::check_and_unload().await {
            Ok(_) => {
                log::debug!("Check and unload voicevox engine success");
            }
            Err(err) => {
                log::error!("Check and unload voicevox engine failed, err: {}", err);
            }
        }
    });
}

pub async fn build_engine_api() -> String {
    let params = binary::get_engine_params().await;
    format!("http://127.0.0.1:{}", params.port)
}

pub async fn gen_audio(config: &VoiceVoxEngineConfig, text: String) -> Result<Bytes, ProgramError> {
    let data = http::audio_query(config, text).await?;
    let audio = http::synthesis(config, data).await?;
    Ok(audio)
}

pub async fn speakers(config: &VoiceVoxEngineConfig) -> Result<Vec<VoiceVoxSpeaker>, ProgramError> {
    http::speakers(config).await
}

pub async fn speaker_info(config: &VoiceVoxEngineConfig, speaker_uuid: String) -> Result<VoiceVoxSpeakerInfo, ProgramError> {
    http::speaker_info(config, speaker_uuid).await
}