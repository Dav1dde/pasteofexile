use crate::{dangerous::DangerousError, sentry::Level};
use serde::Serialize;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("The requested '{0}' with does not exist")]
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

    #[error("Access Denied")]
    AccessDenied,

    #[error("{0}")]
    // error, XML
    InvalidPoB(String, String),

    #[error("{0}")]
    InvalidId(&'static str),

    #[error(transparent)]
    Dangerous(#[from] DangerousError),

    #[error("{0}")]
    Base64(#[from] base64::DecodeError),

    #[error("{0}")]
    IOError(#[from] std::io::Error),

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
            Self::AccessDenied => "AccessDenied",
            Self::InvalidPoB(..) => "InvalidPoB",
            Self::InvalidId(..) => "InvalidId",
            Self::Dangerous(..) => "DangerousError",
            Self::Base64(..) => "Base64",
            Self::IOError(..) => "IOError",
            Self::Error(..) => "Error",
        }
    }

    pub fn level(&self) -> Level {
        match self {
            Self::NotFound(..) => Level::Info,
            Self::RemoteFailed(..) => Level::Warning,
            Self::Serde(..) => Level::Error,
            Self::Kv(..) => Level::Error,
            Self::Worker(..) => Level::Error,
            Self::BadRequest(..) => Level::Info,
            Self::AccessDenied => Level::Info,
            Self::InvalidId(..) => Level::Info,
            Self::InvalidPoB(..) => Level::Error,
            Self::Dangerous(err) => match err {
                DangerousError::BadEncoding => Level::Warning,
                DangerousError::BadSignature => Level::Warning,
                DangerousError::Deserialize => Level::Warning,
                _ => Level::Error,
            },
            Self::Base64(..) => Level::Error,
            Self::IOError(..) => Level::Error,
            Self::Error(..) => Level::Error,
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
            err @ Error::AccessDenied => ErrorResponse {
                code: 403,
                message: err.to_string(),
            },
            Error::Dangerous(err) => ErrorResponse {
                code: match err {
                    DangerousError::BadEncoding => 400,
                    DangerousError::BadSignature => 400,
                    DangerousError::Deserialize => 400,
                    _ => 500,
                },
                message: err.to_string(),
            },
            Error::InvalidId(message) => ErrorResponse {
                code: 404,
                message: message.to_owned(),
            },
            err => ErrorResponse {
                code: 500,
                message: err.to_string(),
            },
        }
    }
}
