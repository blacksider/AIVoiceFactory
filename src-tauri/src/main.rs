// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate core;

use tauri::{AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, Window, Wry};

use crate::controller::{audio_manager, audio_recorder, generator};

mod logger;
mod cypher;
mod config;
mod commands;
mod controller;
mod utils;

fn create_system_tray() -> SystemTray {
    let quit = CustomMenuItem::new("exit".to_string(), "Exit");
    let hide = CustomMenuItem::new("close".to_string(), "Close");
    let tray_menu = SystemTrayMenu::new()
        .add_item(hide)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);
    SystemTray::new().with_menu(tray_menu)
}

pub fn handle_system_tray_event(app: &AppHandle<Wry>, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::DoubleClick { position: _, size: _, .. } => {
            let window = app.get_window("main").unwrap();
            window.unminimize().unwrap();
            window.show().unwrap();
            window.set_focus().unwrap();
        }
        SystemTrayEvent::MenuItemClick { id, .. } => {
            match id.as_str() {
                "exit" => {
                    log::info!("Exit from system tray");
                    app.exit(0);
                }
                "close" => {
                    app.get_window("main").unwrap().hide().unwrap();
                    app.emit_all("close", {}).unwrap();
                    log::debug!("Close from system tray");
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn main() {
    logger::setup::setup_logger();

    log::info!("Starting application");

    tauri::Builder::default()
        .setup(|app| {
            log::info!("Application started");
            // FIXME Setup window location to the right side of the screent
            // This is just for local testing, will be removed in the future
            let window: Window<Wry> = app.get_window("main").ok_or("Main window not found")?;
            let window_width = window.outer_size()?.width;
            let screen = window.current_monitor()?.ok_or("Monitor info not found")?;
            let screen_width = screen.size().width;
            let window_x = screen_width - window_width;
            let window_y = 100;
            window.set_position(tauri::PhysicalPosition { x: window_x, y: window_y })?;
            // -------------------------------------------------------------

            tauri::async_runtime::spawn(async {
                audio_manager::watch_audio_devices(window);
            });
            tauri::async_runtime::spawn(async {
                generator::start_check_audio_caches();
            });

            match audio_recorder::start_shortcut(&app.app_handle()) {
                Ok(_) => {}
                Err(err) => {
                    log::error!("Unable to start shortcut, err: {}", err);
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::mapping::cmd::get_voice_engine_config,
            commands::mapping::cmd::save_voice_engine_config,
            commands::mapping::cmd::get_auto_translation_config,
            commands::mapping::cmd::save_auto_translation_config,
            commands::mapping::cmd::get_voice_recognition_config,
            commands::mapping::cmd::save_voice_recognition_config,
            commands::mapping::cmd::list_audios,
            commands::mapping::cmd::get_audio_detail,
            commands::mapping::cmd::play_audio,
            commands::mapping::cmd::generate_audio,
            commands::mapping::cmd::get_audio_config,
            commands::mapping::cmd::get_voice_vox_speakers,
            commands::mapping::cmd::get_voice_vox_speaker_info,
            commands::mapping::cmd::change_output_device,
            commands::mapping::cmd::is_recorder_recording,
        ])
        .system_tray(create_system_tray())
        .on_system_tray_event(handle_system_tray_event)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
