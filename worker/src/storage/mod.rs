use crate::Result;
use shared::{
    model::{ListPaste, Paste, PasteId, PasteMetadata},
    User,
};
use std::rc::Rc;

#[allow(dead_code)]
mod b2;
mod b2_client;
#[allow(dead_code)]
mod kv;
mod pastebin;
mod utils;

pub(crate) use utils::{to_path, to_prefix};

#[cfg(not(feature = "use-kv-storage"))]
use b2::B2Storage as DefaultStorage;
#[cfg(feature = "use-kv-storage")]
use kv::KvStorage as DefaultStorage;

pub struct Storage {
    storage: DefaultStorage,
}

impl Storage {
    pub fn from_env(env: &worker::Env) -> Result<Self> {
        Ok(Self {
            storage: DefaultStorage::from_env(env)?,
        })
    }
}

impl Storage {
    pub async fn get(&self, id: &PasteId) -> Result<Option<Paste>> {
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
        metadata: Option<PasteMetadata>,
    ) -> Result<()> {
        self.storage.put(id, sha1, data, metadata).await
    }

    pub async fn put_async(
        self,
        ctx: &worker::Context,
        id: &PasteId,
        sha1: &[u8],
        data: Rc<[u8]>,
        metadata: Option<PasteMetadata>,
    ) -> Result<()> {
        self.storage.put_async(ctx, id, sha1, data, metadata).await
    }

    pub async fn list(&self, user: &User) -> Result<Vec<ListPaste>> {
        self.storage.list(user).await
    }
}
