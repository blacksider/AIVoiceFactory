use anyhow::{anyhow, Error, Result};
use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::net::TcpListener;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::common::{app, constants};
use crate::config::voice_engine::VoiceVoxEngineConfig;
use crate::utils;
use crate::utils::http;
use lazy_static::lazy_static;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::RwLock as AsyncRwLock;
use tokio::sync::{broadcast, Mutex as AsyncMutex};
use winapi::um::winbase::CREATE_NO_WINDOW;

const DEVICE_CPU: &str = "cpu";
const DEVICE_CUDA: &str = "cuda";
const DEVICE_DIRECTML: &str = "directml";

const DATA_DIR: &str = "voicevox";
const ENGINE_DIR_PREFIX: &str = "voicevox_engine";

const OUTPUT: &str = "output.log";
const OUTPUT_ERR: &str = "err.log";

const ENGINE_EXE: &str = "run.exe";

const DOWNLOADER_URL: &str =
    "https://github.com/VOICEVOX/voicevox_engine/releases/download/0.14.4/";
const DONWLOADER_FILE_CPU: &str = "voicevox_engine-windows-cpu-0.14.4.7z.001";
const DONWLOADER_FILE_CUDA: &str = "voicevox_engine-windows-nvidia-0.14.4.7z.001";
const DONWLOADER_FILE_DIRECTML: &str = "voicevox_engine-windows-directml-0.14.4.7z.001";

lazy_static! {
    static ref BIN_MANGER: Arc<AsyncRwLock<BinaryManager>> =
        Arc::new(AsyncRwLock::new(BinaryManager::new(None)));
    static ref ENGINE_PROCESS: Arc<AsyncMutex<EngineProcess>> =
        Arc::new(AsyncMutex::new(EngineProcess::new()));
    static ref ENGINE_PROCESS_INITIALIZED: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    static ref ENGINE_LOADING: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    static ref ENGINE_STOP_SIG: (Sender<()>, Receiver<()>) = broadcast::channel(1);
}

#[derive(Debug, Clone)]
struct BinaryOption {
    device: String,
}

fn get_data_path() -> PathBuf {
    PathBuf::from(DATA_DIR)
}

fn create_folder_if_not_exists<P: AsRef<Path>>(path: P) -> Result<()> {
    Ok(std::fs::create_dir_all(path)?)
}

// check voicevox root data dir, create it if not exists
fn check_data_dir() -> Result<()> {
    create_folder_if_not_exists(get_data_path())
}

#[derive(Debug, Clone)]
pub struct EngineParams {
    pub(crate) host: String,
    pub(crate) port: u16,
}

struct EngineProcess {
    option: Option<BinaryOption>,
    engine_params: EngineParams,
}

// try to get an unused port with platform allowed port range for dynamic programs
fn get_available_port() -> Option<u16> {
    let start: u16;
    let end: u16;
    #[cfg(target_os = "windows")]
    {
        start = 49152;
        end = 65535;
    }
    #[cfg(target_os = "linux")]
    {
        start = 32768;
        end = 60999;
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        start = 30000;
        end = 50000;
    }
    for port in start..=end {
        if let Ok(listener) = TcpListener::bind(("localhost", port)) {
            drop(listener);
            return Some(port);
        }
    }
    None
}

#[cfg(target_os = "windows")]
pub fn try_stop_engine_exe() -> Result<()> {
    let data_path = get_data_path();
    let data_path = data_path
        .to_str()
        .ok_or(anyhow!("cannot parse data path to str"))?;
    log::debug!("Check exe {}", ENGINE_EXE);
    let (process_exists, pid) = utils::windows::process_exists(ENGINE_EXE, data_path);
    if process_exists {
        log::debug!(
            "Found exe {} already running with pid {}, ready to stop it",
            ENGINE_EXE,
            pid
        );
        utils::windows::terminate_process(pid)?;
    }
    Ok(())
}

#[cfg(target_os = "windows")]
async fn run_engine_exe<I, S>(
    exe: String,
    args: I,
    out: std::process::Stdio,
    err_out: std::process::Stdio,
) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    try_stop_engine_exe()?;

    let mut handle = Command::new(exe.clone())
        .args(args)
        .stdout(out)
        .stderr(err_out)
        .creation_flags(CREATE_NO_WINDOW) // DETACHED_PROCESS flag
        .spawn()?;

    ENGINE_PROCESS_INITIALIZED.store(true, Ordering::Release);
    log::debug!("Voicevox engine exe started");

    let (interrupted_tx, _) = &*ENGINE_STOP_SIG;
    let mut interrupted_rx = interrupted_tx.subscribe();

    tokio::select! {
        _ = interrupted_rx.recv() => {
            log::debug!("Interrupt voicevox engine signal received");
            match handle.kill() {
                Ok(_) => {
                    log::debug!("Kill voicevox engine success");
                }
                Err(err) => {
                    log::error!("Kill voicevox engine failed with err: {}", err);
                }
            }
            ENGINE_PROCESS_INITIALIZED.store(false, Ordering::Release);
        }
    }

    Ok(())
}

impl EngineProcess {
    fn new() -> Self {
        EngineProcess {
            option: None,
            engine_params: EngineParams {
                host: "localhost".to_string(),
                port: get_available_port().unwrap_or(8080),
            },
        }
    }

    fn set_option(&mut self, opts: BinaryOption) {
        self.option.replace(opts);
    }

    fn get_engine_path(&self) -> PathBuf {
        get_data_path().join(format!(
            "{}_{}",
            ENGINE_DIR_PREFIX,
            self.option.as_ref().unwrap().device.clone()
        ))
    }

    // check voicevox core binary files, download it if not been downloaded
    async fn check_binary(&mut self) -> Result<()> {
        // check root data path
        check_data_dir()?;

        // check binary
        let engine_path = self.get_engine_path();
        log::debug!(
            "Check voicevox engine at {}",
            engine_path.clone().to_str().unwrap()
        );
        if engine_path.exists() && engine_path.is_dir() {
            log::debug!("Voicevox engine found, skip download");
            return Ok(());
        }

        // download binary
        self.download_engine().await
    }

    async fn check_and_download_binary(&mut self) -> Result<()> {
        // wrap check binary function, set downloading flag false after finished(error or not)
        match self.check_binary().await {
            Ok(_) => {
                ENGINE_LOADING.store(false, Ordering::Release);
                Ok(())
            }
            Err(err) => {
                ENGINE_LOADING.store(false, Ordering::Release);
                Err(err)
            }
        }
    }

    async fn download_engine(&self) -> Result<()> {
        if self.option.is_none() {
            return Err(anyhow!("Option should not be none"));
        }

        // set downloading flag
        ENGINE_LOADING.store(true, Ordering::Release);

        let option = self.option.as_ref().unwrap();
        let exe = match &*option.device {
            DEVICE_CPU => DONWLOADER_FILE_CPU,
            DEVICE_CUDA => DONWLOADER_FILE_CUDA,
            DEVICE_DIRECTML => DONWLOADER_FILE_DIRECTML,
            _ => {
                return Err(anyhow!("unsupported device type"));
            }
        };
        let download_url = DOWNLOADER_URL.to_owned() + exe;

        let download_tmp_file = get_data_path().join("engine.tmp.7z");

        log::debug!(
            "Download voicevox engine 7z file to {}",
            download_tmp_file.clone().to_str().unwrap()
        );

        let (interrupted_tx, _) = &*ENGINE_STOP_SIG;
        let mut interrupted_rx = interrupted_tx.subscribe();

        tokio::select! {
            response = async {
                log::debug!("Downloading engin file");
                let mut tmp_file = File::create(download_tmp_file.clone())?;
                // download 7z file
                let client = http::new_http_client().await?;
                let mut response: reqwest::Response = client
                    .get(download_url)
                    .send()
                    .await?;

                while let Some(chunk) = response.chunk().await? {
                    tmp_file.write_all(&chunk)?;
                }

                log::debug!("Download engine file success, ready to decompress file");

                 // extract 7z file to specific folder
                log::debug!("Decompressing engine file");
                let decompress_to = self.get_engine_path();
                let tmp_file = File::open(download_tmp_file.clone())?;
                sevenz_rust::decompress_with_extract_fn(
                    tmp_file, decompress_to.clone(),
                    |entry, reader, dest| {
                        sevenz_rust::default_entry_extract_fn(entry, reader, dest)
                    }).map_err(|e| {
                        log::debug!("Failed to decompress engine file, err: {}", e);
                        anyhow!("Decompress engine file error")
                })?;

                app::silent_emit_all(constants::event::ON_VOICEVOX_ENGINE_LOADED, true);

                log::debug!("Decompress engine file success, decompress folder: {}",
                    decompress_to.to_str().unwrap());

                Ok::<_, Error>(())
            } => {
                match response {
                    Ok(_) => {
                        // delete tmp file
                        utils::silent_remove_file(download_tmp_file);
                    }
                    Err(err) => {
                        log::error!("Download voicevox engine failed with err: {}", err);
                        // if download and decompress failed, delete tmp file and decompress folder
                        utils::silent_remove_file(download_tmp_file);
                        let decompress_to = self.get_engine_path();
                        utils::silent_remove_dir(decompress_to);
                    }
                }
            }
            _ = interrupted_rx.recv() => {
                // delete tmp file
                utils::silent_remove_file(download_tmp_file);
                log::debug!("Manually stopped downloading, stop downloading 7z file");
            }
        }
        Ok(())
    }

    async fn initialize(&mut self) -> Result<()> {
        self.check_and_download_binary().await?;

        // start engine exe process
        let data_path = get_data_path();

        let exe = utils::find_file_in_dir(self.get_engine_path(), ENGINE_EXE)
            .ok_or(anyhow!("run.exe not found"))?;
        let exe = exe.to_string_lossy().to_string();

        let output_path = data_path.join(OUTPUT);
        let output = File::create(output_path)?;
        let out = std::process::Stdio::from(output);
        let err_output = data_path.join(OUTPUT_ERR);
        let err_output = File::create(err_output)?;
        let err_out = std::process::Stdio::from(err_output);

        // spawn run.exe
        // wrap args
        let host = self.engine_params.host.clone();
        let port = format!("{}", self.engine_params.port);

        tauri::async_runtime::spawn(async move {
            let args = ["--host", host.as_str(), "--port", port.as_str()];
            log::debug!(
                "Run voicevox engine exe with command: {} {:?}",
                exe.clone(),
                args
            );

            match run_engine_exe(exe, args, out, err_out).await {
                Ok(_) => {
                    log::debug!("Voicevox engine exe exit");
                }
                Err(err) => {
                    log::debug!("Run voicevox engine exe failed, err: {}", err);
                }
            }
        });
        Ok(())
    }
}

struct BinaryManager {
    option: BinaryOption,
}

unsafe impl Send for BinaryManager {}

unsafe impl Sync for BinaryManager {}

async fn set_process_options(opt: BinaryOption) {
    let lock = ENGINE_PROCESS.clone();
    let mut p = lock.lock().await;
    p.set_option(opt);
}

impl BinaryManager {
    fn new(device: Option<String>) -> Self {
        // if device is not given, use cpu as default
        let use_device;
        if let Some(dev) = device {
            use_device = dev;
        } else {
            use_device = DEVICE_CPU.to_string();
        }
        let opt = BinaryOption { device: use_device };
        BinaryManager { option: opt }
    }

    async fn set_option(&mut self, device: String) -> Result<()> {
        let is_changed = device != self.option.device;

        self.option.device = device;
        set_process_options(self.option.clone()).await;

        let initialized = ENGINE_PROCESS_INITIALIZED.load(Ordering::Acquire);
        if !initialized {
            self.initialize().await;
        } else if is_changed {
            self.reinitialize().await?;
        }
        Ok(())
    }

    async fn initialize(&mut self) {
        tauri::async_runtime::spawn(async {
            let lock = ENGINE_PROCESS.clone();
            let mut p = lock.lock().await;
            match p.initialize().await {
                Ok(_) => {
                    log::debug!("Initialize voicevox engine success");
                }
                Err(err) => {
                    log::error!("Initialize voicevox engine failed with error: {}", err);
                }
            }
        });
    }

    async fn stop_engine(&mut self) {
        let (interrupted_tx, _) = &*ENGINE_STOP_SIG;
        match interrupted_tx.send(()) {
            Ok(_) => {
                log::debug!("Send voicevox engine stop signal success");
            }
            Err(err) => {
                log::error!("Failed to send voicevox engine stop signal, err: {}", err);
            }
        }
    }

    async fn reinitialize(&mut self) -> Result<()> {
        self.stop_engine().await;
        self.initialize().await;
        Ok(())
    }
}

pub fn is_initialized() -> bool {
    ENGINE_PROCESS_INITIALIZED.load(Ordering::Acquire)
}

pub fn is_loading() -> bool {
    ENGINE_LOADING.clone().load(Ordering::Acquire)
}

pub async fn stop_loading() {
    let lock = BIN_MANGER.clone();
    let mut man = lock.write().await;
    man.stop_engine().await
}

pub async fn check_and_load(config: VoiceVoxEngineConfig) -> Result<()> {
    log::debug!("Check and load voicevox binary");
    let lock = BIN_MANGER.clone();
    let mut man = lock.write().await;
    man.set_option(config.device).await
}

pub async fn check_and_unload() -> Result<()> {
    log::debug!("Check and unload voicevox binary");
    let lock = BIN_MANGER.clone();
    let mut man = lock.write().await;
    man.stop_engine().await;
    Ok(())
}

pub async fn get_engine_params() -> EngineParams {
    let lock = ENGINE_PROCESS.clone();
    let p = lock.lock().await;
    p.engine_params.clone()
}

/// list all binary programs in [DATA_DIR]
pub fn available_binaries() -> Result<Vec<String>> {
    let data_path = get_data_path();
    let mut models = vec![];

    let prefix = ENGINE_DIR_PREFIX.to_string() + "_";
    let prefix = prefix.as_str();
    for entry in walkdir::WalkDir::new(data_path)
        .max_depth(1)
        .into_iter()
        .filter_entry(|e| e.file_type().is_dir())
        .filter_map(|e| e.ok())
    {
        let file_name = entry.file_name().to_str();
        if file_name.is_some() {
            let file_name = file_name.ok_or(anyhow!("cannot parse entry file name"))?;
            if file_name.starts_with(prefix) {
                let mut stripped = file_name.strip_prefix(prefix);
                if let Some(stripped) = stripped.take() {
                    models.push(stripped.to_string());
                }
            }
        }
    }
    Ok(models)
}
