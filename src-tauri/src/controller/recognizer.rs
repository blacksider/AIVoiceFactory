use crate::config::voice_recognition;
use crate::controller::errors::ProgramError;
use crate::controller::voice_recognition::whisper;

pub async fn recognize(data: Vec<u8>) -> Result<String, ProgramError> {
    let config = tauri::async_runtime::spawn_blocking(move || {
        let manager = voice_recognition::VOICE_REC_CONFIG_MANAGER.lock().unwrap();
        manager.get_config()
    }).await;
    if config.is_err() {
        return Err(ProgramError::from("Failed to retrieve voice recognition config"));
    }
    let config = config.unwrap();
    let (rec, by_whisper) = config.recognize_by_whisper();
    if !rec {
        return Ok("".to_string());
    }
    if let Some(whisper_config) = by_whisper {
        let result = whisper::asr(&whisper_config, data)
            .await;
        return match result {
            Ok(recognized) => {
                Ok(recognized)
            }
            Err(err) => {
                Err(err)
            }
        };
    }
    return Err(ProgramError::from("No available recognizer"));
}
