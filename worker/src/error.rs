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
    // error, XML
    InvalidPoB(String, String),

    #[error("{0}")]
    Error(String),
}

impl Error {
    pub fn name(&self) -> &'static str {
        match self {
            Self::NotFound(..) => "NotFound",
            Self::RemoteFailed(..) => "Remote Failed",
            Self::Serde(..) => "Serde",
            Self::Kv(..) => "Kv",
            Self::Worker(..) => "Worker",
            Self::BadRequest(..) => "BadRequest",
            Self::InvalidPoB(..) => "InvalidPoB",
            Self::Error(..) => "Error",
        }
    }

    pub fn level(&self) -> &'static str {
        match self {
            Self::NotFound(..) => "info",
            Self::RemoteFailed(..) => "warning",
            Self::Serde(..) => "error",
            Self::Kv(..) => "error",
            Self::Worker(..) => "error",
            Self::BadRequest(..) => "info",
            Self::InvalidPoB(..) => "error",
            Self::Error(..) => "error",
        }
    }

    pub fn payload(&self) -> Option<&str> {
        match self {
            Self::InvalidPoB(_, ref payload) => Some(payload),
            _ => None,
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
            err @ Error::NotFound(..) => ErrorResponse {
                code: 404,
                message: err.to_string(),
            },
            err @ Error::BadRequest(..) | err @ Error::InvalidPoB(..) => ErrorResponse {
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
