use std::error::Error;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;

use bytes::Bytes;
use chrono::prelude::*;
use lazy_static::lazy_static;
use rodio::{Decoder, OutputStream, Sink};
use sled::Db;
use sled::transaction::ConflictableTransactionError;
use sled::Transactional;

use crate::config::voice_engine;
use crate::controller::translator;
use crate::controller::voice_engine::voice_vox;
use crate::utils;

static AUDIO_DATA_FOLDER: &str = "audio";
static AUDIO_DATA_TREE_INDEX: &str = "tree_index";
static AUDIO_DATA_TREE_DATA: &str = "tree_data";

lazy_static! {
  static ref DB_MANAGER: Arc<DbManager> = Arc::new(DbManager::new());
}

const MAX_DATA_SIZE: usize = 30;

struct DbManager {
    db: Db,
}

impl DbManager {
    fn new() -> Self {
        let path = get_audio_path();
        let db = sled::open(path).unwrap();
        DbManager { db }
    }
}

fn get_audio_path() -> PathBuf {
    let path = utils::get_app_home_dir()
        .join(AUDIO_DATA_FOLDER);
    if !path.exists() {
        fs::create_dir_all(path.to_path_buf()).unwrap();
    }
    path.to_path_buf()
}

/// only save recent MAX_DATA_SIZE caches
pub fn check_audio_caches() -> Result<(), Box<dyn Error>> {
    log::debug!("Check audio caches");
    let index_tree = DB_MANAGER.clone().db
        .open_tree(AUDIO_DATA_TREE_INDEX)
        .map_err(Box::new)?;
    let data_tree = DB_MANAGER.clone().db
        .open_tree(AUDIO_DATA_TREE_DATA)
        .map_err(Box::new)?;
    log::debug!("Currently contains {} audio caches",index_tree.len());
    if index_tree.len() > MAX_DATA_SIZE {
        let cleans = index_tree.len() - MAX_DATA_SIZE;
        let keys: Vec<Vec<u8>> = index_tree
            .iter()
            .keys()
            .take(cleans)
            .map(|key| key.unwrap().to_vec())
            .collect();
        (&index_tree, &data_tree)
            .transaction(|(tx_index_tree, tx_data_tree)| {
                for key in &keys {
                    tx_index_tree.remove(key.clone())?;
                    tx_data_tree.remove(key.clone())?;
                }
                Ok::<(), ConflictableTransactionError<>>(())
            })?;
        log::info!("Clean total {} audio caches", cleans)
    } else {
        log::debug!("No audio cache need to be clean");
    }

    Ok(())
}

fn new_index_name() -> String {
    let time: DateTime<Utc> = Utc::now();
    let formatted_time = time.format("%Y%m%d%H%M%S").to_string();
    formatted_time
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AudioCacheIndex {
    pub name: String,
    pub time: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AudioCache {
    name: String,
    time: String,
    source: String,
    translated: String,
    audio: Vec<u8>,
}

pub fn list_indices() -> Result<Vec<AudioCacheIndex>, Box<dyn Error>> {
    let index_tree = DB_MANAGER.clone().db
        .open_tree(AUDIO_DATA_TREE_INDEX)
        .map_err(Box::new)?;
    let from = if index_tree.len() > MAX_DATA_SIZE { index_tree.len() - MAX_DATA_SIZE } else { 0 };
    let keys: Vec<Vec<u8>> = index_tree
        .iter()
        .keys()
        .skip(from)
        .take(MAX_DATA_SIZE)
        .map(|key| key.unwrap().to_vec())
        .collect();

    let mut result = vec![];
    for key in keys {
        let cache = index_tree.get(key).map_err(Box::new)?;
        if let Some(encoded) = cache {
            let decoded: AudioCacheIndex = bincode::deserialize(&encoded)
                .map_err(Box::new)?;
            result.push(decoded);
        }
    }
    Ok(result)
}

fn save_audio(source: String, translated: String, audio: Bytes) -> Result<AudioCacheIndex, Box<dyn Error>> {
    let index_name = new_index_name();
    log::debug!("Save audio cache with index: {}", index_name);
    let time: DateTime<Utc> = Utc::now();
    let cache_index = AudioCacheIndex {
        name: index_name.clone(),
        time: time.to_rfc3339(),
    };
    let cache_data = AudioCache {
        name: index_name.clone(),
        time: time.to_rfc3339(),
        source,
        translated,
        audio: audio.to_vec(),
    };

    let index_tree = DB_MANAGER.clone().db
        .open_tree(AUDIO_DATA_TREE_INDEX)
        .map_err(Box::new)?;
    let data_tree = DB_MANAGER.clone().db
        .open_tree(AUDIO_DATA_TREE_DATA)
        .map_err(Box::new)?;

    let encoded_index = bincode::serialize(&cache_index)
        .map_err(Box::new)?;
    let encoded_data = bincode::serialize(&cache_data)
        .map_err(Box::new)?;

    (&index_tree, &data_tree)
        .transaction(|(tx_index_tree, tx_data_tree)| {
            tx_index_tree.insert(index_name.clone().into_bytes(), encoded_index.to_owned())?;
            tx_data_tree.insert(index_name.clone().into_bytes(), encoded_data.to_owned())?;
            Ok::<(), ConflictableTransactionError<>>(())
        })?;

    log::debug!("Save audio cache with index: {} finished", index_name);

    Ok(cache_index)
}

/// generate audio content and it's temporary wav content, and return current cache name
pub async fn generate_audio(text: String) -> Option<AudioCacheIndex> {
    let translated = translator::translate(text.clone()).await;
    let translated_text: String;
    match translated {
        None => {
            translated_text = text.clone();
        }
        Some(data) =>
            {
                translated_text = data;
            }
    }

    let config = tauri::async_runtime::spawn_blocking(move || {
        let manager = voice_engine::VOICE_ENGINE_CONFIG_MANAGER.lock().unwrap();
        manager.get_config()
    }).await;
    if config.is_err() {
        log::error!("Failed to retrieve voice engine config");
        return None;
    }
    let config = config.unwrap();
    if config.is_voice_vox_config() {
        let voice_vox_config = config.get_voice_vox_config();
        log::debug!("Generating audio by text: {}", translated_text.clone());
        let audio_data = voice_vox::gen_audio(&voice_vox_config, translated_text.clone()).await;
        log::debug!("Generate audio cache success");
        match audio_data {
            Ok(audio) => {
                let save = save_audio(
                    text.clone(),
                    translated_text.clone(),
                    audio);
                match save {
                    Ok(index) => {
                        log::debug!("Generate audio cache with index: {}", index.name.clone());
                        return Some(index);
                    }
                    Err(err) => {
                        log::error!("Failed to save voice vox audio, err: {}", err);
                    }
                }
            }
            Err(err) => {
                log::error!("Failed to generate voice vox audio, err: {}", err);
            }
        }
    }
    None
}

pub fn play_audio(index: String) -> Result<bool, Box<dyn Error>> {
    let data_tree = DB_MANAGER.clone().db.open_tree(AUDIO_DATA_TREE_DATA)
        .map_err(Box::new)?;
    let cache = data_tree.get(index.into_bytes())
        .map_err(Box::new)?;
    if let Some(encoded) = cache {
        let decoded: AudioCache = bincode::deserialize(&encoded)
            .map_err(Box::new)?;
        let wav_bytes = decoded.audio;
        let cursor = Cursor::new(wav_bytes);
        let source = Decoder::new(cursor)
            .map_err(Box::new)?;
        let (_stream, stream_handle) = OutputStream::try_default()
            .map_err(Box::new)?;
        let sink = Sink::try_new(&stream_handle)
            .map_err(Box::new)?;
        sink.append(source);
        sink.play();
        sink.sleep_until_end();
        Ok(true)
    } else {
        Ok(false)
    }
}
