use bytes::Bytes;
use reqwest::header::HeaderMap;

use crate::config::voice_engine::VoiceVoxEngineConfig;
use crate::controller::errors::ResponseError;

async fn audio_query(config: &VoiceVoxEngineConfig, text: String) -> Result<serde_json::Value, ResponseError> {
    let client = reqwest::Client::new();
    let res: reqwest::Response = client
        .post(format!("{}/audio_query", config.build_api()))
        .query(&[("speaker", "1"), ("text", text.as_str())])
        .send()
        .await?;
    if res.status() == 200 {
        let res_json: serde_json::Value = res.json().await.map_err(ResponseError::from)?;
        Ok(res_json)
    } else {
        Err(ResponseError::new(res.status(), res.text().await?))
    }
}

async fn synthesis(config: &VoiceVoxEngineConfig, audio_data: serde_json::Value) -> Result<Bytes, ResponseError> {
    let mut headers = HeaderMap::new();

    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("Accept", "audio/wav".parse().unwrap());

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/synthesis", config.build_api()))
        .query(&[("speaker", "1")])
        .headers(headers)
        .json(&audio_data)
        .send()
        .await?;
    if res.status() == 200 {
        res.bytes().await.map_err(ResponseError::from)
    } else {
        Err(ResponseError::new(res.status(), res.text().await?))
    }
}

pub async fn gen_audio(config: &VoiceVoxEngineConfig, text: String) -> Result<Bytes, ResponseError> {
    let data = audio_query(config, text).await?;
    let audio = synthesis(config, data).await?;
    Ok(audio)
}
