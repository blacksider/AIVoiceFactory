use bytes::Bytes;

use model::{VoiceVoxSpeaker, VoiceVoxSpeakerInfo};

use crate::config::voice_engine::{VoiceVoxConfigType, VoiceVoxEngineConfig};
use crate::controller::errors::ProgramError;

mod binary;
mod http;
pub mod model;

pub async fn is_downloading() -> bool {
    binary::is_downloading().await
}

pub fn check_and_load(config: VoiceVoxEngineConfig) {
    if config.config_type != VoiceVoxConfigType::Binary {
        return;
    }
    tauri::async_runtime::spawn(async {
        match binary::check_and_load(config).await {
            Ok(_) => {
                log::debug!("Check and load voicevox success");
            }
            Err(err) => {
                log::debug!("Check and load voicevox failed, err: {}", err);
            }
        }
    });
}

pub fn check_and_unload() {
    tauri::async_runtime::spawn(async {
        match binary::check_and_unload().await {
            Ok(_) => {
                log::debug!("Check and unload voicevox success");
            }
            Err(err) => {
                log::debug!("Check and unload voicevox failed, err: {}", err);
            }
        }
    });
}

pub async fn gen_audio(config: &VoiceVoxEngineConfig, text: String) -> Result<Bytes, ProgramError> {
    match config.config_type {
        VoiceVoxConfigType::Http => {
            let data = http::audio_query(config, text).await?;
            let audio = http::synthesis(config, data).await?;
            Ok(audio)
        }
        VoiceVoxConfigType::Binary => {
            binary::tts(text, config.speaker_style_id).await
        }
    }
}

pub async fn speakers(config: &VoiceVoxEngineConfig) -> Result<Vec<VoiceVoxSpeaker>, ProgramError> {
    match config.config_type {
        VoiceVoxConfigType::Http => {
            http::speakers(config).await
        }
        VoiceVoxConfigType::Binary => {
            binary::speakers().await
        }
    }
}

pub async fn speaker_info(config: &VoiceVoxEngineConfig, speaker_uuid: String) -> Result<VoiceVoxSpeakerInfo, ProgramError> {
    match config.config_type {
        VoiceVoxConfigType::Http => {
            http::speaker_info(config, speaker_uuid).await
        }
        VoiceVoxConfigType::Binary => {
            binary::speaker_info(speaker_uuid).await
        }
    }
}