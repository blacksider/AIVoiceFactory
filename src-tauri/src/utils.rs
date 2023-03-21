use std::env::current_dir;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use lazy_static::lazy_static;
#[cfg(not(target_os = "windows"))]
use tauri::api::path::home_dir;

static CONFIG_FOLDER: &str = "configs";

lazy_static! {
   static ref CONFIG_PATH: Mutex<PathBuf> = Mutex::new(get_app_home_dir().join(CONFIG_FOLDER));
}

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

pub fn get_config_path() -> PathBuf {
    let path = CONFIG_PATH.lock().unwrap();
    if !path.exists() {
        fs::create_dir_all(path.to_path_buf()).unwrap();
    }
    path.to_path_buf()
}
