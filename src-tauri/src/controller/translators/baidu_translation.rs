use reqwest::header::{HeaderMap, HeaderValue};

use crate::config::auto_translation::TranslateByBaidu;
use crate::controller::errors::{CommonError, ProgramError};

pub async fn translate(config: &TranslateByBaidu, text: String) -> Result<String, ProgramError> {
    log::debug!("Translate text by baidu api, source: {}", text);
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/x-www-form-urlencoded"));
    headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0"));
    let res: reqwest::Response = client
        .post(config.to_owned().get_api())
        .form(&config.to_owned().build_params(&text))
        .headers(headers)
        .send()
        .await
        .map_err(ProgramError::from)?;
    if res.status() == 200 {
        let json: serde_json::Value = res.json().await.map_err(ProgramError::from)?;
        let result = json["trans_result"][0]["dst"].as_str().unwrap();
        log::debug!("Translated text by baidu api, result: {}", result);
        Ok(result.to_string())
    } else {
        Err(ProgramError::from(
            CommonError::from_http_error( res.status(),res.text().await?)))
    }
}
