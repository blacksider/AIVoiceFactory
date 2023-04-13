use bytes::Bytes;

pub use binary::available_binaries;
pub use binary::is_initialized as is_binary_initialized;
pub use binary::is_loading as is_binary_loading;
pub use binary::stop_loading as stop_binary_loading;
use model::{VoiceVoxSpeaker, VoiceVoxSpeakerInfo};

use crate::config::voice_engine::{VoiceVoxConfigType, VoiceVoxEngineConfig};
use crate::controller::errors::ProgramError;

mod binary;
mod http;
pub mod model;

pub fn check_and_load_binary(config: VoiceVoxEngineConfig) {
    if config.config_type != VoiceVoxConfigType::Binary {
        return;
    }
    log::debug!("Check voicevox binary");
    tauri::async_runtime::spawn(async {
        _check_and_unload_binary().await;
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

async fn _check_and_unload_binary() {
    match binary::check_and_unload().await {
        Ok(_) => {
            log::debug!("Check and unload voicevox engine success");
        }
        Err(err) => {
            log::error!("Check and unload voicevox engine failed, err: {}", err);
        }
    }
}

pub fn check_and_unload_binary() {
    tauri::async_runtime::spawn(async {
        _check_and_unload_binary().await
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