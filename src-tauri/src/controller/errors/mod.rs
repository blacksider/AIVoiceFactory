use std::error::Error;
use std::fmt;

use reqwest::StatusCode;

#[derive(Debug)]
pub struct CommonError {
    message: String,
}

impl fmt::Display for CommonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl CommonError {
    pub fn new(message: String) -> CommonError {
        CommonError {
            message,
        }
    }

    pub fn from_http_error(status: StatusCode, message: String) -> CommonError {
        CommonError {
            message: format!("http request return status: {}, error: {}",
                             status, message),
        }
    }
}

impl Error for CommonError {}

/// A common error wrapper for all kind of errors we met
#[derive(Debug)]
pub struct ProgramError {
    error: Box<dyn Error>,
}

impl Error for ProgramError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.error.as_ref())
    }
}

impl fmt::Display for ProgramError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

unsafe impl Send for ProgramError {}

unsafe impl Sync for ProgramError {}

impl ProgramError {
    pub fn wrap<E>(err: E) -> ProgramError where E: Error + 'static {
        ProgramError {
            error: Box::new(err)
        }
    }
}

impl From<&str> for ProgramError {
    fn from(err: &str) -> Self {
        ProgramError {
            error: Box::new(CommonError::new(err.to_string()))
        }
    }
}

impl From<String> for ProgramError {
    fn from(err: String) -> Self {
        ProgramError {
            error: Box::new(CommonError::new(err))
        }
    }
}

macro_rules! impl_program_error {
  ($error_type:ty) => {
    impl From<$error_type> for ProgramError {
        fn from(error: $error_type) -> Self {
            ProgramError::wrap(error)
        }
    }
  }
}

impl_program_error!(serde_json::Error);
impl_program_error!(std::io::Error);
impl_program_error!(sled::Error);
impl_program_error!(sled::transaction::TransactionError);
impl_program_error!(reqwest::Error);
impl_program_error!(cpal::DevicesError);
impl_program_error!(cpal::DeviceNameError);
impl_program_error!(tauri::Error);
impl_program_error!(tauri_runtime::Error);
impl_program_error!(cpal::DefaultStreamConfigError);
impl_program_error!(cpal::BuildStreamError);
impl_program_error!(cpal::PlayStreamError);
impl_program_error!(hound::Error);
impl_program_error!(rodio::PlayError);
impl_program_error!(rodio::StreamError);
impl_program_error!(rodio::decoder::DecoderError);
impl_program_error!(sevenz_rust::Error);
impl_program_error!(samplerate::error::Error);
impl_program_error!(libloading::Error);
impl_program_error!(tokio::sync::TryLockError);
impl_program_error!(Box<bincode::ErrorKind>);
impl_program_error!(CommonError);
