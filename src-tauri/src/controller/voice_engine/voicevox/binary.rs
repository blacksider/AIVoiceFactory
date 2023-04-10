use std::ffi::{CStr, CString};
use std::fs::File;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use base64::{Engine as _, engine::general_purpose};
use bytes::Bytes;
use lazy_static::lazy_static;
use libloading::{Library, Symbol};
use tokio::sync::Mutex as AsyncMutex;
use tokio::sync::RwLock as AsyncRwLock;
#[cfg(target_os = "windows")]
use winapi::shared::minwindef::DWORD;
#[cfg(target_os = "windows")]
use winapi::um::errhandlingapi::GetLastError;
#[cfg(target_os = "windows")]
use winapi::um::libloaderapi;
#[cfg(target_os = "windows")]
use winapi::um::memoryapi::VirtualFree;
use winapi::um::processthreadsapi::OpenProcess;
#[cfg(target_os = "windows")]
use winapi::um::winbase::FormatMessageW;

use crate::config::voice_engine::VoiceVoxEngineConfig;
use crate::controller::errors::ProgramError;
use crate::controller::voice_engine::voicevox::model::{VoiceVoxSpeaker, VoiceVoxSpeakerInfo, VoiceVoxSpeakerStyleInfo};
use crate::utils::http;

const OS: &str = "windows";

const VERSION: &str = "0.14.3";
const DEVICE_CPU: &str = "cpu";

const CPU_ARCH_X86: &str = "x64";
const CPU_ARCH_X64: &str = "x64";
const CPU_ARCH_ARM64: &str = "arm64";

const DATA_DIR: &str = "voicevox";
const CORE_DIR: &str = "voicevox_core";

const OUTPUT: &str = "cmd.output";
const OUTPUT_ERR: &str = "cmd.err";
const DONWLOADER_EXE: &str = "download.exe";
const DOWNLOADER_URL: &str =
    "https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download-windows-x64.exe";

lazy_static! {
    static ref BIN_MANGER: Arc<AsyncRwLock<BinaryManager>> = Arc::new(AsyncRwLock::new(BinaryManager::new(None, None)));
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VoicevoxInitializeOptions {
    pub acceleration_mode: i32,
    pub cpu_num_threads: u16,
    pub load_all_models: bool,
    pub open_jtalk_dict_dir: *const std::os::raw::c_char,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VoicevoxTtsOptions {
    pub kana: bool,
    pub enable_interrogative_upspeak: bool,
}

struct BinaryOption {
    version: String,
    device: String,
    cpu_arch: Option<String>,
}

fn check_cpu_arch() -> String {
    match std::env::consts::ARCH {
        "x86" => {
            String::from(CPU_ARCH_X86)
        }
        "x86_64" => {
            String::from(CPU_ARCH_X64)
        }
        "aarch64" => {
            String::from(CPU_ARCH_ARM64)
        }
        others => {
            log::warn!("couldn't select arch: {}, use default value: x86", others);
            String::from(CPU_ARCH_X86)
        }
    }
}

fn create_folder_if_not_exists<P: AsRef<Path>>(path: P) -> Result<(), ProgramError> {
    std::fs::create_dir_all(path).map_err(|e| ProgramError::from(e))
}

// check voicevox root data dir, create it if not exists
fn check_data_dir() -> Result<(), ProgramError> {
    create_folder_if_not_exists(get_data_path())
}

// check download exe file, download if not exists
async fn check_downloader() -> Result<(), ProgramError> {
    let exe = get_data_path().join(DONWLOADER_EXE);
    if !exe.exists() || !exe.is_file() {
        log::debug!("Downloader not found, download exe");
        http::download(String::from(DOWNLOADER_URL), exe).await?;
        log::debug!("Download downloader exe success");
    } else {
        log::debug!("Downloader exists, skip download");
    }
    Ok(())
}

#[cfg(target_os = "windows")]
unsafe fn get_last_error() -> String {
    let error_code = GetLastError();
    let mut buffer: [u16; 1024] = [0; 1024];
    let chars_written = FormatMessageW(
        winapi::um::winbase::FORMAT_MESSAGE_FROM_SYSTEM,
        std::ptr::null(),
        error_code,
        0,
        buffer.as_mut_ptr(),
        buffer.len() as DWORD,
        std::ptr::null_mut(),
    );
    String::from_utf16_lossy(&buffer[..chars_written as usize]).trim_end().to_owned()
}

fn get_data_path() -> PathBuf {
    PathBuf::from(DATA_DIR)
}

fn release_cookie(cookie: libloaderapi::DLL_DIRECTORY_COOKIE) {
    unsafe {
        libloaderapi::RemoveDllDirectory(cookie);
    }
    log::debug!("Voicevox lib cookie dropped");
}

struct LoadedLib {
    lib: Library,
    handle: Mutex<winapi::shared::minwindef::HMODULE>,
    cookie: Mutex<libloaderapi::DLL_DIRECTORY_COOKIE>,
}

struct BinaryManager {
    option: BinaryOption,
    downloading: bool,
    stop_downloading_sig: Arc<AtomicBool>,
    initialized: bool,
    lib: Option<LoadedLib>,
}

unsafe impl Send for BinaryManager {}

unsafe impl Sync for BinaryManager {}

impl BinaryManager {
    fn new(device: Option<String>, arch: Option<String>) -> Self {
        // if device is not given, use cpu as default
        let use_device;
        if let Some(dev) = device {
            use_device = dev;
        } else {
            use_device = DEVICE_CPU.to_string();
        }
        let cpu_arch;
        if use_device == DEVICE_CPU {
            // if select cpu, and arch is not given, check automatically
            if let Some(arch) = arch {
                cpu_arch = Some(arch)
            } else {
                cpu_arch = Some(check_cpu_arch())
            }
        } else {
            cpu_arch = None;
        }
        BinaryManager {
            option: BinaryOption {
                version: VERSION.to_string(),
                device: use_device,
                cpu_arch,
            },
            initialized: false,
            downloading: false,
            stop_downloading_sig: Arc::new(AtomicBool::new(false)),
            lib: None,
        }
    }

    async fn set_option(&mut self, device: String, arch: Option<String>) -> Result<(), ProgramError> {
        let mut cpu_arch = None;
        if device == DEVICE_CPU {
            if let Some(arch) = arch {
                cpu_arch = Some(arch)
            } else {
                cpu_arch = Some(check_cpu_arch())
            }
        }
        let is_changed = device != self.option.device || cpu_arch != self.option.cpu_arch;
        self.option.device = device.clone();
        self.option.cpu_arch = cpu_arch;

        if !self.initialized {
            return self.initialize().await;
        } else {
            if is_changed {
                return self.reinitialize().await;
            }
        }
        Ok(())
    }

    fn get_core_path(&self) -> PathBuf {
        return get_data_path()
            .join(format!("{}_{}", CORE_DIR, self.option.device.clone()));
    }

    fn stop_downloading(&self) {
        self.stop_downloading_sig.store(true, Ordering::Release);
    }

    // check voicevox core binary files, download it if not been downloaded
    async fn check_binary(&mut self) -> Result<(), ProgramError> {
        // check root data path
        check_data_dir()?;

        // check downloader
        check_downloader().await?;

        // check binary
        let core_path = self.get_core_path();
        log::debug!("Check voicevox core lib at {}", core_path.clone().to_str().unwrap());
        if core_path.exists() && core_path.is_dir() {
            log::debug!("Voicevox binary found, skip download");
            return Ok(());
        }

        // set downloading flag
        self.downloading = true;

        // download binary
        let data_path = get_data_path();
        let output_path = data_path.join(OUTPUT);
        let exe = data_path.join(DONWLOADER_EXE);
        let exe = exe.to_str().ok_or("unable to parse downloader exe path to str")?;
        let output_to = std::fs::canonicalize(core_path)?;
        let output_to = output_to.to_str()
            .ok_or("unable to parse output to path to str")?;

        // wrap cmd
        let cmd;
        if self.option.device == DEVICE_CPU {
            let arch = self.option.cpu_arch.clone()
                .ok_or("no cpu arch set")?;
            cmd = format!(
                "{} -v {} -o {} --os {} --device {} --cpu-arch {}",
                exe,
                self.option.version.clone(),
                output_to,
                OS,
                self.option.device.clone(),
                arch
            );
        } else {
            cmd = format!(
                "{} -v {} -o {} --os {} --device {}",
                exe,
                self.option.version.clone(),
                output_to,
                OS,
                self.option.device.clone(),
            );
        }

        log::debug!("Execute voicevox binary download with cmd: {}", cmd.clone());

        let output = File::create(output_path.clone())?;
        let out = std::process::Stdio::from(output);
        let err_output = data_path.join(OUTPUT_ERR);
        let err_output = File::create(err_output)?;
        let err_out = std::process::Stdio::from(err_output);

        let execution = Command::new("cmd")
            .args(["/C", cmd.as_str()])
            .stdout(out)
            .stderr(err_out)
            .spawn()?;
        let execution_mutex = Arc::new(AsyncMutex::new(execution));
        let execution_clone = execution_mutex.clone();
        self.stop_downloading_sig.store(false, Ordering::Release);
        let stop_downloading_sig = self.stop_downloading_sig.clone();
        let stop_signal_handle = tauri::async_runtime::spawn(async move {
            let mut execution = execution_clone.lock().await;
            while !stop_downloading_sig.load(Ordering::Acquire) {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            // if get stop signal, abort execution
            match execution.kill() {
                Ok(_) => {
                    log::debug!("Download stopped");
                }
                Err(err) => {
                    log::debug!("Download stop failed, err: {}", err);
                }
            }
        });
        let execution = execution_mutex.clone();
        let mut execution = execution.lock().await;
        let result = execution.wait()?;

        // if download finished, stop the stop signal thread
        stop_signal_handle.abort();
        let _ = tauri::async_runtime::block_on(stop_signal_handle);

        if !result.success() {
            let exit_code = result
                .code()
                .ok_or(ProgramError::from("unable to parse exit code"))?;
            return Err(ProgramError::from(
                format!("spawn download.exe failed, exit code {}, check full error at {}",
                        exit_code,
                        output_path.clone().to_str().unwrap())));
        }
        Ok(())
    }

    async fn check_and_download_binary(&mut self) -> Result<(), ProgramError> {
        // wrap check binary function, set downloading flag false after finished(error or not)
        match self.check_binary().await {
            Ok(_) => {
                self.downloading = false;
                Ok(())
            }
            Err(err) => {
                self.downloading = false;
                Err(err)
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn load_libs(&mut self) -> Result<(), ProgramError> {
        let core_path = self.get_core_path();
        let base_abs = std::fs::canonicalize(core_path.clone())?;
        let base_os = base_abs.as_os_str().encode_wide().chain(Some(0)).collect::<Vec<u16>>();
        let os_string_ptr = base_os.as_ptr();

        let cookie: libloaderapi::DLL_DIRECTORY_COOKIE = unsafe {
            libloaderapi::AddDllDirectory(os_string_ptr)
        };
        if cookie.is_null() {
            let last_err = unsafe { get_last_error() };
            return Err(ProgramError::from(
                format!("Failed to add directory to search path: {}", last_err)));
        }

        let dll = core_path.join("voicevox_core.dll");
        unsafe {
            let base_abs = std::fs::canonicalize(dll)?;
            let wide_filename: Vec<u16> = base_abs.as_os_str().encode_wide().chain(Some(0)).collect();
            let handle = libloaderapi::LoadLibraryExW(
                wide_filename.as_ptr(), std::ptr::null_mut(),
                libloaderapi::LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR |
                    libloaderapi::LOAD_LIBRARY_SEARCH_APPLICATION_DIR |
                    libloaderapi::LOAD_LIBRARY_SEARCH_USER_DIRS |
                    libloaderapi::LOAD_LIBRARY_SEARCH_SYSTEM32 |
                    libloaderapi::LOAD_LIBRARY_SEARCH_DEFAULT_DIRS);
            if handle.is_null() {
                let last_err = get_last_error();

                // unload cookie if load dll failed
                release_cookie(cookie);

                return Err(ProgramError::from(
                    format!("Failed to load core: {}", last_err)));
            }
            self.lib = Some(LoadedLib {
                lib: Library::from(libloading::os::windows::Library::from_raw(handle)),
                handle: Mutex::new(handle),
                cookie: Mutex::new(cookie),
            });
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn unload_lib(&mut self) {
        if let Some(lib) = self.lib.take() {
            let res = lib.lib.close();
            if res.is_err() {
                log::error!("Failed to close voicevox lib, error: {}", res.unwrap_err());
            } else {
                log::debug!("Close voicevox lib success");
            }

            let ptr = lib.cookie.into_inner().unwrap();
            unsafe {
                libloaderapi::RemoveDllDirectory(ptr);
            }
            log::debug!("Voicevox lib cookie dropped");
        }
    }

    #[cfg(target_os = "windows")]
    async fn initialize(&mut self) -> Result<(), ProgramError> {
        self.check_and_download_binary().await?;
        self.load_libs()?;
        match self.voicevox_initialize() {
            Ok(_) => {
                self.initialized = true;
                Ok(())
            }
            Err(err) => {
                // if initialize failed, unload lib as well
                self.unload_lib();
                Err(err)
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn voicevox_initialize(&mut self) -> Result<(), ProgramError> {
        let lib = self.lib.as_ref().unwrap();
        let initialize: Symbol<unsafe extern "C" fn(*const VoicevoxInitializeOptions) -> i32> = unsafe {
            lib.lib.get(b"voicevox_initialize\0")?
        };

        let jtalk_dic = self.get_core_path()
            .join("open_jtalk_dic_utf_8-1.11");
        let jtalk_dic_str = jtalk_dic.to_str()
            .ok_or("unable to parse jtalk dict path to str")?;
        let open_jtalk_dict_dir = CString::new(jtalk_dic_str.as_bytes())
            .map_err(|_| "unable to parse string to cstring")?;

        let opts = &VoicevoxInitializeOptions {
            acceleration_mode: 0,
            cpu_num_threads: 4,
            load_all_models: false,
            open_jtalk_dict_dir: open_jtalk_dict_dir.as_ptr(),
        };

        let res = unsafe { initialize(opts as *const VoicevoxInitializeOptions) };

        if res != 0 {
            let last_err = unsafe { get_last_error() };

            return Err(ProgramError::from(
                format!("execute voicevox_initialize failed, code: {} error: {}", res, last_err)));
        }

        Ok(())
    }

    async fn reinitialize(&mut self) -> Result<(), ProgramError> {
        self.finalize();
        self.unload_lib();
        self.initialize().await
    }

    #[cfg(target_os = "windows")]
    fn do_finalize(&mut self) -> Result<(), ProgramError> {
        let lib = self.lib.as_ref().unwrap();
        let finalize: Symbol<unsafe extern "C" fn()> = unsafe {
            lib.lib.get(b"voicevox_finalize\0")?
        };
        unsafe {
            finalize();
        }
        self.initialized = false;
        Ok(())
    }

    fn finalize(&mut self) {
        if self.initialized {
            #[cfg(target_os = "windows")]
            match self.do_finalize() {
                Ok(_) => {
                    log::debug!("Finalize voicevox success");
                }
                Err(err) => {
                    log::debug!("Finalize voicevox failed, error: {}", err);
                }
            }
        }
    }

    fn assert_initialized(&self) -> Result<(), ProgramError> {
        if !self.initialized {
            return Err(ProgramError::from("core lib is not initialized"));
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn get_metas_json(&self) -> Result<Vec<VoiceVoxSpeaker>, ProgramError> {
        self.assert_initialized()?;
        let lib = self.lib.as_ref().unwrap();
        let get_metas_json: Symbol<unsafe extern "C" fn() -> *const ::std::os::raw::c_char> = unsafe {
            lib.lib.get(b"voicevox_get_metas_json\0")?
        };
        let result = unsafe {
            let metas = get_metas_json();
            CStr::from_ptr(metas).to_string_lossy().to_string()
        };

        let json_parsed = serde_json::from_str::<Vec<VoiceVoxSpeaker>>(result.as_str()).unwrap();
        Ok(json_parsed)
    }

    #[cfg(target_os = "windows")]
    fn load_model(&self, speaker_id: u32) -> Result<(), ProgramError> {
        self.assert_initialized()?;
        let lib = self.lib.as_ref().unwrap();
        let load_model: Symbol<unsafe extern "C" fn(u32) -> u32> = unsafe {
            lib.lib.get(b"voicevox_load_model\0")?
        };
        let result = unsafe {
            load_model(speaker_id)
        };
        if result != 0 {
            let last_err = unsafe { get_last_error() };
            return Err(ProgramError::from(
                format!("execute voicevox_load_model failed, code: {} error: {}",
                        result, last_err)));
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn is_model_loaded(&self, speaker_id: u32) -> Result<bool, ProgramError> {
        self.assert_initialized()?;
        let lib = self.lib.as_ref().unwrap();
        let is_model_loaded: Symbol<unsafe extern "C" fn(u32) -> bool> = unsafe {
            lib.lib.get(b"voicevox_is_model_loaded\0")?
        };
        let result = unsafe {
            is_model_loaded(speaker_id)
        };
        Ok(result)
    }

    // voicevox_tts
    #[cfg(target_os = "windows")]
    fn tts(&self, text: String, speaker_id: u32) -> Result<Bytes, ProgramError> {
        self.assert_initialized()?;

        let lib = self.lib.as_ref().unwrap();
        let tts: Symbol<unsafe extern "C" fn(
            *const std::os::raw::c_char,
            u32,
            *const VoicevoxTtsOptions,
            *mut usize,
            *mut *mut u8,
        ) -> i32> = unsafe {
            lib.lib.get(b"voicevox_tts\0")?
        };
        let opts = &VoicevoxTtsOptions {
            kana: true,
            enable_interrogative_upspeak: true,
        };
        let txt_ptr = CString::new(text.as_bytes())
            .map_err(|_| "unable to parse text to cstring")?;
        let txt_ptr = txt_ptr.as_ptr();
        let mut output_wav_length: usize = 0;
        let output_wav_length_ptr: *mut usize = (&mut output_wav_length) as *mut usize;
        let mut output_wav: *mut u8 = std::ptr::null_mut();
        let output_wav_ptr: *mut *mut u8 = (&mut output_wav) as *mut *mut u8;
        let result = unsafe {
            tts(txt_ptr, speaker_id,
                opts as *const VoicevoxTtsOptions,
                output_wav_length_ptr,
                output_wav_ptr)
        };
        if result != 0 {
            let last_err = unsafe { get_last_error() };
            return Err(ProgramError::from(
                format!("execute voicevox_tts failed, code: {} error: {}",
                        result, last_err)));
        }
        let slice = unsafe {
            slice::from_raw_parts(output_wav, output_wav_length)
        };
        Ok(Bytes::from(slice))
    }
}

impl Drop for BinaryManager {
    fn drop(&mut self) {
        self.finalize();
        #[cfg(target_os = "windows")]
        self.unload_lib();
    }
}

pub async fn speakers() -> Result<Vec<VoiceVoxSpeaker>, ProgramError> {
    let lock = BIN_MANGER.clone();
    let man = lock.read().await;
    man.get_metas_json()
}

pub async fn tts(text: String, speaker_id: u32) -> Result<Bytes, ProgramError> {
    let lock = BIN_MANGER.clone();
    let man = lock.write().await;
    // check model is loaded
    if !man.is_model_loaded(speaker_id)? {
        man.load_model(speaker_id)?;
    }
    man.tts(text, speaker_id)
}

pub async fn speaker_info(speaker_uuid: String) -> Result<VoiceVoxSpeakerInfo, ProgramError> {
    let speakers = speakers().await?;
    let mut speaker = None;
    for _speaker in speakers {
        if _speaker.speaker_uuid == speaker_uuid {
            speaker = Some(_speaker);
            break;
        }
    }
    if speaker.is_none() {
        return Err(ProgramError::from("No speaker found with given uuid"));
    }
    let speaker = speaker.take().unwrap();

    let speaker_info_root = get_data_path().join("speaker_info");
    if !speaker_info_root.exists() {
        // TODO: maybe try to download from somewhere?
        return Err(ProgramError::from("No speaker_info folder found"));
    }
    let speaker_info = speaker_info_root.join(speaker_uuid.clone());
    if !speaker_info.exists() {
        return Err(ProgramError::from("No speaker info of given uuid found"));
    }
    let policy_file = speaker_info.join("policy.md");
    let policy = std::fs::read_to_string(policy_file)?;

    let portrait_file = speaker_info.join("portrait.png");
    let contents = std::fs::read(portrait_file)?;
    let portrait = general_purpose::STANDARD.encode(&contents);

    let mut style_infos = Vec::new();
    for style in speaker.styles {
        let icon_file = speaker_info.join("icons")
            .join(format!("{}.png", style.id));
        let contents = std::fs::read(icon_file)?;
        let icon = general_purpose::STANDARD.encode(&contents);

        let portrait_file = speaker_info.join("portraits")
            .join(format!("{}.png", style.id));
        let portrait = if portrait_file.exists() {
            let contents = std::fs::read(portrait_file)?;
            let portrait = general_purpose::STANDARD.encode(&contents);
            Some(portrait)
        } else {
            None
        };

        let samples = speaker_info.join("voice_samples");
        let sample_file_prefix = format!("{}_", style.id);
        let mut voice_samples = Vec::new();
        for entry in walkdir::WalkDir::new(samples).into_iter()
            .filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.starts_with(sample_file_prefix.as_str()) {
                        let contents = std::fs::read(entry.path())?;
                        let sample = general_purpose::STANDARD.encode(&contents);
                        voice_samples.push(sample);
                    }
                }
            }
        }

        let info = VoiceVoxSpeakerStyleInfo {
            id: style.id,
            icon,
            portrait,
            voice_samples,
        };
        style_infos.push(info);
    }

    Ok(VoiceVoxSpeakerInfo {
        policy,
        portrait,
        style_infos,
    })
}

pub async fn is_downloading() -> bool {
    let lock = BIN_MANGER.clone();
    let man = lock.read().await;
    man.downloading
}

pub async fn check_and_load(config: VoiceVoxEngineConfig) -> Result<(), ProgramError> {
    let lock = BIN_MANGER.clone();
    let mut man = lock.write().await;
    man.set_option(config.device, config.cpu_arch).await
}

pub async fn check_and_unload() -> Result<(), ProgramError> {
    let lock = BIN_MANGER.clone();
    let mut man = lock.write().await;
    if man.downloading {
        man.stop_downloading();
    }
    if man.initialized {
        man.finalize();
        man.unload_lib();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init() {
        // init
        {
            let lock = BIN_MANGER.clone();
            let mut man = lock.write().await;
            man.initialize().await.unwrap();
        }
        /*// get speaker info
        let speakers = {
            let speakers = speakers().await.unwrap();
            assert!(speakers.len() > 0, "library should contains speakers");
            speakers
        };
        {
            let speaker = speakers.get(0).unwrap();
            let speaker_info = speaker_info(speaker.speaker_uuid.clone()).await.unwrap();
            println!("policy: {}", speaker_info.policy);
        }*/
        // unload
        {
            let lock = BIN_MANGER.clone();
            let mut man = lock.write().await;
            man.finalize();
            man.unload_lib();
        }
    }
}
