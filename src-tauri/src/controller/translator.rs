use crate::config::auto_translation;
use crate::controller::translators::baidu_translation;

pub async fn translate(text: String) -> Option<String> {
    let manager = auto_translation::AUTO_TRANS_CONFIG_MANAGER.lock().await;
    let config = manager.get_config();
    let (translate, by_baidu) = config.translated_by_baidu();
    if !translate {
        log::info!("Translate not enabled, skip");
        return None;
    }
    if let Some(baidu_config) = by_baidu {
        log::info!("Translate by baidu, text: {}", text.clone());
        let result = baidu_translation::translate(&baidu_config, text)
            .await;
        match result {
            Ok(translated) => {
                return Some(translated);
            }
            Err(err) => {
                log::error!("Failed to translate text, err: {}", err);
            }
        }
    }
    None
}
