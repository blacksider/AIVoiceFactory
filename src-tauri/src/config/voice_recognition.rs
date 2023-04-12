use tokio::sync::Mutex;

use lazy_static::lazy_static;

use crate::config::config;
use crate::controller::errors::ProgramError;

static RECOGNITION_CONFIG: &str = "voice_recognition";

lazy_static! {
    pub static ref VOICE_REC_CONFIG_MANAGER: Mutex<VoiceRecognitionConfigManager> = Mutex::new(VoiceRecognitionConfigManager::init());
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

fn gen_default_config() -> Result<VoiceRecognitionConfig, ProgramError> {
    let empty_str = "".to_string();
    let default_config = VoiceRecognitionConfig {
        enable: false,
        record_key: "F1".to_string(),
        tool: RecognitionTool::Whisper(RecognizeByWhisper {
            config_type: WhisperConfigType::Http,
            use_model: "base".to_string(),
            api_addr: empty_str.clone(),
            language: None,
        }),
    };
    config::save_config(RECOGNITION_CONFIG, &default_config)?;
    Ok(default_config)
}

pub fn load_voice_recognition_config() -> Result<VoiceRecognitionConfig, ProgramError> {
    let default_config = config::get_config_raw::<VoiceRecognitionConfig>(RECOGNITION_CONFIG)?;
    if default_config.is_none() {
        let default_config = gen_default_config()?;
        return Ok(default_config);
    }
    config::load_config::<VoiceRecognitionConfig>(RECOGNITION_CONFIG)
}

pub fn save_voice_recognition_config(config: &VoiceRecognitionConfig) -> Result<(), ProgramError> {
    let default_config = config::get_config_raw::<VoiceRecognitionConfig>(RECOGNITION_CONFIG)?;
    if default_config.is_none() {
        gen_default_config()?;
        return Ok(());
    }
    config::save_config(RECOGNITION_CONFIG, config)
}

#[derive(Debug)]
pub struct VoiceRecognitionConfigManager {
    config: VoiceRecognitionConfig,
}

impl VoiceRecognitionConfigManager {
    pub fn init() -> Self {
        let config = load_voice_recognition_config();
        match config {
            Ok(data) => VoiceRecognitionConfigManager { config: data },
            Err(err) => {
                log::error!("Failed to init voice recognition config manager, load config with error: {}", err);
                panic!("Unable to init voice recognition config manager");
            }
        }
    }

    pub fn get_config(&self) -> VoiceRecognitionConfig {
        self.config.clone()
    }

    pub fn save_config(&mut self, config: VoiceRecognitionConfig) -> bool {
        let result = save_voice_recognition_config(&config);
        match result {
            Ok(_) => {
                self.config = config;
                true
            }
            Err(err) => {
                log::error!("Failed to save voice recognition config, err: {}", err);
                false
            }
        }
    }
}
