use std::{collections::HashMap, rc::Rc};

use serde::{Deserialize, Serialize};
use shared::{
    model::{ListPaste, PasteId, PasteMetadata},
    User,
};
use worker::{Bucket, HttpMetadata, Include, Object};

use super::StoredPaste;
use crate::{
    request_context::{Env, FromEnv},
    utils::{b64_decode, b64_encode},
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

pub struct R2Storage {
    bucket: Bucket,
}

impl FromEnv for R2Storage {
    fn from_env(env: &Env) -> Option<Self> {
        Some(Self {
            bucket: env.bucket(crate::consts::R2_STORAGE_BUCKET)?,
        })
    }
}

impl R2Storage {
    #[tracing::instrument(skip(self))]
    pub async fn get(&self, id: &PasteId) -> Result<Option<StoredPaste>> {
        let path = super::to_path(id)?;
        let Some(obj) = self.bucket.get(path).execute().await? else {
            return Ok(None);
        };

        let content = match obj.body() {
            Some(body) => body.text().await?,
            None => return Ok(None),
        };

        let (mtime, metadata) = to_metadata(&obj)?;

        Ok(Some(StoredPaste {
            content,
            metadata,
            entity_id: obj.etag(),
            last_modified: mtime,
        }))
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: &PasteId) -> Result<()> {
        let path = super::to_path(id)?;
        self.bucket.delete(path).await?;
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

        let metadata = metadata
            .map(serde_json::to_string)
            .transpose()?
            .map(b64_encode);

        let mut custom_metdata = HashMap::new();
        if let Some(metadata) = metadata {
            custom_metdata.insert("metadata".to_owned(), metadata);
        }

        self.bucket
            .put(path, worker::Data::Bytes(data))
            .http_metadata(HttpMetadata {
                content_type: Some("text/plain".to_owned()),
                ..Default::default()
            })
            .custom_metdata(custom_metdata)
            .sha1(sha1.try_into().unwrap_or_default())
            .execute()
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self, _rctx, sha1, data))]
    pub async fn put_async(
        self,
        _rctx: &crate::RequestContext,
        id: &PasteId,
        sha1: &[u8],
        data: Rc<[u8]>,
        metadata: Option<&PasteMetadata>,
    ) -> Result<()> {
        self.put(id, sha1, &data, metadata).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn list(&self, user: &User) -> Result<Vec<ListPaste>> {
        let prefix = super::to_prefix(user);
        let objects = self
            .bucket
            .list()
            .prefix(&prefix)
            .include(vec![Include::CustomMetadata])
            .limit(100)
            .execute()
            .await?;

        objects
            .objects()
            .into_iter()
            .map(|obj| {
                let (mtime, metadata) = to_metadata(&obj)?;
                Ok(ListPaste {
                    name: super::strip_prefix(&obj.key(), &prefix)?,
                    metadata,
                    last_modified: mtime,
                })
            })
            .collect::<Result<_>>()
    }
}

fn to_metadata(obj: &Object) -> Result<(u64, Option<PasteMetadata>)> {
    let custom_metdata = obj.custom_metadata()?;

    let mtime = custom_metdata
        .get("mtime")
        .and_then(|mtime| mtime.parse::<f32>().ok())
        .map(|mtime| (mtime * 1000.0) as u64)
        .unwrap_or_else(|| obj.uploaded().as_millis());

    let metadata = custom_metdata
        .get("metadata")
        .map(b64_decode)
        .transpose()?
        .map(|m| serde_json::from_slice(&m))
        .transpose()?;

    Ok((mtime, metadata))
}
