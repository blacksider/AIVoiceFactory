use std::env::current_dir;
use std::path::PathBuf;

pub mod audio;
pub mod http;
pub mod windows;

/// app home dir is current exe path
pub fn get_app_home_dir() -> PathBuf {
    if let Err(e) = current_dir() {
        log::error!("Failed to get app home dir. error: {}", e);
        std::process::exit(-1);
    } else {
        current_dir().unwrap()
    }
}

/// find the the first occurrence of specific file with given file name inside given directory
pub fn find_file_in_dir(dir: PathBuf, file_name: &str) -> Option<PathBuf> {
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name() == file_name {
            return Some(entry.into_path());
        }
    }
    None
}

pub fn silent_remove_file(file: PathBuf) {
    if !file.is_file() {
        return;
    }
    match std::fs::remove_file(file.clone()) {
        Ok(_) => {}
        Err(err) => {
            log::error!(
                "Failed to remove file {}, err: {}",
                file.to_string_lossy().to_string(),
                err
            );
        }
    }
}

pub fn silent_remove_dir(dir: PathBuf) {
    if !dir.is_dir() {
        return;
    }
    match std::fs::remove_dir_all(dir.clone()) {
        Ok(_) => {}
        Err(err) => {
            log::error!(
                "Failed to remove dir {}, err: {}",
                dir.to_string_lossy().to_string(),
                err
            );
        }
    }
}
