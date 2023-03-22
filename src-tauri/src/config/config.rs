use std::{fmt, fs};
use std::error::Error;
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;

use lazy_static::lazy_static;
use serde::de::DeserializeOwned;
use serde::Serialize;
use sled::{Db};

use crate::{cypher, utils};
use crate::controller::errors::CommonError;

static DATA_FOLDER: &str = "data";
static TREE_CONFIG: &str = "tree_config";
static CONFIG_CIPHER_KEY: &str = "aivoice_factory20230319@macarron";

lazy_static! {
    pub static ref DB_MANAGER: Arc<DbManager> = Arc::new(DbManager::new());
}

pub struct DbManager {
    pub(crate) db: Db,
}

impl DbManager {
    fn new() -> Self {
        let path = get_db_path();
        let db = sled::open(path).unwrap();
        DbManager { db }
    }
}

fn get_db_path() -> PathBuf {
    let path = utils::get_app_home_dir()
        .join(DATA_FOLDER);
    if !path.exists() {
        fs::create_dir_all(path.to_path_buf()).unwrap();
    }
    path.to_path_buf()
}

#[derive(Debug)]
pub struct ConfigError {
    error: Box<dyn Error>,
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.error.as_ref())
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(error: serde_json::Error) -> Self {
        ConfigError { error: Box::new(error) }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(error: std::io::Error) -> Self {
        ConfigError { error: Box::new(error) }
    }
}

impl From<sled::Error> for ConfigError {
    fn from(error: sled::Error) -> Self {
        ConfigError { error: Box::new(error) }
    }
}

impl From<CommonError> for ConfigError {
    fn from(error: CommonError) -> Self {
        ConfigError { error: Box::new(error) }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.error)
    }
}

pub fn save_config<T: Serialize>(key: &str, data: &T) -> Result<(), ConfigError> {
    let config_tree = DB_MANAGER.clone().db
        .open_tree(TREE_CONFIG)
        .map_err(ConfigError::from)?;
    let data_json = serde_json::to_string(data)
        .map_err(ConfigError::from)?;
    let data_json = cypher::encrypt::encrypt(
        CONFIG_CIPHER_KEY,
        data_json.as_ref());
    config_tree.insert(key.as_bytes(), data_json.into_bytes())
        .map_err(ConfigError::from)?;
    Ok(())
}

pub fn get_config_raw<T>(key: &str) -> Result<Option<Vec<u8>>, ConfigError> {
    let config_tree = DB_MANAGER.clone().db
        .open_tree(TREE_CONFIG)
        .map_err(ConfigError::from)?;
    let data_json = config_tree.get(key.as_bytes())
        .map_err(ConfigError::from)?;
    if data_json.is_none() {
        return Ok(None);
    }
    Ok(Some(data_json.unwrap().to_vec()))
}

pub fn load_config<'a, T>(key: &'a str) -> Result<T, ConfigError>
    where T: 'a + DeserializeOwned {
    let raw = get_config_raw::<T>(key)?;
    let data = String::from_utf8(raw.unwrap())
        .map_err(|_| {
            ConfigError::from(CommonError::new(
                String::from("config data is not utf8 string")))
        })?;
    let data_decrypted = cypher::encrypt::decrypt(
        CONFIG_CIPHER_KEY,
        data.as_str(),
    );
    let result: T = serde_json::from_str(data_decrypted.as_str()).map_err(ConfigError::from)?;
    Ok(result)
}
