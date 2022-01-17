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
    Error(String),
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
