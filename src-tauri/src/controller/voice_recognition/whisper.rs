use std::io::Cursor;

use reqwest::header::HeaderMap;
use reqwest::multipart::{Form, Part};
use reqwest::StatusCode;

use crate::config::voice_recognition::{RecognitionTool, RecognizeByWhisper, VoiceRecognitionConfig, WhisperConfigType};
use crate::controller::errors::{CommonError, ProgramError};
use crate::controller::voice_recognition::whisper_lib;
pub use crate::controller::voice_recognition::whisper_lib::available_models;
pub use crate::controller::voice_recognition::whisper_lib::init_library as check_whisper_lib;

const REQ_TASK: &str = "transcribe";
const REQ_OUTPUT: &str = "txt";

async fn asr_by_http(config: &RecognizeByWhisper, data: Vec<u8>) -> Result<String, ProgramError> {
    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();

    headers.insert("Content-Type", "multipart/form-data".parse().unwrap());
    headers.insert("Accept", "application/json".parse().unwrap());

    let form = Form::new()
        .part("audio_file", Part::bytes(data).file_name("asr.wav"));

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
        Err(ProgramError::from(CommonError::from_http_error(res.status(), res.text().await?)))
    }
}

// convert source wav to 16khz if it is not
fn convert_wav_16k(wav: Vec<f32>, spec: hound::WavSpec) -> Result<Vec<f32>, ProgramError> {
    if spec.sample_rate != 16000 {
        samplerate::convert(spec.sample_rate,
                            16000,
                            spec.channels as usize,
                            samplerate::ConverterType::SincBestQuality,
                            &wav)
            .map_err(|err| ProgramError::from(err))
    } else {
        Ok(wav)
    }
}

async fn asr_by_lib(config: &RecognizeByWhisper, data: Vec<u8>) -> Result<String, ProgramError> {
    log::debug!("Do ast by whisper library");
    let mut buffer = Cursor::new(data);
    let reader = hound::WavReader::new(&mut buffer).unwrap();
    let spec = reader.spec();
    // Collect the samples from the reader into a Vec<f32>
    log::debug!("Convert wav to 16k from {:?}{}", spec.sample_format, spec.bits_per_sample);
    // read samples as f32
    // TODO: is this always f32?
    let mut samples = Vec::new();
    for x in reader.into_samples::<f32>() {
        samples.push(x?);
    }

    // try to convert to 16k
    let samples = convert_wav_16k(samples, spec)?;
    whisper_lib::recognize(config, samples).await
}

pub async fn asr(config: &RecognizeByWhisper, data: Vec<u8>) -> Result<String, ProgramError> {
    match config.config_type {
        WhisperConfigType::Http => {
            asr_by_http(config, data).await
        }
        WhisperConfigType::Binary => {
            asr_by_lib(config, data).await
        }
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

pub async fn update_model(old: &VoiceRecognitionConfig, current: &VoiceRecognitionConfig) -> Option<String> {
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