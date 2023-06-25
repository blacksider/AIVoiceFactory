use anyhow::{anyhow, Result};
use reqwest::header::HeaderMap;
use reqwest::multipart::{Form, Part};
use reqwest::StatusCode;

use crate::config::voice_recognition::{
    RecognitionTool, RecognizeByWhisper, VoiceRecognitionConfig, WhisperConfigType,
};
use crate::controller::voice_recognition::whisper_lib;
pub use crate::controller::voice_recognition::whisper_lib::available_models;
pub use crate::controller::voice_recognition::whisper_lib::init_library as check_whisper_lib;
use crate::utils::{audio, http};

const REQ_TASK: &str = "transcribe";
const REQ_OUTPUT: &str = "txt";

async fn asr_by_http(config: &RecognizeByWhisper, samples: &Vec<f32>) -> Result<String> {
    let client = http::new_http_client().await?;

    let mut headers = HeaderMap::new();

    headers.insert("Content-Type", "multipart/form-data".parse().unwrap());
    headers.insert("Accept", "application/json".parse().unwrap());

    let data =
        unsafe { std::slice::from_raw_parts(samples.as_ptr() as *const _, samples.len() * 4) };

    let form = Form::new().part("audio_file", Part::bytes(data).file_name("asr.wav"));

    let mut query = vec![("task", REQ_TASK), ("output", REQ_OUTPUT)];
    let language;
    if config.language.is_some() {
        language = config.language.clone().unwrap();
        query.push(("language", &*language))
    }

    let query = query;
    let res: reqwest::Response = client
        .post(format!("{}/asr", config.api_addr))
        .query(&query)
        .multipart(form)
        .send()
        .await?;
    if res.status() == StatusCode::OK {
        let text: String = res.text().await?;
        Ok(text)
    } else {
        Err(anyhow!(
            "do whisper asr request with status: {}, error: {}",
            res.status(),
            res.text().await?
        ))
    }
}

async fn asr_by_lib(config: &RecognizeByWhisper, samples: &Vec<f32>) -> Result<String> {
    log::debug!("Do ast by whisper library");
    whisper_lib::recognize(config.language.clone(), samples).await
}

async fn asr_16k_mono(config: &RecognizeByWhisper, mono_16k_samples: &Vec<f32>) -> Result<String> {
    match config.config_type {
        WhisperConfigType::Http => asr_by_http(config, mono_16k_samples).await,
        WhisperConfigType::Binary => asr_by_lib(config, mono_16k_samples).await,
    }
}

async fn asr_mono(
    config: &RecognizeByWhisper,
    mono_samples: &Vec<f32>,
    sample_rate: u32,
) -> Result<String> {
    if sample_rate != 16000 {
        log::debug!("Convert audio to rate 16k");
        let sample_16k = samplerate::convert(
            sample_rate,
            16000,
            1,
            samplerate::ConverterType::SincBestQuality,
            mono_samples,
        )?;
        asr_16k_mono(config, &sample_16k).await
    } else {
        asr_16k_mono(config, mono_samples).await
    }
}

async fn asr_non_mono(
    config: &RecognizeByWhisper,
    samples: &Vec<f32>,
    channels: u16,
    sample_rate: u32,
) -> Result<String> {
    assert!(channels > 1, "None mono channels should be greater than 1");
    let mono_samples = audio::convert_to_mono(samples, channels);
    asr_mono(config, &mono_samples, sample_rate).await
}

pub async fn asr(
    config: &RecognizeByWhisper,
    data: &Vec<f32>,
    channels: u16,
    sample_rate: u32,
) -> Result<String> {
    if channels == 0 {
        return Err(anyhow!("unsupported input channel value: {}", channels));
    }
    if channels == 1 {
        asr_mono(config, data, sample_rate).await
    } else {
        asr_non_mono(config, data, channels, sample_rate).await
    }
}

fn get_whisper_lib_model(config: &VoiceRecognitionConfig) -> Option<String> {
    if config.enable {
        match config.tool.clone() {
            RecognitionTool::Whisper(whisper_config) => {
                if whisper_config.config_type == WhisperConfigType::Binary {
                    return Some(whisper_config.use_model);
                }
            }
        }
    }
    None
}

pub async fn update_model(
    old: &VoiceRecognitionConfig,
    current: &VoiceRecognitionConfig,
) -> Option<String> {
    let old_model = get_whisper_lib_model(old);
    let current_model = get_whisper_lib_model(current);
    if current_model.is_none() && old_model.is_some() {
        match whisper_lib::free_model().await {
            Ok(_) => {}
            Err(err) => {
                log::error!("Failed to free whisper model, err: {}", err);
            }
        }
    }
    if current_model.is_some() && old_model != current_model {
        let update_model = current_model.unwrap();
        match whisper_lib::update_model(update_model.clone()).await {
            Ok(_) => {
                return Some(update_model);
            }
            Err(err) => {
                log::error!("Failed to update whisper model, err: {}", err);
            }
        }
    }
    None
}
