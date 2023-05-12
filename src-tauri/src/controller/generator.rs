use std::io::Cursor;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use bytes::Bytes;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use rodio::{Decoder, OutputStream, Sink};
use sled::{IVec, Transactional};
use sled::transaction::ConflictableTransactionError;

use crate::common::{app, constants};
use crate::config::config::DB_MANAGER;
use crate::config::voice_engine;
use crate::controller::{audio_manager, translator};
use crate::controller::errors::ProgramError;
use crate::controller::voice_engine::voicevox;

static AUDIO_DATA_TREE_INDEX: &str = "tree_index";
static AUDIO_DATA_TREE_DATA: &str = "tree_data";

lazy_static! {
    static ref PLAY_MUTEX: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

const MAX_DATA_SIZE: usize = 30;


pub fn start_check_audio_caches() {
    log::info!("Start checking audio caches thread");
    std::thread::spawn(move || {
        loop {
            match check_audio_caches() {
                Ok(_) => {}
                Err(err) => {
                    log::error!("Unable to check audio caches, err: {}", err)
                }
            }
            std::thread::sleep(Duration::from_secs(60));
        }
    });
}

/// only save recent MAX_DATA_SIZE caches
pub fn check_audio_caches() -> Result<(), ProgramError> {
    log::debug!("Check audio caches");
    let index_tree = DB_MANAGER.clone().db
        .open_tree(AUDIO_DATA_TREE_INDEX)?;
    let data_tree = DB_MANAGER.clone().db
        .open_tree(AUDIO_DATA_TREE_DATA)?;
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioCacheIndex {
    pub name: String,
    pub time: String,
}

unsafe impl Send for AudioCacheIndex {}

unsafe impl Sync for AudioCacheIndex {}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AudioCache {
    name: String,
    time: String,
    source: String,
    translated: String,
    audio: Vec<u8>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AudioCacheDetail {
    source: String,
    translated: String,
}

impl From<AudioCache> for AudioCacheDetail {
    fn from(value: AudioCache) -> Self {
        AudioCacheDetail {
            source: value.source.clone(),
            translated: value.translated.clone(),
        }
    }
}

pub fn list_indices() -> Result<Vec<AudioCacheIndex>, ProgramError> {
    let index_tree = DB_MANAGER.clone().db
        .open_tree(AUDIO_DATA_TREE_INDEX)?;
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
        let cache = index_tree.get(key)?;
        if let Some(encoded) = cache {
            let decoded: AudioCacheIndex = bincode::deserialize(&encoded)?;
            result.push(decoded);
        }
    }
    Ok(result)
}

pub fn get_index_detail(index: String) -> Result<AudioCacheDetail, ProgramError> {
    let data_tree = DB_MANAGER.clone().db
        .open_tree(AUDIO_DATA_TREE_DATA)?;
    let cache = data_tree.get(index.clone().into_bytes())?;
    if let Some(encoded) = cache {
        let decoded: AudioCache = bincode::deserialize(&encoded)?;
        Ok(AudioCacheDetail::from(decoded))
    } else {
        Err(ProgramError::from(format!("No such record of index {}", index.clone())))
    }
}

pub fn delete_index(index: String) -> Result<(), ProgramError> {
    let index_tree = DB_MANAGER.clone().db
        .open_tree(AUDIO_DATA_TREE_INDEX)?;
    let data_tree = DB_MANAGER.clone().db
        .open_tree(AUDIO_DATA_TREE_DATA)?;
    (&index_tree, &data_tree)
        .transaction(|(tx_index_tree, tx_data_tree)| {
            tx_index_tree.remove(&*index)?;
            tx_data_tree.remove(&*index)?;
            Ok::<(), ConflictableTransactionError<>>(())
        })?;
    Ok(())
}

fn save_audio(source: String, translated: String, audio: Bytes) -> Result<AudioCacheIndex, ProgramError> {
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
        .open_tree(AUDIO_DATA_TREE_INDEX)?;
    let data_tree = DB_MANAGER.clone().db
        .open_tree(AUDIO_DATA_TREE_DATA)?;

    let encoded_index = bincode::serialize(&cache_index)?;
    let encoded_data = bincode::serialize(&cache_data)?;

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
    let translated_text: String = translated.or_else(|| Some(text.clone())).unwrap();
    let manager = voice_engine::VOICE_ENGINE_CONFIG_MANAGER.lock().await;
    let config = manager.get_config();

    if config.is_voice_vox_config() {
        let voice_vox_config = config.get_voice_vox_config().unwrap();
        log::info!("Generating audio by voicevox with text: {}", translated_text.clone());
        let audio_data = voicevox::gen_audio(&voice_vox_config, translated_text.clone()).await;
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
                        // send event
                        app::silent_emit_all(constants::event::ON_AUDIO_GENERATED, index.clone());
                        // play generated silently
                        play_audio_silently(index.name.clone());
                        log::debug!("Generated index: {:?}", index.clone());
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

fn play_audio_silently(index: String) {
    match play_audio(index.clone()) {
        Ok(_) => {
            log::debug!("Play audio {} silently success", index);
        }
        Err(err) => {
            log::error!("Cannot play audio {}, err: {}", index, err)
        }
    }
}

pub fn play_audio(index: String) -> Result<bool, ProgramError> {
    let data_tree = DB_MANAGER.clone().db.open_tree(AUDIO_DATA_TREE_DATA)?;
    let cache = data_tree.get(index.into_bytes())?;
    if let Some(encoded) = cache {
        let mutex = PLAY_MUTEX.clone();
        let playing = mutex.load(Ordering::Relaxed);
        if !playing {
            mutex.store(true, Ordering::Relaxed);
            tauri::async_runtime::spawn(async move {
                match play_encoded_audio(&encoded).await {
                    Ok(_) => {}
                    Err(err) => {
                        log::error!("Unable to play encoded audio, err: {}", err)
                    }
                }
                mutex.store(false, Ordering::Relaxed);
            });
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

async fn play_wav_audio(wav_bytes: Vec<u8>) -> Result<(), ProgramError> {
    let cursor = Cursor::new(wav_bytes);
    let source = Decoder::new(cursor)?;
    let output_device = audio_manager::get_output_device().await?;
    let (_stream, stream_handle) = OutputStream::try_from_device(&output_device)?;
    let sink = Sink::try_new(&stream_handle)?;
    sink.append(source);
    sink.play();
    sink.sleep_until_end();
    Ok(())
}

fn play_to_vb_audio_cable(wav_bytes: Vec<u8>) -> Result<(), ProgramError> {
    let cursor = Cursor::new(wav_bytes);
    let source = Decoder::new(cursor)?;
    let output_device = audio_manager::get_vb_audio_cable_output()?;
    let (_stream, stream_handle) = OutputStream::try_from_device(&output_device)?;
    let sink = Sink::try_new(&stream_handle)?;
    sink.append(source);
    sink.play();
    sink.sleep_until_end();
    Ok(())
}

async fn play_encoded_audio(encoded: &IVec) -> Result<(), ProgramError> {
    let decoded: AudioCache = bincode::deserialize(&encoded)?;
    let wav_bytes = decoded.audio;
    if audio_manager::is_stream_input_enabled().await {
        let wav_bytes = wav_bytes.clone();
        std::thread::spawn(|| {
            match play_to_vb_audio_cable(wav_bytes) {
                Ok(_) => {}
                Err(err) => {
                    log::error!("Failed to play by VB audio cable, err: {}", err);
                }
            }
        });
    }
    play_wav_audio(wav_bytes).await?;
    Ok(())
}

