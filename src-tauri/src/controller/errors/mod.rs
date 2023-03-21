use std::error::Error;
use std::fmt;

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
        ResponseError {
            status: err.status().unwrap(),
            message: err.to_string(),
        }
    }
}
