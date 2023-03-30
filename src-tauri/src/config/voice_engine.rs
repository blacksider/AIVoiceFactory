use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::config::config;
use crate::controller::errors::ProgramError;

static VOICE_ENGINE_CONFIG: &str = "voice_engine";

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
    speaker_uuid: String,
    speaker_style_id: i32,
}

impl VoiceVoxEngineConfig {
    pub fn build_api(&self) -> String {
        format!("{}://{}", self.protocol, self.api_addr)
    }

    pub fn get_speaker(&self) -> i32 {
        self.speaker_style_id
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

fn gen_default_config() -> Result<VoiceEngineConfig, ProgramError> {
    let default_config = VoiceEngineConfig {
        engine_type: EngineType::VoiceVox,
        config: EngineConfig::VoiceVox(VoiceVoxEngineConfig {
            protocol: "http".to_string(),
            api_addr: String::new(),
            speaker_uuid: "7ffcb7ce-00ec-4bdc-82cd-45a8889e43ff".to_string(),
            speaker_style_id: 0,
        }),
    };
    config::save_config(VOICE_ENGINE_CONFIG, &default_config)?;
    Ok(default_config)
}

fn load_voice_engine_config() -> Result<VoiceEngineConfig, ProgramError> {
    let default_config = config::get_config_raw::<VoiceEngineConfig>(VOICE_ENGINE_CONFIG)?;
    if default_config.is_none() {
        let default_config = gen_default_config()?;
        return Ok(default_config);
    }
    config::load_config::<VoiceEngineConfig>(VOICE_ENGINE_CONFIG)
}

fn save_voice_engine_config(config: &VoiceEngineConfig) -> Result<(), ProgramError> {
    let default_config = config::get_config_raw::<VoiceEngineConfig>(VOICE_ENGINE_CONFIG)?;
    if default_config.is_none() {
        gen_default_config()?;
        return Ok(());
    }
    config::save_config(VOICE_ENGINE_CONFIG, config)
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
                speaker_uuid: "7ffcb7ce-00ec-4bdc-82cd-45a8889e43ff".to_string(),
                speaker_style_id: 0,
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
