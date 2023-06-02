use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use lazy_static::lazy_static;
use serde::de::DeserializeOwned;
use serde::Serialize;
use sled::Db;

use crate::{cypher, utils};
use crate::controller::errors::{CommonError, ProgramError};

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

/// save any serializable data of given key
pub fn save_config<T: Serialize>(key: &str, data: &T) -> Result<(), ProgramError> {
    let config_tree = DB_MANAGER.clone().db
        .open_tree(TREE_CONFIG)
        .map_err(ProgramError::from)?;
    let data_json = serde_json::to_string(data)
        .map_err(ProgramError::from)?;
    let data_json = cypher::encrypt::encrypt(
        CONFIG_CIPHER_KEY,
        data_json.as_ref());
    config_tree.insert(key.as_bytes(), data_json.into_bytes())
        .map_err(ProgramError::from)?;
    Ok(())
}

/// get config raw data by given key
pub fn get_config_raw<T>(key: &str) -> Result<Option<Vec<u8>>, ProgramError> {
    let config_tree = DB_MANAGER.clone().db
        .open_tree(TREE_CONFIG)
        .map_err(ProgramError::from)?;
    let data_json = config_tree.get(key.as_bytes())
        .map_err(ProgramError::from)?;
    if data_json.is_none() {
        return Ok(None);
    }
    Ok(Some(data_json.unwrap().to_vec()))
}

/// try to load config of given key, if config does not exist,
/// use param `get_default` to get default config data and save it, then return it.
/// # params:
/// * `key`: config key name
/// * `get_default`: a function that return a default config data
pub fn load_config<'a, T>(key: &'a str, get_default: fn() -> T) -> Result<T, ProgramError>
    where T: 'a + Serialize + DeserializeOwned {
    let raw = get_config_raw::<T>(key)?;
    if let Some(raw) = raw {
        // if config exists, parse config raw
        let data = String::from_utf8(raw)
            .map_err(|_| {
                ProgramError::from(CommonError::new(
                    String::from("config data is not utf8 string")))
            })?;
        let data_decrypted = cypher::encrypt::decrypt(
            CONFIG_CIPHER_KEY,
            data.as_str(),
        );
        let result: T = serde_json::from_str(data_decrypted.as_str()).map_err(ProgramError::from)?;
        Ok(result)
    } else {
        // config does not exists, get default data and save it
        let default = get_default();
        save_config(key, &default)?;
        Ok(default)
    }
}

#[macro_export]
macro_rules! gen_simple_config_manager {
    ($manager_ident: ident, $config_type: ty, $config_key: expr, $gen_default_config: expr) => {
        #[derive(Debug)]
        pub struct $manager_ident {
            config: $config_type,
        }

        impl $manager_ident {
            pub fn init() -> Self {
                let config = crate::config::config::load_config::<$config_type>(
                    $config_key,
                    $gen_default_config);
                match config {
                    Ok(data) => $manager_ident {
                        config: data
                    },
                    Err(err) => {
                        log::error!("Failed to init config manager for {}, load config with error: {}",
                            $config_key, err);
                        panic!("Unable to init config manager for {}", $config_key);
                    }
                }
            }

            pub fn get_config(&self) -> $config_type {
                self.config.clone()
            }

            pub fn save_config(&mut self, config: $config_type) -> bool {
                let result = crate::config::config::save_config($config_key, &config);
                match result {
                    Ok(_) => {
                        self.config = config;
                        true
                    }
                    Err(err) => {
                        log::error!("Failed to save config {}, err: {}", $config_key, err);
                        false
                    }
                }
            }
        }
    }
}
