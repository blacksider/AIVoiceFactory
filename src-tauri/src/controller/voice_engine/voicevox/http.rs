use bytes::Bytes;
use reqwest::header::HeaderMap;
use reqwest::StatusCode;

use crate::config::voice_engine::VoiceVoxEngineConfig;
use crate::controller::errors::{CommonError, ProgramError};
use crate::controller::voice_engine::voicevox::model::{VoiceVoxSpeaker, VoiceVoxSpeakerInfo};
use crate::utils::http;

async fn concat_api(config: &VoiceVoxEngineConfig, suffix: &str) -> String {
    http::concat_api(&*config.build_api().await, suffix)
}

pub async fn audio_query(config: &VoiceVoxEngineConfig, text: String) -> Result<serde_json::Value, ProgramError> {
    let client = http::new_http_client().await?;
    let res: reqwest::Response = client
        .post(concat_api(config, "audio_query").await)
        .query(&[("speaker", config.get_speaker().to_string()), ("text", text)])
        .send()
        .await?;
    if res.status() == StatusCode::OK {
        let res_json: serde_json::Value = res.json().await.map_err(ProgramError::from)?;
        Ok(res_json)
    } else {
        Err(ProgramError::from(CommonError::from_http_error(res.status(), res.text().await?)))
    }
}

pub async fn synthesis(config: &VoiceVoxEngineConfig, audio_data: serde_json::Value) -> Result<Bytes, ProgramError> {
    let mut headers = HeaderMap::new();

    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("Accept", "audio/wav".parse().unwrap());

    let client = http::new_http_client().await?;
    let res = client
        .post(concat_api(config, "synthesis").await)
        .query(&[("speaker", config.get_speaker())])
        .headers(headers)
        .json(&audio_data)
        .send()
        .await?;
    if res.status() == StatusCode::OK {
        res.bytes().await.map_err(ProgramError::from)
    } else {
        Err(ProgramError::from(CommonError::from_http_error(res.status(), res.text().await?)))
    }
}


pub async fn speakers(config: &VoiceVoxEngineConfig) -> Result<Vec<VoiceVoxSpeaker>, ProgramError> {
    let url = concat_api(config, "speakers").await;
    log::debug!("load speakers: {}", url.clone());
    http::get_json(url).await
}

pub async fn speaker_info(config: &VoiceVoxEngineConfig, speaker_uuid: String) -> Result<VoiceVoxSpeakerInfo, ProgramError> {
    let suffix = format!("speaker_info?speaker_uuid={}", speaker_uuid);
    http::get_json(concat_api(config, &*suffix).await).await
}