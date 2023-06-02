use lazy_static::lazy_static;
use tokio::sync::RwLock;

use crate::gen_simple_config_manager;

static RECOGNITION_CONFIG: &str = "voice_recognition";

gen_simple_config_manager!(VoiceRecognitionConfigManager, VoiceRecognitionConfig, RECOGNITION_CONFIG, gen_default_config);

lazy_static! {
    pub static ref VOICE_REC_CONFIG_MANAGER: RwLock<VoiceRecognitionConfigManager> = RwLock::new(VoiceRecognitionConfigManager::init());
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, strum_macros::EnumString, serde::Serialize, serde::Deserialize)]
pub enum WhisperConfigType {
    #[strum(serialize = "http")]
    Http,
    #[strum(serialize = "binary")]
    Binary,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecognizeByWhisper {
    pub(crate) config_type: WhisperConfigType,
    // tiny, tiny.en, base, base.en, small, small.en, medium, medium.en, large-v1, large
    pub(crate) use_model: String,
    pub(crate) api_addr: String,
    pub(crate) language: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum RecognitionTool {
    Whisper(RecognizeByWhisper)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VoiceRecognitionConfig {
    pub(crate) enable: bool,
    pub(crate) generate_after: bool,
    #[serde(rename = "recordKey")]
    pub(crate) record_key: String,
    pub(crate) tool: RecognitionTool,
}

impl VoiceRecognitionConfig {
    pub fn recognize_by_whisper(self) -> (bool, Option<RecognizeByWhisper>) {
        if !self.enable {
            return (false, None);
        }
        return match self.tool {
            RecognitionTool::Whisper(config) => {
                (true, Some(config))
            }
        };
    }
}

fn gen_default_config() -> VoiceRecognitionConfig {
    let empty_str = "".to_string();
    VoiceRecognitionConfig {
        enable: false,
        generate_after: false,
        record_key: "F1".to_string(),
        tool: RecognitionTool::Whisper(RecognizeByWhisper {
            config_type: WhisperConfigType::Http,
            use_model: "base".to_string(),
            api_addr: empty_str.clone(),
            language: None,
        }),
    }
}
