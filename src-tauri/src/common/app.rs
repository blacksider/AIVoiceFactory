use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;
use tauri::{AppHandle, Manager, Wry};

use crate::controller::errors::ProgramError;

lazy_static! {
    static ref APP_HANDLE: Arc<Mutex<App>> = Arc::new(Mutex::new(App::new()));
}

struct App {
    handle: Box<Option<AppHandle<Wry>>>,
}

unsafe impl Send for App {}

unsafe impl Sync for App {}

impl App {
    fn new() -> Self {
        App {
            handle: Box::new(None)
        }
    }

    fn set_handle(&mut self, handle: AppHandle<Wry>) {
        self.handle.replace(handle);
    }

    fn get_handle(&self) -> Result<AppHandle<Wry>, ProgramError> {
        let handle = *self.handle.clone();
        if handle.is_none() {
            return Err(ProgramError::from("app handle is empty"));
        }
        Ok(handle.unwrap().app_handle())
    }

    fn emit_all<S: serde::Serialize + Clone>(&self,
                                             event: &str,
                                             payload: S) -> Result<(), ProgramError> {
        self.get_handle()?.emit_all(event, payload)?;
        Ok(())
    }
}

pub fn set_app_handle(handle: AppHandle<Wry>) {
    let lock = APP_HANDLE.clone();
    let mut lock = lock.lock().unwrap();
    lock.set_handle(handle);
}

pub fn get_app_handle() -> Result<AppHandle<Wry>, ProgramError> {
    let lock = APP_HANDLE.clone();
    let lock = lock.lock().unwrap();
    lock.get_handle()
}

pub fn silent_emit_all<S: serde::Serialize + Clone>(event: &str,
                                                    payload: S) {
    let lock = APP_HANDLE.clone();
    let lock = lock.lock().unwrap();
    match lock.emit_all(event, payload) {
        Ok(_) => {}
        Err(err) => {
            log::error!("Failed to emit event {}, err: {}", event, err);
        }
    }
}