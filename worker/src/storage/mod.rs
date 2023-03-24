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
mod pastebin;
mod r2;
mod utils;

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
    primary: r2::R2Storage,
    secondary: b2::B2Storage,
}

impl FromEnv for Storage {
    fn from_env(env: &Env) -> Option<Self> {
        Some(Self {
            primary: r2::R2Storage::from_env(env)?,
            secondary: b2::B2Storage::from_env(env)?,
        })
    }
}

impl Storage {
    pub async fn get(&self, id: &PasteId) -> Result<Option<StoredPaste>> {
        if pastebin::could_be_pastebin_id(id) {
            tracing::info!("fetching from pastebin.com");
            return pastebin::get(id).await;
        }

        if let Some(p) = self.primary.get(id).await? {
            return Ok(Some(p));
        }

        self.secondary.get(id).await
    }

    pub async fn delete(&self, id: &PasteId) -> Result<()> {
        // Ignore errors, lazy way of ignoring errors for files that dont exist
        let _ = self.primary.delete(id).await;
        let _ = self.secondary.delete(id).await;
        Ok(())
    }

    pub async fn put(
        &self,
        id: &PasteId,
        sha1: &[u8],
        data: &[u8],
        metadata: Option<&PasteMetadata>,
    ) -> Result<()> {
        self.primary.put(id, sha1, data, metadata).await
    }

    pub async fn put_auto(
        self,
        rctx: &crate::RequestContext,
        id: &PasteId,
        sha1: &[u8],
        data: Rc<[u8]>,
        metadata: Option<&PasteMetadata>,
    ) -> Result<()> {
        // Turkey blocks b2 for some reason...
        if rctx.country().as_deref() == Some("TR") {
            self.primary.put(id, sha1, &data, metadata).await
        } else {
            self.primary.put_async(rctx, id, sha1, data, metadata).await
        }
    }

    pub async fn list(&self, user: &User) -> Result<Vec<ListPaste>> {
        let r = self.primary.list(user).await?;
        if !r.is_empty() {
            return Ok(r);
        }
        self.secondary.list(user).await
    }
}
