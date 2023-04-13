pub mod event {
    pub const WINDOW_CLOSE: &str = "close";

    pub const ON_AUDIO_GENERATED: &str = "on_audio_generated";
    pub const ON_WHISPER_MODEL_LOADED: &str = "on_whisper_model_loaded";
    pub const ON_AUDIO_CONFIG_CHANGE: &str = "on_audio_config_change";
    pub const ON_AUDIO_RECOGNIZE_TEXT: &str = "on_audio_recognize_text";
    pub const ON_VOICEVOX_ENGINE_LOADED: &str = "on_voicevox_engine_loaded";
}