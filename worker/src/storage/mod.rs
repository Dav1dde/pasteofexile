use crate::{
    sentry, utils,
    utils::{b64_decode, b64_encode, hex},
    Error, Result,
};
use serde::{Deserialize, Serialize};
use shared::model::{ListPaste, Paste, PasteId, PasteMetadata};

pub mod b2;

#[cfg(not(feature = "use-kv-storage"))]
pub type DefaultStorage = B2Storage;
#[cfg(feature = "use-kv-storage")]
pub type DefaultStorage = KvStorage;

#[allow(dead_code)]
pub struct B2Storage {
    b2: b2::B2,
}

fn to_path(id: &PasteId) -> Result<String> {
    match id {
        PasteId::Paste(id) => Ok(crate::utils::to_path(id)?),
        PasteId::UserPaste(up) => Ok(format!("user/{}/pastes/{}", up.user, up.id)),
    }
}

#[allow(dead_code)]
impl B2Storage {
    pub fn from_env(env: &worker::Env) -> Result<Self> {
        Ok(Self {
            b2: b2::B2::from_env(env)?,
        })
    }

    pub async fn get(&self, id: &PasteId) -> Result<Option<Paste>> {
        let path = to_path(id)?;
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

                let entity_id = response.headers().get("X-Bz-Content-Sha1").unwrap();

                let last_modified = response
                    .headers()
                    .get("X-Bz-Upload-Timestamp")
                    .unwrap()
                    .and_then(|x| x.parse().ok())
                    .unwrap_or(0);

                let content = response.text().await?;

                Ok(Some(Paste {
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

    pub async fn delete(&self, id: &PasteId) -> Result<()> {
        let path = to_path(id)?;
        self.b2.hide(&path).await.map(|_| ())
    }

    pub async fn put(
        &self,
        id: &PasteId,
        sha1: &[u8],
        data: &mut [u8],
        metadata: Option<PasteMetadata>,
    ) -> Result<()> {
        let path = to_path(id)?;
        let hex = utils::hex(sha1);
        let metadata = metadata
            .map(|m| serde_json::to_string(&m))
            .transpose()?
            .map(b64_encode);
        let settings = b2::UploadSettings {
            filename: &path,
            content_type: "text/plain",
            sha1: Some(&hex),
            metadata: metadata.as_deref(),
        };

        let upload = self.b2.get_upload_url().await?;
        self.b2.upload(&settings, data, upload).await.map(|_| ())
    }

    pub async fn put_async(
        self,
        ctx: &worker::Context,
        id: &PasteId,
        sha1: &[u8],
        mut data: Vec<u8>,
        metadata: Option<PasteMetadata>,
    ) -> Result<()> {
        let path = to_path(id)?;
        let upload = self.b2.get_upload_url().await?;

        let hex = utils::hex(sha1);
        let metadata = metadata
            .map(|m| serde_json::to_string(&m))
            .transpose()?
            .map(b64_encode);
        let future = async move {
            let settings = b2::UploadSettings {
                filename: &path,
                content_type: "text/plain",
                sha1: Some(&hex),
                metadata: metadata.as_deref(),
            };

            if let Err(err) = self.b2.upload(&settings, &mut data, upload).await {
                log::error!("<-- failed to upload paste: {:?}", err);
                sentry!(sentry, sentry.capture_err_level(&err, "error"));
            } else {
                log::debug!("<-- paste uploaded");
            }
        };
        ctx.wait_until(future);
        Ok(())
    }

    pub async fn list(&self, prefix: impl Into<String>) -> Result<Vec<ListPaste>> {
        let prefix = prefix.into();
        let response = self.b2.list_files(&prefix, 100).await?;

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

#[allow(dead_code)]
impl KvStorage {
    pub fn from_env(env: &worker::Env) -> Result<Self> {
        Ok(Self {
            kv: env.kv(crate::consts::KV_PASTE_STORAGE)?,
        })
    }

    pub async fn get(&self, id: &PasteId) -> Result<Option<Paste>> {
        let path = to_path(id)?;
        let (data, metadata) = self
            .kv
            .get(&path)
            .text_with_metadata::<KvMetadata>()
            .await?;

        let metadata = metadata.unwrap_or_default();

        Ok(data.map(|content| Paste {
            content,
            metadata: metadata.metadata,
            entity_id: metadata.entity_id,
            last_modified: metadata.last_modified,
        }))
    }

    pub async fn delete(&self, id: &PasteId) -> Result<()> {
        let path = to_path(id)?;
        self.kv.delete(&path).await?;
        Ok(())
    }

    pub async fn put(
        &self,
        id: &PasteId,
        sha1: &[u8],
        data: &mut [u8],
        metadata: Option<PasteMetadata>,
    ) -> Result<()> {
        let path = to_path(id)?;
        self.kv
            .put_bytes(&path, data)?
            .metadata(KvMetadata {
                entity_id: Some(hex(sha1)),
                last_modified: js_sys::Date::new_0().get_time() as u64,
                metadata,
            })?
            .execute()
            .await?;
        Ok(())
    }

    pub async fn put_async(
        self,
        ctx: &worker::Context,
        id: &PasteId,
        sha1: &[u8],
        data: Vec<u8>,
        metadata: Option<PasteMetadata>,
    ) -> Result<()> {
        let path = to_path(id)?;
        let metadata = KvMetadata {
            entity_id: Some(hex(sha1)),
            last_modified: js_sys::Date::new_0().get_time() as u64,
            metadata,
        };
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
                log::error!("<-- failed to upload paste: {:?}", err);
                sentry!(sentry, sentry.capture_err(&err.into()));
            } else {
                log::debug!("<-- paste uploaded");
            }
        };
        ctx.wait_until(future);
        Ok(())
    }

    pub async fn list(&self, path: impl Into<String>) -> Result<Vec<ListPaste>> {
        let response = self.kv.list().prefix(path.into()).execute().await?;

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
