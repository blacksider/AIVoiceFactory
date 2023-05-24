use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;

use cpal::traits::DeviceTrait;
use lazy_static::lazy_static;
use tauri::regex::Regex;

use crate::audio::listener;
use crate::audio::listener::Listener;
use crate::common::{app, constants};
use crate::controller::{audio_manager, generator};
use crate::controller::errors::ProgramError;
use crate::controller::recognizer;

lazy_static! {
    static ref TALKING: AtomicBool = AtomicBool::new(false);
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

    let mut audio = Listener::new(30 * 1000);
    audio.init(device)?;
    audio.resume()?;

    let vad_thold = params.vad_thold;
    let freq_thold = params.freq_thold;
    let verbose = params.verbose;
    let voice_ms = params.voice_ms;

    tauri::async_runtime::spawn(async move {
        let mut is_running = true;
        let mut force_speak = false;
        let mut pcmf32_cur = Vec::<f32>::new();
        while is_running {
            is_running = TALKING.load(Ordering::Acquire);
            if !is_running {
                break;
            }

            app::silent_emit_all(constants::event::ON_RECORDING_STATE,
                                 TalkRecordingState::Recording);

            sleep(Duration::from_millis(100));

            audio.get(2000, &mut pcmf32_cur);

            let vad_ok = listener::vad_simple(&mut pcmf32_cur, sample_rate, 1250, vad_thold, freq_thold, verbose);

            // check vad
            if vad_ok || force_speak {
                app::silent_emit_all(constants::event::ON_RECORDING_STATE,
                                     TalkRecordingState::DetectSpeech);

                log::debug!("Speech detected, Processing ...");

                audio.get(voice_ms, &mut pcmf32_cur);

                let mut text_heard = String::new();
                if !force_speak {
                    let reg_res = recognizer::recognize(&pcmf32_cur, channels, sample_rate).await;
                    match reg_res {
                        Ok(text) => {
                            text_heard = String::from(text.trim());
                        }
                        Err(err) => {
                            log::error!("Failed to recognize audio, err: {}", err);
                            audio.clear();
                            continue;
                        }
                    }
                }

                log::debug!("Heard original: {}", text_heard.clone());

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

                // remove all characters except letters, numbers, punctuation and ':', '\'', '-', ' '
                // comment this part since we need other languages working as well
                /*{
                    // let re = Regex::new(r#"[^a-zA-Z0-9\\.,\\?!\\s\\:\\'\\-]"#).unwrap();
                    let re = Regex::new(r#"[^a-zA-Z0-9\\.,\\?!\s\\:\\'\\-]"#).unwrap();
                    text_heard = re.replace_all(&text_heard, "").to_string();
                }*/

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
                    continue;
                }

                force_speak = false;

                log::info!("Heard: {}", text_heard.clone());

                // emit  events
                app::silent_emit_all(constants::event::ON_AUDIO_RECOGNIZE_TEXT,
                                     text_heard.clone());
                app::silent_emit_all(constants::event::ON_RECORDING_RECOGNIZE_TEXT,
                                     text_heard.clone());

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
                    };
                }

                audio.clear();
            }
        }

        // if loop exit, pause audio listener
        match audio.pause() {
            Ok(_) => {}
            Err(err) => {
                log::error!("Failed to pause listener, err: {}", err);
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
    TALKING.store(false, Ordering::Release);
    Ok(())
}