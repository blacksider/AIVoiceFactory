use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

use crate::config::config;
use crate::config::config::ConfigError;

static TRANSLATION_CONFIG_FILE: &str = "auto_translation.cfg";
static mutex: Mutex<i32> = Mutex::new(0);

#[derive(Debug, PartialEq, Eq, Hash, EnumString, Serialize, Deserialize)]
pub enum TranslatorType {
    #[strum(serialize = "Baidu")]
    Baidu,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
enum AutoTranslateTool {
    Baidu(TranslateByBaidu)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranslateByBaidu {
    #[serde(rename = "apiAddr")]
    api_addr: String,
    #[serde(rename = "appId")]
    app_id: String,
    secret: String,
    from: String,
    to: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutoTranslationConfig {
    enable: bool,
    tool: AutoTranslateTool,
}

fn get_config_path() -> PathBuf {
    let config_path = config::get_app_home_dir().join(config::CONFIG_FOLDER)
        .join(TRANSLATION_CONFIG_FILE);
    config_path
}

fn gen_default_config(config_path: PathBuf) -> Result<AutoTranslationConfig, ConfigError> {
    let empty_str = "".to_string();
    let default_config = AutoTranslationConfig {
        enable: false,
        tool: AutoTranslateTool::Baidu(TranslateByBaidu {
            api_addr: empty_str.clone(),
            app_id: empty_str.clone(),
            secret: empty_str.clone(),
            from: "auto".to_string(),
            to: "jp".to_string(),
        }),
    };
    config::save_config(config_path.to_str().unwrap(), &default_config)?;
    Ok(default_config)
}

pub fn load_auto_translation_config() -> Result<AutoTranslationConfig, ConfigError> {
    let config_path = get_config_path();
    {
        let _unused = mutex.lock().unwrap();
        if !config_path.exists() {
            let default_config = gen_default_config(config_path.clone())?;
            return Ok(default_config);
        }
    }
    config::load_config::<AutoTranslationConfig>(
        config_path.to_str().unwrap(),
    )
}

pub fn save_auto_translation_config(config: &AutoTranslationConfig) -> Result<(), ConfigError> {
    let config_path = get_config_path();
    {
        let _unused = mutex.lock().unwrap();
        if !config_path.exists() {
            gen_default_config(config_path.clone())?;
            return Ok(());
        }
    }
    config::save_config(config_path.to_str().unwrap(),
                        config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_config() {
        let api_addr = "localhost:8080".to_string();
        let config = &AutoTranslationConfig {
            enable: true,
            tool: AutoTranslateTool::Baidu(TranslateByBaidu {
                api_addr: api_addr.clone(),
                app_id: "app_1".to_string(),
                secret: "secret".to_string(),
                from: "abc".to_string(),
                to: "abc".to_string(),
            }),
        };
        let json_value = serde_json::to_string(&config).unwrap();
        let json_parsed = serde_json::from_str::<AutoTranslationConfig>(json_value.as_str()).unwrap();
        assert_eq!(json_parsed.enable, true);
        match json_parsed.tool {
            AutoTranslateTool::Baidu(config) => {
                assert_eq!(config.api_addr, api_addr);
            }
        }
    }
}
