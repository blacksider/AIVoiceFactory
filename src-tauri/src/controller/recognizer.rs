use crate::config::voice_recognition;
use crate::controller::voice_recognition::whisper;
use anyhow::{anyhow, Result};

pub async fn recognize(data: &Vec<f32>, channels: u16, sample_rate: u32) -> Result<String> {
    let config = {
        let manager = voice_recognition::VOICE_REC_CONFIG_MANAGER.read().await;
        manager.get_config()
    };
    let (rec, by_whisper) = config.recognize_by_whisper();
    if !rec {
        return Ok("".to_string());
    }
    if let Some(whisper_config) = by_whisper {
        let result = whisper::asr(&whisper_config, data, channels, sample_rate).await;
        return match result {
            Ok(recognized) => Ok(recognized),
            Err(err) => Err(err),
        };
    }
    Err(anyhow!("No available recognizer"))
}
