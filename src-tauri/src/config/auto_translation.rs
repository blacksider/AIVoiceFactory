use std::collections::HashMap;
use tokio::sync::Mutex;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::gen_simple_config_manager;

static TRANSLATION_CONFIG: &str = "auto_translation";
static SALT: &str = "1435660288";

gen_simple_config_manager!(AutoTranslationConfigManager, AutoTranslationConfig, TRANSLATION_CONFIG, gen_default_config);

lazy_static! {
   pub static ref AUTO_TRANS_CONFIG_MANAGER: Mutex<AutoTranslationConfigManager> = Mutex::new(AutoTranslationConfigManager::init());
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
enum AutoTranslateTool {
    Baidu(TranslateByBaidu)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateByBaidu {
    api_addr: String,
    #[serde(rename = "appId")]
    app_id: String,
    secret: String,
    from: String,
    to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoTranslationConfig {
    enable: bool,
    tool: AutoTranslateTool,
}

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

impl AutoTranslationConfig {
    pub fn translated_by_baidu(self) -> (bool, Option<TranslateByBaidu>) {
        if !self.enable {
            return (false, None);
        }
        return match self.tool {
            AutoTranslateTool::Baidu(config) => {
                (true, Some(config))
            }
        };
    }
}

fn gen_default_config() -> AutoTranslationConfig {
    let empty_str = "".to_string();
    AutoTranslationConfig {
        enable: false,
        tool: AutoTranslateTool::Baidu(TranslateByBaidu {
            api_addr: empty_str.clone(),
            app_id: empty_str.clone(),
            secret: empty_str.clone(),
            from: "auto".to_string(),
            to: "jp".to_string(),
        }),
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
