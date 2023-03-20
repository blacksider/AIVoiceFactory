use std::path::PathBuf;
use std::sync::Mutex;

use crate::config::config;
use crate::config::config::ConfigError;

static VOICE_ENGINE_CONFIG_FILE: &str = "voice_engine.cfg";
static mutex: Mutex<i32> = Mutex::new(0);

#[derive(Debug, PartialEq, Eq, Hash, strum_macros::EnumString, serde::Serialize, serde::Deserialize)]
pub enum EngineType {
    #[strum(serialize = "VoiceVox")]
    VoiceVox,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct VoiceVoxEngineConfig {
    protocol: String,
    #[serde(rename = "apiAddr")]
    api_addr: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "config")]
enum EngineConfig {
    VoiceVox(VoiceVoxEngineConfig)
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct VoiceEngineConfig {
    #[serde(rename = "type")]
    engine_type: EngineType,
    config: EngineConfig,
}

fn get_config_path() -> PathBuf {
    let config_path = config::get_app_home_dir().join(config::CONFIG_FOLDER)
        .join(VOICE_ENGINE_CONFIG_FILE);
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

pub fn load_voice_engine_config() -> Result<VoiceEngineConfig, ConfigError> {
    let config_path = get_config_path();
    {
        let _unused = mutex.lock().unwrap();
        if !config_path.exists() {
            let default_config = gen_default_config(config_path.clone())?;
            return Ok(default_config);
        }
    }
    config::load_config::<VoiceEngineConfig>(
        config_path.to_str().unwrap(),
    )
}

pub fn save_voice_engine_config(config: &VoiceEngineConfig) -> Result<(), ConfigError> {
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
