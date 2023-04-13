use crate::controller::voice_recognition::whisper;

#[tauri::command]
pub fn whisper_available_models() -> Option<Vec<String>> {
    let models = whisper::available_models();
    match models {
        Ok(models) => {
            Some(models)
        }
        Err(err) => {
            log::error!("Failed to list whisper available models, err: {}", err);
            None
        }
    }
}
