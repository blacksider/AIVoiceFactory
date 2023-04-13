// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate core;

use tauri::{App, AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, Window, Wry};

use crate::common::{app, constants};
use crate::config::voice_engine;
use crate::controller::{audio_manager, audio_recorder, generator};
use crate::controller::errors::ProgramError;
use crate::controller::voice_engine::voicevox;
use crate::controller::voice_recognition::whisper;

mod logger;
mod cypher;
mod config;
mod commands;
mod controller;
mod utils;
mod common;

fn create_system_tray() -> SystemTray {
    let quit = CustomMenuItem::new("exit".to_string(), "Exit");
    let hide = CustomMenuItem::new("close".to_string(), "Close");
    let tray_menu = SystemTrayMenu::new()
        .add_item(hide)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);
    SystemTray::new().with_menu(tray_menu)
}

pub fn get_main_window(app: &AppHandle<Wry>) -> Result<Window<Wry>, ProgramError> {
    app.get_window("main")
        .ok_or(ProgramError::from("cannot get main window"))
}

pub fn handle_dbl_click(app: &AppHandle<Wry>) -> Result<(), ProgramError> {
    let window = get_main_window(app)?;
    window.unminimize()?;
    window.show()?;
    window.set_focus()?;
    Ok(())
}

pub fn handle_close(app: &AppHandle<Wry>) -> Result<(), ProgramError> {
    get_main_window(app)?.hide()?;
    app::silent_emit_all(constants::event::WINDOW_CLOSE, {});
    Ok(())
}

pub fn handle_system_tray_event(app: &AppHandle<Wry>, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::DoubleClick { position: _, size: _, .. } => {
            match handle_dbl_click(app) {
                Ok(_) => {}
                Err(err) => {
                    log::error!("Failed to handle dbl click event, err: {}", err);
                }
            }
        }
        SystemTrayEvent::MenuItemClick { id, .. } => {
            match id.as_str() {
                "exit" => {
                    log::debug!("Exit from system tray");
                    // try to stop voicevox engine if running
                    voicevox::check_and_unload_binary();
                    voicevox::try_stop_engine_exe();
                    app.exit(0);
                }
                "close" => {
                    log::debug!("Close from system tray");
                    match handle_close(app) {
                        Ok(_) => {}
                        Err(err) => {
                            log::error!("Failed to handle close event, err: {}", err);
                        }
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
}

pub fn setup(app: &mut App<Wry>) -> Result<(), ProgramError> {
    let window = get_main_window(&app.app_handle())?;
    // if at dev mode, set window to the right side of current monitor
    if cfg!(debug_assertions) {
        log::debug!("Running at dev mode, move window to right side of the screen");
        let window_width = window.outer_size()?.width;
        let screen = window.current_monitor()?.ok_or("Monitor info not found")?;
        let screen_width = screen.size().width;
        let window_x = screen_width - window_width;
        let window_y = 100;
        window.set_position(tauri::PhysicalPosition { x: window_x, y: window_y })?;
    }

    audio_recorder::start_shortcut(app.app_handle())?;

    tauri::async_runtime::spawn(async {
        audio_manager::watch_audio_devices();
    });
    tauri::async_runtime::spawn(async {
        generator::start_check_audio_caches();
    });
    tauri::async_runtime::spawn(async {
        voice_engine::check_voicevox().await;
        whisper::check_whisper_lib().await;
    });


    Ok(())
}

fn main() {
    logger::setup::setup_logger();

    log::info!("Starting application");

    tauri::Builder::default()
        .setup(|app| {
            log::info!("Application started");
            common::app::set_app_handle(app.app_handle());
            setup(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::configs::get_voice_engine_config,
            commands::configs::save_voice_engine_config,
            commands::configs::get_auto_translation_config,
            commands::configs::save_auto_translation_config,
            commands::configs::get_voice_recognition_config,
            commands::configs::save_voice_recognition_config,
            commands::configs::get_audio_config,

            commands::voicevox::is_voicevox_engine_initialized,
            commands::voicevox::is_loading_voicevox_engine,
            commands::voicevox::check_voicevox_engine,
            commands::voicevox::stop_loading_voicevox_engine,
            commands::voicevox::get_voice_vox_speakers,
            commands::voicevox::get_voice_vox_speaker_info,
            commands::voicevox::available_voicevox_binaries,

            commands::audios::list_audios,
            commands::audios::get_audio_detail,
            commands::audios::delete_audio,
            commands::audios::play_audio,
            commands::audios::generate_audio,
            commands::audios::change_output_device,
            commands::audios::change_input_device,
            commands::audios::is_recorder_recording,

            commands::whisper::whisper_available_models,
        ])
        .system_tray(create_system_tray())
        .on_system_tray_event(handle_system_tray_event)
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
