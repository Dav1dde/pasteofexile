mod b2;

// Only supposed to be used for local development
#[cfg(feature = "storage-kv")]
mod kv;

#[cfg(not(feature = "storage-kv"))]
pub use b2::{get, put};
#[cfg(feature = "storage-kv")]
pub use kv::{get, put};
