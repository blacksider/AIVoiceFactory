use std::env::current_dir;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
#[cfg(not(target_os = "windows"))]
use tauri::api::path::home_dir;

pub fn get_app_home_dir() -> PathBuf {
    // on windows, set to current app path
    #[cfg(target_os = "windows")]
    if let Err(e) = current_dir() {
        log::error!("Failed to get app home dir. error: {}", e);
        std::process::exit(-1);
    } else {
        return current_dir().unwrap();
    }
    // if not on windows, set to use app data path
    #[cfg(not(target_os = "windows"))]
    match home_dir() {
        None => {
            error!("Failed to get app home dir");
            std::process::exit(-1);
        }
        Some(path) => {
            // APP_DIR is tauri platform app data path
            return path.join(APP_DIR);
        }
    }
}

#[derive(Debug)]
pub struct ConfigError {
    error: Box<dyn Error>,
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.error.as_ref())
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(error: serde_json::Error) -> Self {
        ConfigError { error: Box::new(error) }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(error: std::io::Error) -> Self {
        ConfigError { error: Box::new(error) }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.error)
    }
}

fn read_file(path: &str) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn write_file(file_path: &str, content: &str) -> std::io::Result<()> {
    let mut file = File::create(file_path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

pub fn save_config<T: Serialize>(config_file: &str, data: &T) -> Result<(), ConfigError> {
    let data_json = serde_json::to_string(data)
        .map_err(ConfigError::from)?;
    write_file(config_file, data_json.as_str())
        .map_err(ConfigError::from)
}

pub fn load_config<'a, T>(config_file: &'a str) -> Result<T, ConfigError>
    where T: 'a + DeserializeOwned {
    let data_json = read_file(config_file)
        .map_err(ConfigError::from)?;
    let result: T = serde_json::from_str(data_json.as_str()).map_err(ConfigError::from)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_read_write_file() {
        let path = "test.txt";
        let content = "This is a config!";
        write_file(path, content).unwrap();

        let file_content = read_file(path).unwrap();
        assert_eq!(file_content, content);
        fs::remove_file(path).unwrap();
    }
}
