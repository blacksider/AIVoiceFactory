#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VoiceVoxSpeakerStyle {
    pub(crate) id: u32,
    pub(crate) name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VoiceVoxSpeaker {
    pub(crate) name: String,
    pub(crate) speaker_uuid: String,
    pub(crate) version: String,
    pub(crate) styles: Vec<VoiceVoxSpeakerStyle>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VoiceVoxSpeakerStyleInfo {
    pub(crate) id: u32,
    pub(crate) icon: String,
    pub(crate) portrait: Option<String>,
    pub(crate) voice_samples: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VoiceVoxSpeakerInfo {
    pub(crate) policy: String,
    pub(crate) portrait: String,
    pub(crate) style_infos: Vec<VoiceVoxSpeakerStyleInfo>,
}
