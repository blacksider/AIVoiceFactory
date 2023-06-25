use lazy_static::lazy_static;
use tokio::sync::RwLock;

use crate::gen_simple_config_manager;

static AUDIO_SEL_CONFIG: &str = "audio_selection";
static AUDIO_STREAM_CONFIG: &str = "audio_stream";

gen_simple_config_manager!(
    AudioSelConfigManager,
    AudioSelectionConfig,
    AUDIO_SEL_CONFIG,
    gen_audio_sel_config
);
gen_simple_config_manager!(
    AudioStreamConfigManager,
    AudioStreamConfig,
    AUDIO_STREAM_CONFIG,
    gen_audio_stream_config
);

lazy_static! {
    pub static ref AUDIO_SEL_CONFIG_MANAGER: RwLock<AudioSelConfigManager> =
        RwLock::new(AudioSelConfigManager::init());
    pub static ref AUDIO_STREAM_CONFIG_MANAGER: RwLock<AudioStreamConfigManager> =
        RwLock::new(AudioStreamConfigManager::init());
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SelectByName {
    pub(crate) name: String,
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct SelectDefault {}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type")]
pub enum AudioSelection {
    Default(SelectDefault),
    ByName(SelectByName),
}

impl Default for AudioSelection {
    fn default() -> Self {
        AudioSelection::Default(SelectDefault::default())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioSelectionConfig {
    pub(crate) output: AudioSelection,
    pub(crate) input: AudioSelection,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioStreamConfig {
    pub(crate) stream_input: bool,
    pub(crate) stream_mic_input: bool,
}

fn gen_audio_sel_config() -> AudioSelectionConfig {
    AudioSelectionConfig {
        output: AudioSelection::default(),
        input: AudioSelection::default(),
    }
}

fn gen_audio_stream_config() -> AudioStreamConfig {
    AudioStreamConfig {
        stream_input: false,
        stream_mic_input: false,
    }
}
