use std::collections::HashMap;

use shared::{
    model::{ListPaste, PasteMetadata},
    PasteId, User,
};
use worker::{Bucket, HttpMetadata, Include, Object};

use super::StoredPaste;
use crate::{
    crypto::Sha1,
    request_context::{Env, FromEnv},
    retry,
    utils::{b64_decode, b64_encode},
    Result,
};

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
        let path = super::to_path_r2(id)?;

        let obj = retry::retry_all(3, |_| self.bucket.get(&path).execute()).await?;

        let Some(obj) = obj else {
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
        let path = super::to_path_r2(id)?;

        retry::retry_all(3, |_| self.bucket.delete(&path)).await?;

        Ok(())
    }

    #[tracing::instrument(skip(self, sha1, data))]
    pub async fn put(
        &self,
        id: &PasteId,
        sha1: &Sha1,
        data: &[u8],
        metadata: Option<&PasteMetadata>,
    ) -> Result<()> {
        let path = super::to_path_r2(id)?;

        let metadata = metadata
            .map(serde_json::to_string)
            .transpose()?
            .map(b64_encode);

        let mut custom_metdata = HashMap::new();
        if let Some(metadata) = metadata {
            custom_metdata.insert("metadata".to_owned(), metadata);
        }

        retry::retry_all(3, |_| {
            self.bucket
                // TODO the to_vec() is wasted compute, but the worker crate sucks.
                .put(&path, worker::Data::Bytes(data.to_vec()))
                .http_metadata(HttpMetadata {
                    content_type: Some("text/plain".to_owned()),
                    ..Default::default()
                })
                .custom_metadata(custom_metdata.clone())
                .sha1(sha1.0)
                .execute()
        })
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn list(&self, user: &User) -> Result<Vec<ListPaste>> {
        let prefix = super::to_prefix_r2(user);

        let objects = retry::retry_all(3, |_| {
            self.bucket
                .list()
                .prefix(&prefix)
                .include(vec![Include::CustomMetadata])
                .limit(100)
                .execute()
        })
        .await?;

        objects
            .objects()
            .into_iter()
            .map(|obj| {
                let (mtime, metadata) = to_metadata(&obj)?;
                let metadata = metadata.ok_or_else(|| {
                    crate::Error::StorageError(format!(
                        "missing metadata on user paste {user}:{}",
                        obj.key()
                    ))
                })?;
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
