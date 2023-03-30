use reqwest::StatusCode;

use crate::controller::errors::{CommonError, ProgramError};

/// execute GET request by url and return specific type of object
pub async fn get_json<T>(url: String) -> Result<T, ProgramError>
    where T: serde::de::DeserializeOwned {
    let client = reqwest::Client::new();
    let res: reqwest::Response = client
        .get(url)
        .send()
        .await
        .map_err(ProgramError::from)?;
    if res.status() == StatusCode::OK {
        let res_json: String = res.text().await.map_err(ProgramError::from)?;
        serde_json::from_str::<T>(&res_json).map_err(ProgramError::from)
    } else {
        Err(ProgramError::from(CommonError::from_http_error(res.status(), res.text().await?)))
    }
}
