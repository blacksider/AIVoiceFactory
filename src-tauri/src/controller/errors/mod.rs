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
}

impl Error for CommonError {}

#[derive(Debug)]
pub struct ResponseError {
    status: reqwest::StatusCode,
    message: String,
}

impl ResponseError {
    pub fn new(status: reqwest::StatusCode, message: String) -> ResponseError {
        ResponseError {
            status,
            message,
        }
    }
}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "status: {}, msg: {}", self.status, self.message)
    }
}

impl Error for ResponseError {}

impl From<reqwest::Error> for ResponseError {
    fn from(err: reqwest::Error) -> Self {
        let status;
        match err.status() {
            None => {
                status = StatusCode::BAD_GATEWAY
            }
            Some(value) => {
                status = value
            }
        }
        ResponseError {
            status,
            message: err.to_string(),
        }
    }
}
