use anyhow::{anyhow, Result};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::StatusCode;

use crate::config::auto_translation::TranslateByBaidu;
use crate::utils::http;

pub async fn translate(config: &TranslateByBaidu, text: String) -> Result<String> {
    log::debug!("Translate text by baidu api, source: {}", text);
    let client = http::new_http_client().await?;
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("application/x-www-form-urlencoded"),
    );
    headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0"));
    let res: reqwest::Response = client
        .post(config.to_owned().get_api())
        .form(&config.to_owned().build_params(&text))
        .headers(headers)
        .send()
        .await?;
    if res.status() == StatusCode::OK {
        let json: serde_json::Value = res.json().await?;
        let result = json["trans_result"][0]["dst"].as_str().unwrap();
        log::debug!("Translated text by baidu api, result: {}", result);
        Ok(result.to_string())
    } else {
        Err(anyhow!(
            "translate by baidu return status {}, error: {}",
            res.status(),
            res.text().await?
        ))
    }
}
