use std::{fs::File, io::Write, path::PathBuf};

use reqwest::{Proxy, StatusCode};

use crate::config::proxy;
use crate::config::proxy::HttpProxyConfig;
use crate::controller::errors::{CommonError, ProgramError};

/// build http client proxy config from param: config
fn build_proxy(config: HttpProxyConfig) -> Result<Option<Proxy>, ProgramError> {
    if !config.enable {
        return Ok(None);
    }
    if config.hostname.is_none() {
        return Ok(None);
    }
    if config.port.is_none() {
        return Ok(None);
    }
    let proxy_url = format!("http://{}:{}",
                            config.hostname.unwrap(),
                            config.port.unwrap());
    let proxy = Proxy::all(&proxy_url)?;
    if !config.enable_auth {
        return Ok(Some(proxy));
    }
    if config.authentication.is_none() {
        return Ok(None);
    }
    let auth = config.authentication.unwrap();
    if auth.username.is_empty() || auth.password.is_empty() {
        return Ok(None);
    }
    Ok(Some(proxy.basic_auth(&*auth.username, &*auth.password)))
}

/// a common method for generating new http client with set configs such as http proxy config.
pub async fn new_http_client() -> Result<reqwest::Client, ProgramError> {
    let manager = proxy::HTTP_PROXY_CONFIG_MANAGER
        .read().await;
    let config = manager.get_config();
    let proxy_built = build_proxy(config)?;
    if let Some(proxy) = proxy_built {
        let client = reqwest::Client::builder()
            .proxy(proxy)
            .build()?;
        Ok(client)
    } else {
        Ok(reqwest::Client::new())
    }
}

/// a util method to concat api urls, make sure connected by one "/"
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
    let client = new_http_client().await?;
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

/// download from given url to given file location
pub async fn download(url: String, download_file: PathBuf) -> Result<(), ProgramError> {
    let client = new_http_client().await?;
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