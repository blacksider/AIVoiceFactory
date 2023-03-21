use std::path::PathBuf;
use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::config::config;
use crate::config::config::ConfigError;
use crate::utils;

static VOICE_ENGINE_CONFIG_FILE: &str = "voice_engine.cfg";

lazy_static! {
  pub static ref VOICE_ENGINE_CONFIG_MANAGER: Mutex<VoiceEngineConfigManager> =
    Mutex::new(VoiceEngineConfigManager::init());
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, strum_macros::EnumString, serde::Serialize, serde::Deserialize)]
pub enum EngineType {
    #[strum(serialize = "VoiceVox")]
    VoiceVox,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VoiceVoxEngineConfig {
    protocol: String,
    #[serde(rename = "apiAddr")]
    api_addr: String,
}

impl VoiceVoxEngineConfig {
    pub fn build_api(&self) -> String {
        format!("{}://{}", self.protocol, self.api_addr)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "config")]
enum EngineConfig {
    VoiceVox(VoiceVoxEngineConfig)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VoiceEngineConfig {
    #[serde(rename = "type")]
    engine_type: EngineType,
    config: EngineConfig,
}

impl VoiceEngineConfig {
    pub fn is_voice_vox_config(&self) -> bool {
        self.engine_type == EngineType::VoiceVox
    }

    pub fn get_voice_vox_config(&self) -> VoiceVoxEngineConfig {
        match self.clone().config {
            EngineConfig::VoiceVox(config) => config
        }
    }
}

fn get_config_path() -> PathBuf {
    let config_path = utils::get_config_path().join(VOICE_ENGINE_CONFIG_FILE);
    config_path
}

fn gen_default_config(config_path: PathBuf) -> Result<VoiceEngineConfig, ConfigError> {
    let default_config = VoiceEngineConfig {
        engine_type: EngineType::VoiceVox,
        config: EngineConfig::VoiceVox(VoiceVoxEngineConfig {
            protocol: "http".to_string(),
            api_addr: String::new(),
        }),
    };
    config::save_config(config_path.to_str().unwrap(), &default_config)?;
    Ok(default_config)
}

fn load_voice_engine_config() -> Result<VoiceEngineConfig, ConfigError> {
    let config_path = get_config_path();
    if !config_path.exists() {
        let default_config = gen_default_config(config_path.clone())?;
        return Ok(default_config);
    }
    config::load_config::<VoiceEngineConfig>(
        config_path.to_str().unwrap(),
    )
}

fn save_voice_engine_config(config: &VoiceEngineConfig) -> Result<(), ConfigError> {
    let config_path = get_config_path();
    if !config_path.exists() {
        gen_default_config(config_path.clone())?;
        return Ok(());
    }
    config::save_config(config_path.to_str().unwrap(),
                        config)
}

#[derive(Debug)]
pub struct VoiceEngineConfigManager {
    config: VoiceEngineConfig,
}

impl VoiceEngineConfigManager {
    pub fn init() -> Self {
        let config = load_voice_engine_config();
        match config {
            Ok(data) => VoiceEngineConfigManager { config: data },
            Err(err) => {
                log::error!("Failed to init voice engine config manager, load config with error: {}", err);
                panic!("Unable to init voice engine config manager");
            }
        }
    }

    pub fn get_config(&self) -> VoiceEngineConfig {
        self.config.clone()
    }

    pub fn save_config(&mut self, config: VoiceEngineConfig) -> bool {
        let result = save_voice_engine_config(&config);
        match result {
            Ok(_) => {
                self.config = config;
                true
            }
            Err(err) => {
                log::error!("Failed to save voice engine config, err: {}", err);
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
        let protocol = "http".to_string();
        let api_addr = "localhost:8080".to_string();
        let config = VoiceEngineConfig {
            engine_type: EngineType::VoiceVox,
            config: EngineConfig::VoiceVox(VoiceVoxEngineConfig {
                protocol: protocol.clone(),
                api_addr: api_addr.clone(),
            }),
        };
        let json_value = serde_json::to_string(&config).unwrap();
        let json_parsed = serde_json::from_str::<VoiceEngineConfig>(json_value.as_str()).unwrap();
        assert_eq!(json_parsed.engine_type, EngineType::VoiceVox);
        match json_parsed.config {
            EngineConfig::VoiceVox(config) => {
                assert_eq!(config.protocol, protocol);
                assert_eq!(config.api_addr, api_addr);
            }
        }
    }
}
