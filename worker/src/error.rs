use serde::Serialize;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("The requested {0} with id {1} does not exist")]
    NotFound(&'static str, String),

    #[error("Request failed {0}: {1}")]
    RemoteFailed(u16, String),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Kv(#[from] worker::kv::KvError),

    #[error(transparent)]
    Worker(#[from] worker::Error),

    #[error("{0}")]
    BadRequest(String),

    #[error("{0}")]
    InvalidPoB(String),

    #[error("{0}")]
    Error(String),
}

impl Error {
    pub fn name(&self) -> &'static str {
        match self {
            Self::NotFound(_, _) => "NotFound",
            Self::RemoteFailed(_, _) => "Remote Failed",
            Self::Serde(_) => "Serde",
            Self::Kv(_) => "Kv",
            Self::Worker(_) => "Worker",
            Self::BadRequest(_) => "BadRequest",
            Self::InvalidPoB(_) => "InvalidPoB",
            Self::Error(_) => "Error",
        }
    }

    pub fn level(&self) -> &'static str {
        match self {
            Self::NotFound(_, _) => "info",
            Self::RemoteFailed(_, _) => "warning",
            Self::Serde(_) => "error",
            Self::Kv(_) => "error",
            Self::Worker(_) => "error",
            Self::BadRequest(_) => "info",
            Self::InvalidPoB(_) => "error",
            Self::Error(_) => "error",
        }
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Error::Error(err)
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Error::Error(err.to_owned())
    }
}

impl From<wasm_bindgen::JsValue> for Error {
    fn from(js_value: wasm_bindgen::JsValue) -> Self {
        Self::Worker(js_value.into())
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub code: u16,
    pub message: String,
}

impl From<Error> for ErrorResponse {
    fn from(err: Error) -> Self {
        match err {
            err @ Error::NotFound(_, _) => ErrorResponse {
                code: 404,
                message: err.to_string(),
            },
            err @ Error::BadRequest(_) | err @ Error::InvalidPoB(_) => ErrorResponse {
                code: 400,
                message: err.to_string(),
            },
            err => ErrorResponse {
                code: 500,
                message: err.to_string(),
            },
        }
    }
}
