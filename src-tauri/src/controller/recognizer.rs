use crate::config::voice_recognition;
use crate::controller::errors::ProgramError;
use crate::controller::voice_recognition::whisper;

pub async fn recognize(data: Vec<u8>) -> Result<String, ProgramError> {
    let manager = voice_recognition::VOICE_REC_CONFIG_MANAGER.lock().await;
    let config = manager.get_config();
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
