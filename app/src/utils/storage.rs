use serde::{Deserialize, Serialize};
use wasm_bindgen::UnwrapThrowExt;

/// A tiny abstraction over [`web_sys::Storage`], which resets data on error.
///
/// Abstraction over local storage which integrates with serde.
/// Deserialization errors are silently ignored and treated as if there simply was no data.
#[derive(Clone)]
pub struct LocalStorage(web_sys::Storage);

impl LocalStorage {
    pub fn new() -> Self {
        let storage = web_sys::window()
            .expect_throw("expected window")
            .local_storage()
            .expect_throw("unable to get local storage")
            .expect_throw("no local storage");
        LocalStorage(storage)
    }

    pub fn get<T>(&self, key: &str) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let item = self.0.get_item(key).unwrap_throw()?;
        serde_json::from_str(&item).ok()
    }

    pub fn set<T>(&self, key: &str, value: &T)
    where
        T: Serialize,
    {
        let value = serde_json::to_string(value).unwrap_throw();
        let _ = self.0.set_item(key, &value);
    }
}

impl Default for LocalStorage {
    fn default() -> Self {
        Self::new()
    }
}
