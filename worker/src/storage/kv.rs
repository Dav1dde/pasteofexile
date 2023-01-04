use std::rc::Rc;

use serde::{Deserialize, Serialize};
use shared::{
    model::{ListPaste, PasteId, PasteMetadata},
    User,
};

use super::StoredPaste;
use crate::{
    request_context::{Env, FromEnv},
    sentry::{self, WithSentry},
    utils::hex,
    Result,
};

#[derive(Default, Serialize, Deserialize)]
struct KvMetadata {
    #[serde(default)]
    last_modified: u64,
    #[serde(default)]
    entity_id: Option<String>,
    #[serde(flatten)]
    metadata: Option<PasteMetadata>,
}

#[allow(dead_code)]
pub struct KvStorage {
    kv: worker::kv::KvStore,
}

impl FromEnv for KvStorage {
    fn from_env(env: &Env) -> Option<Self> {
        Some(Self {
            kv: env.kv(crate::consts::KV_PASTE_STORAGE)?,
        })
    }
}

#[allow(dead_code)]
impl KvStorage {
    #[tracing::instrument(skip(self))]
    pub async fn get(&self, id: &PasteId) -> Result<Option<StoredPaste>> {
        let path = super::to_path(id)?;
        let (data, metadata) = self
            .kv
            .get(&path)
            .text_with_metadata::<KvMetadata>()
            .await?;

        let metadata = metadata.unwrap_or_default();

        Ok(data.map(|content| StoredPaste {
            content,
            metadata: metadata.metadata,
            entity_id: metadata.entity_id.unwrap_or_default(),
            last_modified: metadata.last_modified,
        }))
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: &PasteId) -> Result<()> {
        let path = super::to_path(id)?;
        self.kv.delete(&path).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self, sha1, data))]
    pub async fn put(
        &self,
        id: &PasteId,
        sha1: &[u8],
        data: &[u8],
        metadata: Option<&PasteMetadata>,
    ) -> Result<()> {
        let path = super::to_path(id)?;
        self.kv
            .put_bytes(&path, data)?
            .metadata(serde_json::json!({
                "entity_id": hex(sha1),
                "last_modified": js_sys::Date::new_0().get_time() as u64,
                "metadata": metadata,
            }))?
            .execute()
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self, ctx, sha1, data))]
    pub async fn put_async(
        self,
        ctx: &worker::Context,
        id: &PasteId,
        sha1: &[u8],
        data: Rc<[u8]>,
        metadata: Option<&PasteMetadata>,
    ) -> Result<()> {
        let path = super::to_path(id)?;
        let metadata = serde_json::json!({
            "entity_id": hex(sha1),
            "last_modified": js_sys::Date::new_0().get_time() as u64,
            "metadata": metadata,
        });
        let future = async move {
            let r = self
                .kv
                .put_bytes(&path, &data)
                .unwrap()
                .metadata(metadata)
                .unwrap()
                .execute()
                .await;

            if let Err(err) = r {
                tracing::warn!("<-- failed to upload paste: {:?}", err);
                sentry::capture_err_level(&err.into(), sentry::Level::Error);
            } else {
                tracing::debug!("<-- paste uploaded");
            }
        };
        ctx.wait_until(future.with_current_sentry());
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn list(&self, user: &User) -> Result<Vec<ListPaste>> {
        let response = self
            .kv
            .list()
            .prefix(super::to_prefix(user))
            .execute()
            .await?;

        response
            .keys
            .into_iter()
            .map(|key| {
                Ok(ListPaste {
                    name: key.name,
                    metadata: key.metadata.map(serde_json::from_value).transpose()?,
                    last_modified: 0,
                })
            })
            .collect::<Result<_>>()
    }
}
