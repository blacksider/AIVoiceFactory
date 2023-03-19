use std::any::Any;
use std::fmt;
use std::fmt::Formatter;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::ser::SerializeMap;
use strum_macros::EnumString;

use crate::config::config;

#[derive(Debug, PartialEq, Eq, Hash, EnumString, Serialize, Deserialize)]
pub enum EngineType {
    #[strum(serialize = "VoiceVox")]
    VoiceVox,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VoiceVoxEngineConfig {
    protocol: String,
    apiAddr: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "config")]
enum EngineConfig {
    VoiceVox(VoiceVoxEngineConfig)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VoiceEngineConfig {
    #[serde(rename = "type")]
    engine_type: EngineType,
    config: EngineConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_config() {
        let protocol = "http".to_string();
        let apiAddr = "localhost:8080".to_string();
        let config = VoiceEngineConfig {
            engine_type: EngineType::VoiceVox,
            config: EngineConfig::VoiceVox(VoiceVoxEngineConfig {
                protocol: protocol.clone(),
                apiAddr: apiAddr.clone(),
            }),
        };
        let json_value = serde_json::to_string(&config).unwrap();
        let json_parsed = serde_json::from_str::<VoiceEngineConfig>(json_value.as_str()).unwrap();
        assert_eq!(json_parsed.engine_type, EngineType::VoiceVox);
        match json_parsed.config {
            EngineConfig::VoiceVox(config) => {
                assert_eq!(config.protocol, protocol);
                assert_eq!(config.apiAddr, apiAddr);
            }
        }
    }
}
