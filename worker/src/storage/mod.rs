use std::rc::Rc;

use serde::{Deserialize, Serialize};
use shared::{
    model::{ListPaste, PasteId, PasteMetadata},
    User,
};

use crate::{
    request_context::{Env, FromEnv},
    Result,
};

#[allow(dead_code)]
mod b2;
mod b2_client;
#[allow(dead_code)]
mod kv;
mod pastebin;
mod utils;

#[cfg(not(feature = "use-kv-storage"))]
use b2::B2Storage as DefaultStorage;
#[cfg(feature = "use-kv-storage")]
use kv::KvStorage as DefaultStorage;
pub(crate) use utils::{to_path, to_prefix};

#[derive(Debug, Deserialize, Serialize)]
pub struct StoredPaste {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<PasteMetadata>,
    #[serde(default)]
    pub last_modified: u64,
    pub entity_id: String,
    pub content: String,
}

pub struct Storage {
    storage: DefaultStorage,
}

impl FromEnv for Storage {
    fn from_env(env: &Env) -> Option<Self> {
        Some(Self {
            storage: DefaultStorage::from_env(env)?,
        })
    }
}

impl Storage {
    pub async fn get(&self, id: &PasteId) -> Result<Option<StoredPaste>> {
        if pastebin::could_be_pastebin_id(id) {
            tracing::info!("fetching from pastebin.com");
            pastebin::get(id).await
        } else {
            self.storage.get(id).await
        }
    }

    pub async fn delete(&self, id: &PasteId) -> Result<()> {
        self.storage.delete(id).await
    }

    pub async fn put(
        &self,
        id: &PasteId,
        sha1: &[u8],
        data: &[u8],
        metadata: Option<&PasteMetadata>,
    ) -> Result<()> {
        self.storage.put(id, sha1, data, metadata).await
    }

    pub async fn put_async(
        self,
        rctx: &crate::RequestContext,
        id: &PasteId,
        sha1: &[u8],
        data: Rc<[u8]>,
        metadata: Option<&PasteMetadata>,
    ) -> Result<()> {
        self.storage.put_async(rctx, id, sha1, data, metadata).await
    }

    pub async fn list(&self, user: &User) -> Result<Vec<ListPaste>> {
        self.storage.list(user).await
    }
}
