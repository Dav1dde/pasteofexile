use serde::Serialize;
use thiserror::Error;

use crate::{dangerous::DangerousError, sentry::Level};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("The requested '{0}' does not exist")]
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

    #[error("Missing Authorization Grant")]
    MissingAuthorizationGrant,

    #[error("Authorization Grant Error: {0}")]
    AuthorizationGrantError(String),

    #[error("Invalid Session State")]
    InvalidSessionState,

    #[error(transparent)]
    PoEApiError(#[from] crate::poe_api::PoEApiError),

    #[error("{0}")]
    InvalidPoB(pob::Error, String),

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
            Self::MissingAuthorizationGrant => "MissingAuthorizationGrant",
            Self::AuthorizationGrantError(..) => "AuthorizationGrantError",
            Self::InvalidSessionState => "InvalidSessionState",
            Self::InvalidPoB(..) => "InvalidPoB",
            Self::InvalidId(..) => "InvalidId",
            Self::Dangerous(..) => "DangerousError",
            Self::PoEApiError(..) => "PoEApiError",
            Self::Base64(..) => "Base64",
            Self::IOError(..) => "IOError",
            Self::Error(..) => "Error",
        }
    }

    pub fn status_code(&self) -> u16 {
        match self {
            Self::NotFound(..) | Self::InvalidId(..) => 404,
            Self::BadRequest(..) | Self::InvalidPoB(..) => 400,
            Self::AccessDenied
            | Self::MissingAuthorizationGrant
            | Self::AuthorizationGrantError(..)
            | Self::InvalidSessionState => 403,
            Self::Dangerous(err) => match err {
                DangerousError::BadEncoding => 400,
                DangerousError::BadSignature => 400,
                DangerousError::Deserialize => 400,
                _ => 500,
            },
            _ => 500,
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
            Self::MissingAuthorizationGrant => Level::Warning,
            Self::AuthorizationGrantError(..) => Level::Warning,
            Self::InvalidSessionState => Level::Info,
            Self::InvalidId(..) => Level::Info,
            Self::InvalidPoB(..) => Level::Error,
            Self::Dangerous(err) => match err {
                DangerousError::BadEncoding => Level::Warning,
                DangerousError::BadSignature => Level::Warning,
                DangerousError::Deserialize => Level::Warning,
                _ => Level::Error,
            },
            Self::PoEApiError(..) => Level::Warning,
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

impl From<Error> for crate::Response {
    fn from(err: Error) -> Self {
        crate::Response::status(err.status_code()).json(&ErrorResponse {
            code: err.status_code(),
            message: err.to_string(),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub code: u16,
    pub message: String,
}
