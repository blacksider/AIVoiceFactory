use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;

use cpal::traits::DeviceTrait;
use lazy_static::lazy_static;
use tauri::regex::Regex;
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};

use crate::audio::listener;
use crate::common::{app, constants};
use crate::config::voice_recognition;
use crate::controller::{audio_manager, generator};
use crate::controller::errors::ProgramError;
use crate::controller::recognizer;

lazy_static! {
    static ref TALKING: AtomicBool = AtomicBool::new(false);
    static ref TALKING_STOP_SIG: (Sender<()>, Receiver<()>) = broadcast::channel(1);
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
enum TalkRecordingState {
    Recording,
    DetectSpeech,
}

pub struct TalkParams {
    pub vad_thold: f32,
    pub freq_thold: f32,
    pub voice_ms: u32,
    pub verbose: bool,
}

impl TalkParams {
    pub fn default() -> Self {
        TalkParams {
            vad_thold: 0.6,
            freq_thold: 100.0,
            voice_ms: 10000,
            verbose: false,
        }
    }
}

async fn start_talk_process(params: &TalkParams) -> Result<(), ProgramError> {
    let device = audio_manager::get_input_device().await?;
    let config = device.default_input_config()?;
    let sample_rate = config.sample_rate().0;
    let channels = config.channels();

    let mut audio = listener::LISTENER.lock().await;
    audio.init(device)?;

    let ok = audio.resume()?;
    if !ok {
        return Ok(());
    }

    let vad_thold = params.vad_thold;
    let freq_thold = params.freq_thold;
    let verbose = params.verbose;
    let voice_ms = params.voice_ms;

    tauri::async_runtime::spawn(async move {
        let (interrupted_tx, _) = &*TALKING_STOP_SIG;
        let mut interrupted_rx = interrupted_tx.subscribe();

        let mut force_speak = false;
        let mut pcmf32_cur = Vec::<f32>::new();
        let mut is_running = true;
        loop {
            if !is_running {
                break;
            }

            tokio::select! {
                response = async {
                    app::silent_emit_all(constants::event::ON_RECORDING_STATE,
                                         TalkRecordingState::Recording);
                    sleep(Duration::from_millis(100));
                    audio.get(2000, &mut pcmf32_cur);

                    let vad_ok = listener::vad_simple(&mut pcmf32_cur, sample_rate,
                        1250, vad_thold, freq_thold, verbose);

                    // check vad
                    if vad_ok || force_speak {
                        app::silent_emit_all(constants::event::ON_RECORDING_STATE,
                                             TalkRecordingState::DetectSpeech);

                        log::debug!("Speech detected, ready to recognize");

                        audio.get(voice_ms, &mut pcmf32_cur);

                        let mut text_heard = String::new();
                        if !force_speak {
                            let reg_res = recognizer::recognize(
                                &pcmf32_cur, channels, sample_rate).await;
                            match reg_res {
                                Ok(text) => {
                                    text_heard = String::from(text.trim());
                                }
                                Err(err) => {
                                    log::error!("Failed to recognize audio, err: {}", err);
                                    audio.clear();
                                    return Ok(());
                                }
                            }
                        }

                        // remove text between brackets []
                        {
                            let re = Regex::new(r"\[.*?\]").unwrap();
                            text_heard = re.replace_all(&*text_heard, "").to_string();
                        }
                        // remove text between brackets ()
                        {
                            let re = Regex::new(r"\(.*?\)").unwrap();
                            text_heard = re.replace_all(&*text_heard, "").to_string();
                        }

                        // take first line
                        {
                            let re = Regex::new(r"\n").unwrap();
                            let index = re.find(&text_heard).map(|m| m.start()).unwrap_or_else(|| text_heard.len());
                            text_heard.truncate(index);
                        }

                        // remove leading and trailing whitespace
                        {
                            let re = Regex::new(r"^\s+|\s+$").unwrap();
                            text_heard = re.replace_all(&text_heard, "").to_string();
                        }

                        if text_heard.is_empty() || force_speak {
                            audio.clear();
                            return Ok(());
                        }

                        force_speak = false;

                        log::info!("Recognized: {}", text_heard.clone());

                        // emit  events
                        app::silent_emit_all(constants::event::ON_AUDIO_RECOGNIZE_TEXT,
                                             text_heard.clone());
                        app::silent_emit_all(constants::event::ON_RECORDING_RECOGNIZE_TEXT,
                                             text_heard.clone());

                        let gen_audio = {
                            let manager =
                                voice_recognition::VOICE_REC_CONFIG_MANAGER.read().await;
                            manager.get_config().generate_after
                        };
                        if gen_audio {
                            // generate audio by text
                            let index = generator::generate_audio(text_heard).await;
                            if let Some(cache) = index {
                                // play generated audio
                                match generator::PLAY_AUDIO_CHANNEL.send(cache.name.clone()).await {
                                    Ok(_) => {
                                        log::debug!("Send generated audio to play channel success");
                                    }
                                    Err(err) => {
                                        log::error!("Failed to send generated audio to play channel, err: {}", err);
                                    }
                                }
                            }
                        }

                        audio.clear();
                    }
                    Ok::<_, ProgramError>(())
                } => {
                    match response {
                        Ok(_) => {
                            // loop success then do nothing, enter next loop
                        }
                        Err(err) => {
                            log::error!("Listener handle loop with err: {}", err);
                        }
                    }
                }
                _ = interrupted_rx.recv() => {
                    log::debug!("Listener interrupted");
                    // if loop interrupted, pause audio listener
                    match audio.pause() {
                        Ok(_) => {}
                        Err(err) => {
                            log::error!("Failed to pause listener, err: {}", err);
                        }
                    }
                    // set flag to false to stop loop
                    is_running = false;
                }
            }
        }
    });

    Ok(())
}

pub async fn start(params: &TalkParams) -> Result<(), ProgramError> {
    if TALKING.load(Ordering::Acquire) {
        return Err(ProgramError::from("Talk process already running"));
    }

    TALKING.store(true, Ordering::Release);

    let process = start_talk_process(params).await;
    match process {
        Ok(_) => {
            log::debug!("Talk process started");
            Ok(())
        }
        Err(err) => {
            TALKING.store(false, Ordering::Release);
            Err(err)
        }
    }
}

pub fn stop() -> Result<(), ProgramError> {
    if !TALKING.load(Ordering::Acquire) {
        return Err(ProgramError::from("Talk process already stopped"));
    }
    let (interrupted_tx, _) = &*TALKING_STOP_SIG;
    match interrupted_tx.send(()) {
        Ok(_) => {
            TALKING.store(false, Ordering::Release);
        }
        Err(err) => {
            log::error!("Failed to send talk process stop signal, error: {}", err);
        }
    }
    Ok(())
}