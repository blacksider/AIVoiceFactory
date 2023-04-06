use reqwest::header::HeaderMap;
use reqwest::multipart::{Form, Part};
use reqwest::StatusCode;

use crate::config::voice_recognition::RecognizeByWhisper;
use crate::controller::errors::{CommonError, ProgramError};

const REQ_TASK: &str = "transcribe";
const REQ_OUTPUT: &str = "txt";

pub async fn asr(config: &RecognizeByWhisper, data: Vec<u8>) -> Result<String, ProgramError> {
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
