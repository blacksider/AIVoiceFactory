use std::fs::File;
use std::io::Write;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use lazy_static::lazy_static;
use tokio::sync::{mpsc, Mutex as AsyncMutex};
use tokio::sync::RwLock as AsyncRwLock;

use crate::config::voice_engine::VoiceVoxEngineConfig;
use crate::controller::errors::ProgramError;
use crate::utils;

const DEVICE_CPU: &str = "cpu";
const DEVICE_CUDA: &str = "cuda";
const DEVICE_DIRECTML: &str = "directml";

const DATA_DIR: &str = "voicevox";
const ENGINE_DIR: &str = "voicevox_engine";

const OUTPUT: &str = "output.log";
const OUTPUT_ERR: &str = "err.log";

const DOWNLOADER_URL: &str =
    "https://github.com/VOICEVOX/voicevox_engine/releases/download/0.14.4/";
const DONWLOADER_FILE_CPU: &str = "voicevox_engine-windows-cpu-0.14.4.7z.001";
const DONWLOADER_FILE_CUDA: &str = "voicevox_engine-windows-nvidia-0.14.4.7z.001";
const DONWLOADER_FILE_DIRECTML: &str = "voicevox_engine-windows-directml-0.14.4.7z.001";

lazy_static! {
    static ref BIN_MANGER: Arc<AsyncRwLock<BinaryManager>> = Arc::new(AsyncRwLock::new(BinaryManager::new(None)));
    static ref ENGINE_PROCESS: Arc<AsyncMutex<EngineProcess>> = Arc::new(AsyncMutex::new(EngineProcess::new()));
    static ref ENGINE_PROCESS_INITIALIZED: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    static ref ENGINE_LOADING: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    static ref ENGINE_STOP_SIG: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

#[derive(Debug, Clone)]
struct BinaryOption {
    device: String,
}

fn get_data_path() -> PathBuf {
    PathBuf::from(DATA_DIR)
}

fn create_folder_if_not_exists<P: AsRef<Path>>(path: P) -> Result<(), ProgramError> {
    std::fs::create_dir_all(path).map_err(|e| ProgramError::from(e))
}

// check voicevox root data dir, create it if not exists
fn check_data_dir() -> Result<(), ProgramError> {
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
        match TcpListener::bind(("localhost", port)) {
            Ok(listener) => {
                drop(listener);
                return Some(port);
            }
            Err(_) => {
                // The port is not available
            }
        }
    }
    None
}

async fn run_engine_exe(cmd: String,
                        out: std::process::Stdio,
                        err_out: std::process::Stdio) -> Result<(), ProgramError> {
    let execution = Command::new("cmd")
        .args(["/C", cmd.as_str()])
        .stdout(out)
        .stderr(err_out)
        .spawn()?;

    let execution_mutex = Arc::new(AsyncMutex::new(execution));

    ENGINE_PROCESS_INITIALIZED.store(true, Ordering::Release);

    let (interrupted_tx, mut interrupted_rx) = mpsc::channel(1);
    tokio::spawn(async move {
        while !ENGINE_STOP_SIG.load(Ordering::Acquire) {
            thread::sleep(Duration::from_secs(1));
        }
        match interrupted_tx.send(true).await {
            Ok(_) => {
                log::debug!("Interrupt signal for voicevox engine process send success");
            }
            Err(err) => {
                log::debug!("Interrupt signal for voicevox engine process send failed, err: {}", err);
            }
        }
    });

    tokio::select! {
        exit = async {
            let mut execution = execution_mutex.lock().await;
            execution.wait()?;
            Ok::<(), ProgramError>(())
        } => {
            match exit {
                Ok(_) => {
                    log::debug!("Voicevox engine process exit unexpectedly");
                },
                Err(err) => {
                    log::debug!("Voicevox engine process exit with error: {}", err);
                }
            }
        }
        _ = interrupted_rx.recv() => {
            let mut execution = execution_mutex.lock().await;
            match execution.kill() {
                Ok(_) => {
                    log::debug!("Kill voicevox engine process");
                }
                Err(err) => {
                    log::error!("Kill voicevox engine process failed, err: {}", err);
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
                port: get_available_port().or(Some(8080)).unwrap(),
            },
        }
    }

    fn set_option(&mut self, opts: BinaryOption) {
        self.option.replace(opts);
    }

    fn get_engine_path(&self) -> PathBuf {
        return get_data_path()
            .join(format!("{}_{}",
                          ENGINE_DIR,
                          self.option.as_ref().unwrap().device.clone()));
    }

    // check voicevox core binary files, download it if not been downloaded
    async fn check_binary(&mut self) -> Result<(), ProgramError> {
        // check root data path
        check_data_dir()?;

        // check binary
        let engine_path = self.get_engine_path();
        log::debug!("Check voicevox engine at {}", engine_path.clone().to_str().unwrap());
        if engine_path.exists() && engine_path.is_dir() {
            log::debug!("Voicevox engine found, skip download");
            return Ok(());
        }

        // set downloading flag
        ENGINE_LOADING.store(true, Ordering::Release);

        // download binary
        self.download_engine().await
    }

    async fn check_and_download_binary(&mut self) -> Result<(), ProgramError> {
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

    async fn download_engine(&self) -> Result<(), ProgramError> {
        if self.option.is_none() {
            return Err(ProgramError::from("Option should not be none"));
        }
        let option = self.option.as_ref().unwrap();
        let exe = match &*option.device {
            DEVICE_CPU => DONWLOADER_FILE_CPU,
            DEVICE_CUDA => DONWLOADER_FILE_CUDA,
            DEVICE_DIRECTML => DONWLOADER_FILE_DIRECTML,
            _ => {
                return Err(ProgramError::from("unsupported device type"));
            }
        };
        let download_url = DOWNLOADER_URL.to_owned() + exe;

        let download_tmp_file = get_data_path().join("engine.tmp.7z");

        log::debug!("Download voicevox engine 7z file to {}",
            download_tmp_file.clone().to_str().unwrap());

        let (interrupted_tx, mut interrupted_rx) = mpsc::channel(1);
        tokio::spawn(async move {
            while !ENGINE_STOP_SIG.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            match interrupted_tx.send(true).await {
                Ok(_) => {
                    log::debug!("Interrupt signal send success");
                }
                Err(err) => {
                    log::debug!("Interrupt signal send failed, err: {}", err);
                }
            }
        });

        tokio::select! {
            response = async {
                log::debug!("Downloading engin file");
                let mut tmp_file = File::create(download_tmp_file.clone())?;
                // download 7z file
                let client = reqwest::Client::new();
                let mut response: reqwest::Response = client
                    .get(download_url)
                    .send()
                    .await?;

                while let Some(chunk) = response.chunk().await? {
                    tmp_file.write_all(&chunk)?;
                }

                log::debug!("Download engine file success, ready to decompress file");
                Ok::<_, ProgramError>(())
            } => {
                response?;

                // extract 7z file to specific folder
                log::debug!("Decompressing engine file");
                let decompress_to = self.get_engine_path();
                let stopped = Arc::new(AtomicBool::new(false));
                let tmp_file = File::open(download_tmp_file.clone())?;
                sevenz_rust::decompress_with_extract_fn(
                    tmp_file, decompress_to.clone(),
                    |entry, reader, dest| {
                        if ENGINE_STOP_SIG.load(Ordering::Acquire) {
                            stopped.store(true, Ordering::Release);
                            return Ok(false);
                        }
                        sevenz_rust::default_entry_extract_fn(entry, reader, dest)
                    }).or_else(|e| {
                        log::debug!("Failed to decompress engine file, err: {}", e);
                        Err(ProgramError::from("Decompress engine file error"))
                    })?;
                if stopped.load(Ordering::Acquire) {
                    // delete tmp file
                    std::fs::remove_file(download_tmp_file)?;
                    // delete extracted
                    std::fs::remove_dir_all(decompress_to.clone())?;
                    log::warn!("Manually stopped downloading, stop at decompress 7z file");
                    return Ok(());
                }
                // delete tmp file
                std::fs::remove_file(download_tmp_file)?;
                log::debug!("Decompress engine file success, decompress folder: {}",
                    decompress_to.to_str().unwrap());
            }
            _ = interrupted_rx.recv() => {
                // delete tmp file
                std::fs::remove_file(download_tmp_file)?;
                log::debug!("Manually stopped downloading, stop downloading 7z file");
            }
        }
        Ok(())
    }

    async fn initialize(&mut self) -> Result<(), ProgramError> {
        ENGINE_STOP_SIG.store(false, Ordering::Release);

        self.check_and_download_binary().await?;

        if ENGINE_STOP_SIG.load(Ordering::Acquire) {
            return Err(ProgramError::from("Initialized stopped"));
        }

        // start engine exe process
        let data_path = get_data_path();

        let exe = utils::find_file_in_dir(self.get_engine_path(), "run.exe")
            .ok_or(ProgramError::from("run.exe not found"))?;
        let exe = exe.to_str().ok_or("unable to parse engine run exe path to str")?;

        // wrap cmd
        let cmd = format!(
            "{} --host {} --port {}",
            exe,
            self.engine_params.host,
            self.engine_params.port
        );

        log::debug!("Execute voicevox engine with cmd: {}", cmd.clone());

        let output_path = data_path.join(OUTPUT);
        let output = File::create(output_path.clone())?;
        let out = std::process::Stdio::from(output);
        let err_output = data_path.join(OUTPUT_ERR);
        let err_output = File::create(err_output)?;
        let err_out = std::process::Stdio::from(err_output);

        // spawn run.exe
        tauri::async_runtime::spawn(async move {
            match run_engine_exe(cmd, out, err_out).await {
                Ok(_) => {
                    log::debug!("Voicevox engine exe exit");
                    ENGINE_PROCESS_INITIALIZED.store(false, Ordering::Release);
                }
                Err(err) => {
                    log::debug!("Run voicevox engine exe failed, err: {}", err);
                    ENGINE_PROCESS_INITIALIZED.store(false, Ordering::Release);
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
        let opt = BinaryOption {
            device: use_device,
        };
        BinaryManager {
            option: opt
        }
    }

    async fn set_option(&mut self, device: String) -> Result<(), ProgramError> {
        let is_changed = device != self.option.device;

        self.option.device = device;
        set_process_options(self.option.clone()).await;

        let initialized = ENGINE_PROCESS_INITIALIZED.load(Ordering::Acquire);
        if !initialized {
            self.initialize().await;
        } else {
            if is_changed {
                self.reinitialize().await?;
            }
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
        ENGINE_STOP_SIG.store(true, Ordering::Release);
    }

    async fn reinitialize(&mut self) -> Result<(), ProgramError> {
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

pub async fn check_and_load(config: VoiceVoxEngineConfig) -> Result<(), ProgramError> {
    let lock = BIN_MANGER.clone();
    let mut man = lock.write().await;
    man.set_option(config.device).await
}

pub async fn check_and_unload() -> Result<(), ProgramError> {
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
