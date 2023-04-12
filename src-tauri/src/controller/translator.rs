use crate::config::auto_translation;
use crate::controller::translators::baidu_translation;

pub async fn translate(text: String) -> Option<String> {
    let manager = auto_translation::AUTO_TRANS_CONFIG_MANAGER.lock().await;
    let config = manager.get_config();
    let (translate, by_baidu) = config.translated_by_baidu();
    if !translate {
        return None;
    }
    if let Some(baidu_config) = by_baidu {
        let result = baidu_translation::translate(&baidu_config, text)
            .await;
        match result {
            Ok(translated) => {
                return Some(translated);
            }
            Err(err) => {
                log::error!("Failed to generate voice vox audio, err: {}", err);
            }
        }
    }
    None
}
