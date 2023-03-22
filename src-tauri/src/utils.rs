use std::env::current_dir;
use std::path::PathBuf;

#[cfg(not(target_os = "windows"))]
use tauri::api::path::home_dir;

pub fn get_app_home_dir() -> PathBuf {
    // on windows, set to current app path
    #[cfg(target_os = "windows")]
    if let Err(e) = current_dir() {
        log::error!("Failed to get app home dir. error: {}", e);
        std::process::exit(-1);
    } else {
        return current_dir().unwrap();
    }
    // if not on windows, set to use app data path
    #[cfg(not(target_os = "windows"))]
    match home_dir() {
        None => {
            error!("Failed to get app home dir");
            std::process::exit(-1);
        }
        Some(path) => {
            // APP_DIR is tauri platform app data path
            return path.join(APP_DIR);
        }
    }
}
