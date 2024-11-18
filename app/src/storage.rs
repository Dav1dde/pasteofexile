use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use shared::model::PasteSummary;
use sycamore::prelude::*;

use crate::utils::LocalStorage;

pub fn provide_storage<G: Html>(cx: Scope) {
    if G::IS_BROWSER {
        provide_context(cx, Storage(LocalStorage::default()));
    }
}

pub struct Storage(LocalStorage);

impl Storage {
    pub fn visited(&self) -> PasteList<'_> {
        PasteList {
            key: "visited",
            storage: &self.0,
            max_size: Some(30),
        }
    }
}

#[derive(Copy, Clone)]
pub struct PasteList<'a> {
    key: &'a str,
    storage: &'a LocalStorage,
    max_size: Option<usize>,
}

impl PasteList<'_> {
    pub fn add(&self, summary: PasteSummary) {
        let mut entries = self.get_all();

        entries.retain(|e| e.paste.id != summary.id);
        entries.push_front(StoredPaste {
            stored: js_sys::Date::new_0().get_time() as u64,
            paste: summary,
        });

        if let Some(max_size) = self.max_size {
            entries.truncate(max_size);
        }

        self.storage.set(self.key, &SerDePasteHistory { entries });
    }

    pub fn get_all(&self) -> VecDeque<StoredPaste> {
        self.storage
            .get::<SerDePasteHistory>(self.key)
            .unwrap_or_default()
            .entries
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct StoredPaste {
    pub stored: u64,
    pub paste: PasteSummary,
}

#[derive(Default, Deserialize, Serialize)]
struct SerDePasteHistory {
    entries: VecDeque<StoredPaste>,
}
