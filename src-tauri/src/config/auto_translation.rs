use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

use crate::config::config;
use crate::config::config::ConfigError;
use crate::utils;

static TRANSLATION_CONFIG_FILE: &str = "auto_translation.cfg";

lazy_static! {
   pub static ref AUTO_TRANS_CONFIG_MANAGER: Mutex<AutoTranslationConfigManager> = Mutex::new(AutoTranslationConfigManager::init());
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumString, Serialize, Deserialize)]
pub enum TranslatorType {
    #[strum(serialize = "Baidu")]
    Baidu,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
enum AutoTranslateTool {
    Baidu(TranslateByBaidu)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateByBaidu {
    #[serde(rename = "apiAddr")]
    api_addr: String,
    #[serde(rename = "appId")]
    app_id: String,
    secret: String,
    from: String,
    to: String,
}

static SALT: &str = "1435660288";

impl TranslateByBaidu {
    pub fn get_api(self) -> String {
        self.api_addr.clone()
    }

    pub fn build_params(self, text: &str) -> HashMap<&'static str, String> {
        let mut params = HashMap::new();
        params.insert("q", text.to_owned());
        params.insert("from", self.from.to_owned());
        params.insert("to", self.to.to_owned());

        let concat = format!("{}{}{}{}", self.app_id, text, SALT, self.secret);
        let sign = md5::compute(concat);
        let sign = format!("{:x}", sign);

        params.insert("appid", self.app_id.to_owned());
        params.insert("salt", SALT.to_owned());
        params.insert("sign", sign);

        params
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoTranslationConfig {
    enable: bool,
    tool: AutoTranslateTool,
}

impl AutoTranslationConfig {
    pub fn translated_by_baidu(self) -> (bool, Option<TranslateByBaidu>) {
        if !self.enable {
            return (false, None);
        }
        match self.tool {
            AutoTranslateTool::Baidu(config) => {
                (true, Some(config))
            }
        }
    }
}

fn get_config_path() -> PathBuf {
    let config_path = utils::get_config_path().join(TRANSLATION_CONFIG_FILE);
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
    if !config_path.exists() {
        let default_config = gen_default_config(config_path.clone())?;
        return Ok(default_config);
    }
    config::load_config::<AutoTranslationConfig>(
        config_path.to_str().unwrap(),
    )
}

pub fn save_auto_translation_config(config: &AutoTranslationConfig) -> Result<(), ConfigError> {
    let config_path = get_config_path();
    if !config_path.exists() {
        gen_default_config(config_path.clone())?;
        return Ok(());
    }
    config::save_config(config_path.to_str().unwrap(),
                        config)
}

#[derive(Debug)]
pub struct AutoTranslationConfigManager {
    config: AutoTranslationConfig,
}

impl AutoTranslationConfigManager {
    pub fn init() -> Self {
        let config = load_auto_translation_config();
        match config {
            Ok(data) => AutoTranslationConfigManager { config: data },
            Err(err) => {
                log::error!("Failed to init auto translation config manager, load config with error: {}", err);
                panic!("Unable to init auto translation config manager");
            }
        }
    }

    pub fn get_config(&self) -> AutoTranslationConfig {
        self.config.clone()
    }

    pub fn save_config(&mut self, config: AutoTranslationConfig) -> bool {
        let result = save_auto_translation_config(&config);
        match result {
            Ok(_) => {
                self.config = config;
                true
            }
            Err(err) => {
                log::error!("Failed to save auto translation config, err: {}", err);
                false
            }
        }
    }
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
