#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]

use std::ffi::{CStr, CString};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use lazy_static::lazy_static;
use tauri::regex;
use tauri::regex::Regex;
use tokio::sync::{broadcast, Mutex};
use tokio::sync::broadcast::{Receiver, Sender};

use crate::config::voice_recognition;
use crate::config::voice_recognition::WhisperConfigType;
use crate::controller::errors::ProgramError;
use crate::utils;
use crate::utils::http;

const MODEL_PATH: &str = "whisper/models";
const MODEL_TMP_FILE: &str = "model.tmp";

const DLL_FILE: &str = "whisper/whisper.dll";
const DLL_DOWNLOAD_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/";

lazy_static! {
    static ref WHISPER_LIB: Arc<Mutex<WhisperLibrary>> = Arc::new(Mutex::new(WhisperLibrary::new()));
    static ref MODEL_LOAD_STOP_SIG: (Sender<()>, Receiver<()>) = broadcast::channel(1);
    static ref MODEL_AVAILABLE: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

// declaration of whisper structs and constants
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct whisper_context {
    _unused: [u8; 0],
}

pub type whisper_sampling_strategy = std::os::raw::c_int;
pub type whisper_token = std::os::raw::c_int;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct whisper_full_params__bindgen_ty_1 {
    pub best_of: std::os::raw::c_int,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct whisper_full_params__bindgen_ty_2 {
    pub beam_size: std::os::raw::c_int,
    pub patience: f32,
}

pub type whisper_new_segment_callback = Option<
    unsafe extern "C" fn(
        ctx: *mut whisper_context,
        n_new: std::os::raw::c_int,
        user_data: *mut std::os::raw::c_void,
    ),
>;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct whisper_token_data {
    pub id: whisper_token,
    pub tid: whisper_token,
    pub p: f32,
    pub plog: f32,
    pub pt: f32,
    pub ptsum: f32,
    pub t0: i64,
    pub t1: i64,
    pub vlen: f32,
}

pub type whisper_encoder_begin_callback = Option<
    unsafe extern "C" fn(ctx: *mut whisper_context, user_data: *mut std::os::raw::c_void) -> bool,
>;
pub type whisper_logits_filter_callback = Option<
    unsafe extern "C" fn(
        ctx: *mut whisper_context,
        tokens: *const whisper_token_data,
        n_tokens: std::os::raw::c_int,
        logits: *mut f32,
        user_data: *mut std::os::raw::c_void,
    ),
>;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct whisper_full_params {
    pub strategy: whisper_sampling_strategy,
    pub n_threads: std::os::raw::c_int,
    pub n_max_text_ctx: std::os::raw::c_int,
    pub offset_ms: std::os::raw::c_int,
    pub duration_ms: std::os::raw::c_int,
    pub translate: bool,
    pub no_context: bool,
    pub single_segment: bool,
    pub print_special: bool,
    pub print_progress: bool,
    pub print_realtime: bool,
    pub print_timestamps: bool,
    pub token_timestamps: bool,
    pub thold_pt: f32,
    pub thold_ptsum: f32,
    pub max_len: std::os::raw::c_int,
    pub split_on_word: bool,
    pub max_tokens: std::os::raw::c_int,
    pub speed_up: bool,
    pub audio_ctx: std::os::raw::c_int,
    pub prompt_tokens: *const whisper_token,
    pub prompt_n_tokens: std::os::raw::c_int,
    pub language: *const std::os::raw::c_char,
    pub suppress_blank: bool,
    pub suppress_non_speech_tokens: bool,
    pub temperature: f32,
    pub max_initial_ts: f32,
    pub length_penalty: f32,
    pub temperature_inc: f32,
    pub entropy_thold: f32,
    pub logprob_thold: f32,
    pub no_speech_thold: f32,
    pub greedy: whisper_full_params__bindgen_ty_1,
    pub beam_search: whisper_full_params__bindgen_ty_2,
    pub new_segment_callback: whisper_new_segment_callback,
    pub new_segment_callback_user_data: *mut std::os::raw::c_void,
    pub encoder_begin_callback: whisper_encoder_begin_callback,
    pub encoder_begin_callback_user_data: *mut std::os::raw::c_void,
    pub logits_filter_callback: whisper_logits_filter_callback,
    pub logits_filter_callback_user_data: *mut std::os::raw::c_void,
}

pub const whisper_sampling_strategy_WHISPER_SAMPLING_GREEDY: whisper_sampling_strategy = 0;
pub const whisper_sampling_strategy_WHISPER_SAMPLING_BEAM_SEARCH: whisper_sampling_strategy = 1;
// ^^^^^^ declaration of whisper structs and constants end

/// whisper library wrapper
pub struct WhisperLibrary {
    inner: libloading::Library,
    context: Option<*mut whisper_context>,
}

unsafe impl Send for WhisperLibrary {}

unsafe impl Sync for WhisperLibrary {}

impl WhisperLibrary {
    fn new() -> Self {
        let file = PathBuf::from(DLL_FILE);
        let lib = unsafe {
            libloading::Library::new(file).expect("Unable to load whisper.dll")
        };
        WhisperLibrary {
            inner: lib,
            context: None,
        }
    }

    fn get_context(&self) -> Result<*mut whisper_context, ProgramError> {
        Ok(self.context.ok_or("Context is not initialized")?.clone())
    }

    fn whisper_init_from_file(&mut self, file: String) -> Result<(), ProgramError> {
        // if context exits, free it
        if self.context.is_some() {
            self.whisper_free()?;
        }
        let init_from_file: libloading::Symbol<unsafe extern "C" fn(*const std::os::raw::c_char) -> *mut whisper_context> = unsafe {
            self.inner.get(b"whisper_init_from_file\0")?
        };
        let model_ptr = CString::new(file.as_bytes())
            .map_err(|_| {
                ProgramError::from("unable to convert String to CString")
            })?;
        let context = unsafe {
            init_from_file(model_ptr.as_ptr())
        };
        self.context.replace(context);
        MODEL_AVAILABLE.store(true, Ordering::Release);
        drop(model_ptr);
        Ok(())
    }

    fn whisper_full_default_params(&self) -> Result<whisper_full_params, ProgramError> {
        let full_default_params: libloading::Symbol<unsafe extern "C" fn(whisper_sampling_strategy) -> whisper_full_params> = unsafe {
            self.inner.get(b"whisper_full_default_params\0")?
        };
        let params = unsafe {
            full_default_params(whisper_sampling_strategy_WHISPER_SAMPLING_GREEDY)
        };
        Ok(params)
    }

    fn whisper_full(&self, wav: &Vec<f32>,
                    params: whisper_full_params) -> Result<std::os::raw::c_int, ProgramError> {
        let whisper_full: libloading::Symbol<unsafe extern "C" fn(
            *mut whisper_context,
            whisper_full_params,
            *const f32,
            std::os::raw::c_int,
        ) -> std::os::raw::c_int> = unsafe {
            self.inner.get(b"whisper_full\0")?
        };

        let result = unsafe {
            whisper_full(self.get_context()?,
                         params,
                         wav.as_ptr(),
                         wav.len() as std::os::raw::c_int)
        };

        Ok(result)
    }

    fn whisper_full_n_segments(&self) -> Result<std::os::raw::c_int, ProgramError> {
        let full_n_segments: libloading::Symbol<unsafe extern "C" fn(
            *mut whisper_context
        ) -> std::os::raw::c_int> = unsafe {
            self.inner.get(b"whisper_full_n_segments\0").unwrap()
        };
        let n_segments = unsafe {
            full_n_segments(self.get_context()?)
        };
        Ok(n_segments)
    }

    fn whisper_full_get_segment_text(&self, seg: std::os::raw::c_int) -> Result<String, ProgramError> {
        let full_get_segment_text: libloading::Symbol<unsafe extern "C" fn(
            *mut whisper_context,
            std::os::raw::c_int,
        ) -> *const std::os::raw::c_char> = unsafe {
            self.inner.get(b"whisper_full_get_segment_text\0").unwrap()
        };
        let text = unsafe {
            full_get_segment_text(self.get_context()?, seg)
        };
        let text = unsafe {
            CStr::from_ptr(text).to_string_lossy().to_string()
        };
        Ok(text)
    }

    fn whisper_free(&mut self) -> Result<(), ProgramError> {
        if self.context.is_none() {
            return Ok(());
        }
        let free: libloading::Symbol<unsafe extern "C" fn(*mut whisper_context)> = unsafe {
            self.inner.get(b"whisper_free\0").unwrap()
        };
        unsafe {
            free(self.get_context()?);
        }
        MODEL_AVAILABLE.store(false, Ordering::Release);
        let context = self.context.take()
            .ok_or("Cannot set context to none")?;
        unsafe {
            std::ptr::drop_in_place(context);
        }
        log::debug!("Free current whisper model success");
        Ok(())
    }
}

/// try to check and init whisper library if whisper recognition is enabled;
/// this method should be called at app startup.
pub async fn init_library() {
    let lock = WHISPER_LIB.clone();
    let _ = lock.lock().await;

    let config = {
        let manager =
            voice_recognition::VOICE_REC_CONFIG_MANAGER.read().await;
        manager.get_config()
    };
    if !config.enable {
        return;
    }
    let (rec, by_whisper) = config.recognize_by_whisper();
    if !(rec && by_whisper.is_some()) {
        return;
    }
    let config = by_whisper.unwrap();
    if config.config_type != WhisperConfigType::Binary {
        return;
    }

    // spawn a new thread to load model
    let model = config.use_model.clone();
    tauri::async_runtime::spawn(async move {
        match load_model(model.clone()).await {
            Ok(_) => {}
            Err(err) => {
                log::error!("Load model {} failed with error: {}", model, err);
            }
        }
    });
}

/// load whisper with given model name, note that model name pattern is "ggml-\[name].bin",
/// the param is the part "\[name]",
/// for example: to load model "ggml-base.bin", pass param: "base"
pub async fn load_model(model: String) -> Result<(), ProgramError> {
    log::debug!("Load whisper model: {}", model.clone());
    // lock download file by lib lock
    let lock = WHISPER_LIB.clone();
    let mut lib = lock.lock().await;

    let (interrupted_tx, _) = &*MODEL_LOAD_STOP_SIG;
    let mut interrupted_rx = interrupted_tx.subscribe();

    let model_name = format!("ggml-{}.bin", model);
    log::debug!("Loading whisper model {}", model_name.clone());

    let model_path = PathBuf::from(MODEL_PATH);
    std::fs::create_dir_all(model_path.clone())?;

    let download_tmp_file = model_path.join(MODEL_TMP_FILE);
    let model_file = model_path.join(model_name.clone());

    tokio::select! {
        response = async {
            if !model_file.is_file() {
                // TODO: try to compare model sha256 to make sure file is ok
                let download_url = DLL_DOWNLOAD_URL.to_owned() + &*model_name.clone();
                // download if model not downloaded
                log::debug!("Downloading whisper model {} from path [{}], download to {}",
                    model_name.clone(),
                    download_url.clone(),
                    download_tmp_file.clone().to_str().unwrap());

                http::download(download_url, download_tmp_file.clone()).await?;
                log::debug!("Download model file {} success", model_name.clone());
                // rename tmp file to actual model file
                std::fs::rename(download_tmp_file.clone(), model_file.clone())?;
            }

            // try to load model
            let model_file = model_file.to_str()
                .ok_or(ProgramError::from("Unable to parse model file path to str"))?
                .to_string();
            lib.whisper_init_from_file(model_file)?;
            Ok::<_, ProgramError>(())
        } => {
            response?;
            utils::silent_remove_file(download_tmp_file);
            log::debug!("Load model {} success", model_name);
        }
        _ = interrupted_rx.recv() => {
            utils::silent_remove_file(download_tmp_file);
            log::debug!("Manually whisper model downloading, stop at downloading model file");
        }
    }

    Ok(())
}

/// free whisper model
pub async fn free_model() -> Result<(), ProgramError> {
    let (interrupted_tx, _) = &*MODEL_LOAD_STOP_SIG;
    match interrupted_tx.send(()) {
        Ok(_) => {}
        Err(err) => {
            log::error!("Failed to send model load stop signal, error: {}", err);
        }
    }

    // lock method by lib lock
    // this will wait for load_model to be finished since we have set stop signal
    let lock = WHISPER_LIB.clone();
    let mut lib = lock.lock().await;
    // try to free first
    lib.whisper_free()
}

/// update whisper model if choosing another model
pub async fn update_model(model: String) -> Result<(), ProgramError> {
    free_model().await?;

    // spawn a new thread to load model
    tauri::async_runtime::spawn(async move {
        match load_model(model.clone()).await {
            Ok(_) => {}
            Err(err) => {
                log::error!("Update model {} failed with error: {}", model, err);
            }
        }
    });

    Ok(())
}

/// a united function to do transcribe of whisper,
/// all transcribe params are set by default optimized values,
/// all you need to offer is language and audio data.<br>
/// note data must be Vec\<f32>, mono channel, sample rate: 16000
pub async fn recognize(language: Option<String>, data: &Vec<f32>) -> Result<String, ProgramError> {
    if !MODEL_AVAILABLE.load(Ordering::Acquire) {
        return Err(ProgramError::from("Whisper model is not loaded"));
    }

    let lock = WHISPER_LIB.clone();
    let lib = lock.lock().await;

    let mut wparams = lib.whisper_full_default_params()?;
    wparams.print_realtime = false;
    wparams.print_progress = false;
    wparams.print_timestamps = false;
    wparams.print_special = false;
    wparams.no_context = true;
    wparams.single_segment = true;
    wparams.translate = false;

    let lan: CString = if let Some(language) = language {
        CString::new(language)
            .map_err(|_| "unable to parse string to cstring")?
    } else {
        CString::new("".as_bytes())
            .map_err(|_| "unable to parse string to cstring")?
    };

    wparams.language = lan.as_ptr();

    let n_threads = std::cmp::min(4, num_cpus::get());
    wparams.n_threads = n_threads as std::os::raw::c_int;

    wparams.speed_up = false;

    // the original value is 1500(corresponds to 30s audio), and the value should be multiple of 64;
    // setting it to 768 would make the Encoder evaluate about 2 times faster;
    // refer: https://github.com/ggerganov/whisper.cpp/discussions/297
    wparams.audio_ctx = 768;

    let result = lib.whisper_full(data, wparams)?;
    if result != 0 {
        return Err(ProgramError::from(format!("transcribe failed, code {}", result)));
    }

    let n_segments = lib.whisper_full_n_segments()?;
    let mut result = String::new();
    for i in 0..n_segments {
        let text = lib.whisper_full_get_segment_text(i)?;
        result.push_str(&*text);
    }

    if is_special_text(&*result) {
        log::debug!("Whisper segment text is special text {}, skip", result.clone());
        return Ok(String::new());
    }

    Ok(result)
}

/// if whisper return text like: \[Music] (Whisper), it means it is not regular content
/// of somebody speaking, aka special text
/// note: whisper lib set special_text=false should work,
/// but for somehow it is still trying to send back text like this when you give it an empty audio
fn is_special_text(text: &str) -> bool {
    let text_heard = String::from(text.trim());
    {
        let re = Regex::new(r"\(.*?\)").unwrap();
        if re.is_match(&*text_heard) {
            return true;
        }
    }

    {
        let re = Regex::new(r"\[.*?\]").unwrap();
        if re.is_match(&*text_heard) {
            return true;
        }
    }

    false
}

/// list all models in [MODEL_PATH] that is end with .bin(which is a file suffix of whisper model)
pub fn available_models() -> Result<Vec<String>, ProgramError> {
    let model_path = PathBuf::from(MODEL_PATH);
    let mut models = vec![];

    let file_name_p = regex::Regex::new(r"ggml-(\w+).bin").unwrap();

    for entry in walkdir::WalkDir::new(model_path)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let file_name = entry.file_name().to_str();
        if file_name.is_some() {
            let file_name = file_name.unwrap();
            if let Some(captures) = file_name_p.captures(file_name) {
                let name = &captures[1];
                models.push(name.to_string());
            }
        }
    }
    Ok(models)
}
