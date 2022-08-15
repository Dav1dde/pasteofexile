use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    /// Error indicating a page cannot be loaded, because a critical resource does not exist.
    /// For example the page /<id> cannot be loaded, because no resource with id <id> exists.
    #[error("The requested {0} with id {1} does not exist")]
    NotFound(&'static str, String),

    #[error("{0}: {1}")]
    UnhandledStatus(u16, String),

    #[error("Error {0}: {1}")]
    ApiError(u16, String),

    #[error(transparent)]
    Reqwasm(#[from] reqwasm::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    PobError(#[from] pob::Error),
}
