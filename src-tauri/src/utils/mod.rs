use std::env::current_dir;
use std::path::PathBuf;

pub mod http;

/// app home dir is current exe path
pub fn get_app_home_dir() -> PathBuf {
    if let Err(e) = current_dir() {
        log::error!("Failed to get app home dir. error: {}", e);
        std::process::exit(-1);
    } else {
        return current_dir().unwrap();
    }
}

/// find the the first occurrence of specific file with given file name inside given directory
pub fn find_file_in_dir(dir: PathBuf, file_name: &str) -> Option<PathBuf> {
    for entry in walkdir::WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name() == file_name {
            return Some(entry.into_path());
        }
    }
    None
}
