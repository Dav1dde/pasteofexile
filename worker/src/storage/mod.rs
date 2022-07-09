use crate::{
    sentry, utils,
    utils::{b64_decode, b64_encode},
    Error, Result,
};
use serde::{de::DeserializeOwned, Serialize};

pub mod b2;

#[cfg(not(feature = "use-kv-storage"))]
pub type DefaultStorage = B2Storage;
#[cfg(feature = "use-kv-storage")]
pub type DefaultStorage = KvStorage;

#[derive(Debug)]
pub struct ListItem<T> {
    pub name: String,
    pub metadata: Option<T>,
}

#[allow(dead_code)]
pub struct B2Storage {
    b2: b2::B2,
}

#[allow(dead_code)]
impl B2Storage {
    pub fn from_env(env: &worker::Env) -> Result<Self> {
        Ok(Self {
            b2: b2::B2::from_env(env)?,
        })
    }

    pub async fn get(&self, path: &str) -> Result<Option<worker::Response>> {
        let response = self.b2.download(path).await?;

        match response.status_code() {
            200 => Ok(Some(response)),
            404 => Ok(None),
            status => Err(Error::RemoteFailed(
                status,
                "failed to get paste".to_owned(),
            )),
        }
    }

    pub async fn delete(&self, path: &str) -> Result<()> {
        self.b2.hide(path).await.map(|_| ())
    }

    pub async fn put<T: Serialize + 'static>(
        &self,
        filename: &str,
        sha1: &[u8],
        data: &mut [u8],
        metadata: Option<T>,
    ) -> Result<()> {
        let hex = utils::hex(sha1);
        let metadata = metadata
            .map(|m| serde_json::to_string(&m))
            .transpose()?
            .map(b64_encode);
        let settings = b2::UploadSettings {
            filename,
            content_type: "text/plain",
            sha1: Some(&hex),
            metadata: metadata.as_deref(),
        };

        let upload = self.b2.get_upload_url().await?;
        self.b2.upload(&settings, data, upload).await.map(|_| ())
    }

    pub async fn put_async<T: Serialize + 'static>(
        self,
        ctx: &worker::Context,
        filename: String,
        sha1: &[u8],
        mut data: Vec<u8>,
        metadata: Option<T>,
    ) -> Result<()> {
        let upload = self.b2.get_upload_url().await?;

        let hex = utils::hex(sha1);
        let metadata = metadata
            .map(|m| serde_json::to_string(&m))
            .transpose()?
            .map(b64_encode);
        let future = async move {
            let settings = b2::UploadSettings {
                filename: &filename,
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

    pub async fn list<T: DeserializeOwned>(
        &self,
        prefix: impl Into<String>,
    ) -> Result<Vec<ListItem<T>>> {
        let prefix = prefix.into();
        let response = self.b2.list_files(&prefix, 100).await?;

        response
            .files
            .into_iter()
            .map(|f| {
                Ok(ListItem {
                    name: f.file_name,
                    metadata: f
                        .file_info
                        .get("metadata")
                        .map(b64_decode)
                        .transpose()?
                        .map(|m| serde_json::from_slice(&m))
                        .transpose()?,
                })
            })
            .collect::<Result<_>>()
    }
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

    pub async fn get(&self, path: &str) -> Result<Option<worker::Response>> {
        let data = self.kv.get(path).text().await?;
        Ok(data.map(|data| worker::Response::ok(data).unwrap()))
    }

    pub async fn delete(&self, path: &str) -> Result<()> {
        self.kv.delete(path).await?;
        Ok(())
    }

    pub async fn put<T: Serialize + 'static>(
        &self,
        filename: &str,
        _sha1: &[u8],
        data: &mut [u8],
        metadata: Option<T>,
    ) -> Result<()> {
        self.kv
            .put_bytes(filename, data)?
            .metadata(metadata)?
            .execute()
            .await?;
        Ok(())
    }

    pub async fn put_async<T: Serialize + 'static>(
        self,
        ctx: &worker::Context,
        filename: String,
        _sha1: &[u8],
        data: Vec<u8>,
        metadata: Option<T>,
    ) -> Result<()> {
        let future = async move {
            let r = self
                .kv
                .put_bytes(&filename, &data)
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

    pub async fn list<T: DeserializeOwned>(
        &self,
        path: impl Into<String>,
    ) -> Result<Vec<ListItem<T>>> {
        let response = self.kv.list().prefix(path.into()).execute().await?;

        response
            .keys
            .into_iter()
            .map(|key| {
                Ok(ListItem {
                    name: key.name,
                    metadata: key
                        .metadata
                        .map(|metadata| serde_json::from_value(metadata))
                        .transpose()?,
                })
            })
            .collect::<Result<_>>()
    }
}
