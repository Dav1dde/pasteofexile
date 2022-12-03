use super::{b2_client, StoredPaste};
use crate::{
    sentry, utils,
    utils::{b64_decode, b64_encode},
    Error, Result,
};
use shared::{
    model::{ListPaste, PasteId, PasteMetadata},
    User,
};
use std::rc::Rc;

pub struct B2Storage {
    b2: b2_client::B2,
}

impl B2Storage {
    pub fn from_env(env: &worker::Env) -> Result<Self> {
        Ok(Self {
            b2: b2_client::B2::from_env(env)?,
        })
    }

    #[tracing::instrument(skip(self))]
    pub async fn get(&self, id: &PasteId) -> Result<Option<StoredPaste>> {
        let path = super::to_path(id)?;
        let mut response = self.b2.download(&path).await?;

        match response.status_code() {
            200 => {
                let metadata = response
                    .headers()
                    .get("X-Bz-Info-Metadata")
                    .unwrap()
                    .map(b64_decode)
                    .transpose()?
                    .map(|m| serde_json::from_slice(&m))
                    .transpose()?;

                let entity_id = response
                    .headers()
                    .get("X-Bz-Content-Sha1")
                    .unwrap()
                    // this should always exist, but better be safe than sorry and just fallback to empty
                    .unwrap_or_default();

                let last_modified = response
                    .headers()
                    .get("X-Bz-Upload-Timestamp")
                    .unwrap()
                    .and_then(|x| x.parse().ok())
                    .unwrap_or(0);

                let content = response.text().await?;

                Ok(Some(StoredPaste {
                    metadata,
                    last_modified,
                    entity_id,
                    content,
                }))
            }
            404 => Ok(None),
            status => Err(Error::RemoteFailed(
                status,
                "failed to get paste".to_owned(),
            )),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: &PasteId) -> Result<()> {
        let path = super::to_path(id)?;
        self.b2.hide(&path).await.map(|_| ())
    }

    #[tracing::instrument(skip(self, sha1, data))]
    pub async fn put(
        &self,
        id: &PasteId,
        sha1: &[u8],
        data: &[u8],
        metadata: Option<PasteMetadata>,
    ) -> Result<()> {
        let path = super::to_path(id)?;
        let hex = utils::hex(sha1);
        let metadata = metadata
            .map(|m| serde_json::to_string(&m))
            .transpose()?
            .map(b64_encode);
        let settings = b2_client::UploadSettings {
            filename: &path,
            content_type: "text/plain",
            sha1: Some(&hex),
            metadata: metadata.as_deref(),
        };

        self.b2.upload(&settings, data).await.map(|_| ())
    }

    #[tracing::instrument(skip(self, ctx, sha1, data))]
    pub async fn put_async(
        self,
        ctx: &worker::Context,
        id: &PasteId,
        sha1: &[u8],
        data: Rc<[u8]>,
        metadata: Option<PasteMetadata>,
    ) -> Result<()> {
        let path = super::to_path(id)?;

        let hex = utils::hex(sha1);
        let metadata = metadata
            .map(|m| serde_json::to_string(&m))
            .transpose()?
            .map(b64_encode);
        let future = async move {
            let settings = b2_client::UploadSettings {
                filename: &path,
                content_type: "text/plain",
                sha1: Some(&hex),
                metadata: metadata.as_deref(),
            };

            if let Err(err) = self.b2.upload(&settings, &data).await {
                // TODO: there might not be an active transaction anymore?
                tracing::error!("<-- failed to upload paste: {:?}", err);
                // TODO: is this necessary, tracing should pick it up already from tracing::erorr!
                sentry::with_sentry(|sentry| sentry.capture_err_level(&err, sentry::Level::Error));
            } else {
                tracing::debug!("<-- paste uploaded");
            }
        };
        ctx.wait_until(future);
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn list(&self, user: &User) -> Result<Vec<ListPaste>> {
        let response = self.b2.list_files(&super::to_prefix(user), 100).await?;

        response
            .files
            .into_iter()
            .map(|f| {
                Ok(ListPaste {
                    name: f.file_name,
                    metadata: f
                        .file_info
                        .get("metadata")
                        .map(b64_decode)
                        .transpose()?
                        .map(|m| serde_json::from_slice(&m))
                        .transpose()?,
                    last_modified: f.upload_timestamp,
                })
            })
            .collect::<Result<_>>()
    }
}
