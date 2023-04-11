use std::{fs::File, io::Write, path::PathBuf};

use reqwest::StatusCode;

use crate::controller::errors::{CommonError, ProgramError};

pub fn concat_api(base: &str, concat: &str) -> String {
    let base = if base.ends_with("/") {
        base.to_owned()
    } else {
        base.to_owned() + "/"
    };
    let concat = if concat.starts_with("/") {
        concat.strip_prefix("/").unwrap()
    } else {
        concat
    };
    base + concat
}

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

pub async fn download(url: String, download_file: PathBuf) -> Result<(), ProgramError> {
    let client = reqwest::Client::new();
    let mut response: reqwest::Response = client
        .get(url)
        .send()
        .await
        .map_err(ProgramError::from)?;
    let mut dest_file = File::create(download_file)?;
    while let Some(chunk) = response.chunk().await? {
        dest_file.write_all(&chunk)?;
    }
    Ok(())
}