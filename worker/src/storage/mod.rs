use crate::{sentry, utils, Error, Result};

pub mod b2;

#[cfg(not(feature = "use-kv-storage"))]
pub type DefaultStorage = B2Storage;
#[cfg(feature = "use-kv-storage")]
pub type DefaultStorage = KvStorage;

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

    pub async fn put(&self, filename: &str, sha1: &[u8], data: &mut [u8]) -> Result<()> {
        let hex = utils::hex(sha1);
        let settings = b2::UploadSettings {
            filename,
            content_type: "text/plain",
            sha1: Some(&hex),
        };

        let upload = self.b2.get_upload_url().await?;
        self.b2.upload(&settings, data, upload).await.map(|_| ())
    }

    pub async fn put_async(
        self,
        ctx: &worker::Context,
        filename: String,
        sha1: &[u8],
        mut data: Vec<u8>,
    ) -> Result<()> {
        let upload = self.b2.get_upload_url().await?;

        let hex = utils::hex(sha1);
        let future = async move {
            let settings = b2::UploadSettings {
                filename: &filename,
                content_type: "text/plain",
                sha1: Some(&hex),
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

    pub async fn put(&self, filename: &str, _sha1: &[u8], data: &mut [u8]) -> Result<()> {
        self.kv.put_bytes(filename, data)?.execute().await?;
        Ok(())
    }

    pub async fn put_async(
        self,
        ctx: &worker::Context,
        filename: String,
        _sha1: &[u8],
        data: Vec<u8>,
    ) -> Result<()> {
        let future = async move {
            let r = self.kv.put_bytes(&filename, &data).unwrap().execute().await;

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
}
