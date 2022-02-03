use crate::{
    consts, crypto,
    retry::{retry, Retry},
    utils,
    utils::hex,
    Error, Result,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use std::borrow::Cow;
use worker::{
    kv::KvStore, wasm_bindgen::JsValue, Env, Fetch, Headers, Method, Request, RequestInit,
};

#[allow(dead_code)]
pub async fn get(env: &Env, path: &str) -> Result<Option<worker::Response>> {
    let b2 = B2::from_env(env)?;
    let response = b2.download(path).await?;

    match response.status_code() {
        200 => Ok(Some(response)),
        404 => Ok(None),
        status => Err(Error::RemoteFailed(
            status,
            "failed to get paste".to_owned(),
        )),
    }
}

#[allow(dead_code)]
pub async fn put(env: &Env, filename: &str, sha1: &[u8], data: &mut [u8]) -> Result<()> {
    let b2 = B2::from_env(env)?;

    let hex = utils::hex(sha1);
    let settings = UploadSettings {
        filename,
        content_type: "text/plain",
        sha1: Some(&hex),
    };

    b2.upload(&settings, data).await.map(|_| ())
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthDetails {
    pub account_id: String,
    pub allowed: Bucket,
    pub api_url: String,
    pub authorization_token: String,
    pub download_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Bucket {
    pub bucket_id: String,
    pub bucket_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct UploadDetails {
    pub authorization_token: String,
    pub bucket_id: String,
    pub upload_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadResponse {
    pub account_id: String,
    pub action: String,
    pub bucket_id: String,
    pub content_length: u32,
    pub content_sha1: String,
    pub content_type: String,
    pub file_id: String,
    pub file_name: String,
}

#[derive(Clone, Debug, Default)]
pub struct UploadSettings<'a> {
    pub filename: &'a str,
    pub content_type: &'a str,
    pub sha1: Option<&'a str>,
}

struct Cache<T>
where
    T: DeserializeOwned,
    T: Serialize,
{
    kv: KvStore,
    size: usize,
    _type: std::marker::PhantomData<T>,
}

impl<T> Cache<T>
where
    T: DeserializeOwned,
    T: Serialize,
{
    fn new(kv: KvStore, size: usize) -> Self {
        Self {
            kv,
            size,
            _type: std::marker::PhantomData::default(),
        }
    }

    fn random_key(&self) -> String {
        let rand = worker::Date::now().as_millis() as usize % self.size;
        rand.to_string()
    }

    async fn get(&self, key: &str) -> Result<Option<T>> {
        Ok(self.kv.get(key).json().await?)
    }

    async fn put(&self, key: &str, value: T) -> Result<()> {
        Ok(self
            .kv
            .put(key, value)?
            .expiration_ttl(12 * 3600)
            .execute()
            .await?)
    }
}

const AUTH_DETAILS_URL: &str = "https://api.backblazeb2.com/b2api/v2/b2_authorize_account";

macro_rules! retry_if {
    ($response:ident, $err:expr, $($status:literal)|+, $map:expr) => {{
        let status_code = $response.status_code();
        if $($status == status_code)||+ {
            log::info!("request failed {}, retrying '{}' if more attempts are available", status_code, $err);
            Retry::err(Error::RemoteFailed(status_code, $err.into()))
        } else if status_code >= 300 {
            log::info!("request failed {}, not retrying '{}'", status_code, $err);
            Err(Error::RemoteFailed(status_code, $err.into()))
        } else {
            Retry::ok($map)
        }
    }};
}

pub struct B2 {
    credentials: Credentials,
    public_file_url: String,
    upload_url_cache: Cache<UploadDetails>,
}

impl B2 {
    pub fn from_env(env: &Env) -> Result<Self> {
        Ok(Self {
            credentials: Credentials::from_env(env)?,
            public_file_url: env.var(consts::ENV_B2_PUBLIC_FILE_URL)?.to_string(),
            upload_url_cache: Cache::new(env.kv(consts::KV_B2_UPLOAD_URLS)?, 20),
        })
    }

    pub async fn upload(
        &self,
        settings: &UploadSettings<'_>,
        content: &mut [u8],
    ) -> Result<UploadResponse> {
        let sha1 = match settings.sha1 {
            Some(sha1) => Cow::Borrowed(sha1),
            None => Cow::Owned(hex(&crypto::sha1(content).await?)),
        };

        // TODO: refactor this into a better cache api
        retry(5, |_| async {
            let key = self.upload_url_cache.random_key();
            let cached = self.upload_url_cache.get(&key).await?;

            let mut is_using_cached = cached.is_some();
            let mut upload = Some(match cached {
                Some(upload) => {
                    log::debug!("using cached upload url for key {}", key);
                    upload
                }
                None => {
                    log::debug!("no upload url cached for key {}", key);
                    self.get_upload_url().await?
                }
            });

            let mut r = loop {
                let r =
                    Self::do_upload(settings, upload.as_ref().unwrap(), &content, &sha1).await?;

                if !is_using_cached && r.status_code() == 200 {
                    log::info!("new upload url key {}", key);
                    let _ = self
                        .upload_url_cache
                        .put(&key, upload.take().unwrap())
                        .await;
                    break r;
                } else if is_using_cached && r.status_code() == 400 {
                    log::info!("key in use {}", key);
                    upload = Some(self.get_upload_url().await?);
                } else if is_using_cached && r.status_code() != 200 {
                    log::info!("key is broken {}", key);
                    is_using_cached = false;
                    upload = Some(self.get_upload_url().await?);
                } else {
                    break r;
                }
            };

            retry_if!(r, "upload", 401 | 503, r.json().await?)
        })
        .await
    }

    async fn do_upload(
        settings: &UploadSettings<'_>,
        upload: &UploadDetails,
        content: &&mut [u8],
        sha1: &str,
    ) -> Result<worker::Response> {
        let mut headers = Headers::new();
        headers.set("Authorization", &upload.authorization_token)?;
        headers.set("X-Bz-File-Name", settings.filename)?;
        headers.set("Content-Type", settings.content_type)?;
        headers.set("X-Bz-Content-Sha1", sha1)?;

        let request = Request::new_with_init(
            &upload.upload_url,
            &RequestInit {
                method: Method::Post,
                headers,
                body: Some(unsafe { js_sys::Uint8Array::view(content) }.into()),
                ..Default::default()
            },
        )?;

        Ok(Fetch::Request(request).send().await?)
    }

    pub async fn download(&self, path: &str) -> Result<worker::Response> {
        // Requires a public url for now ...
        let mut url = self.public_file_url.to_owned();
        url.push('/');
        url.push_str(path);

        retry(3, |_| async {
            let request = Request::new(&url, Method::Get)?;
            let response = Fetch::Request(request).send().await?;
            if response.status_code() >= 500 {
                log::info!("download failed {}", response.status_code());
                Retry::err(Error::RemoteFailed(response.status_code(), "upload".into()))
            } else {
                Retry::ok(response)
            }
        })
        .await
    }

    async fn get_upload_url(&self) -> Result<UploadDetails> {
        // Retry once just in case the credentials expired and on the 2nd attempt force new
        // credentials.
        retry(2, |attempt| async move {
            let auth = self.credentials.get_auth_details(attempt > 1).await?;

            let mut url = auth.api_url.to_owned();
            url.push_str("/b2api/v2/b2_get_upload_url");

            let mut headers = Headers::new();
            headers.set("Authorization", &auth.authorization_token)?;

            let body = JsValue::from_str(&serde_json::to_string(
                &json!({"bucketId": auth.allowed.bucket_id}),
            )?);
            let request = Request::new_with_init(
                &url,
                &RequestInit {
                    method: Method::Post,
                    headers,
                    body: Some(body),
                    ..Default::default()
                },
            )?;

            let mut r = Fetch::Request(request).send().await?;
            retry_if!(r, "upload_url", 401 | 503, r.json().await?)
        })
        .await
    }
}

pub struct Credentials {
    kv: KvStore,
    key_id: String,
    application_key: String,
}

impl Credentials {
    pub fn from_env(env: &Env) -> Result<Self> {
        Ok(Self {
            kv: env.kv(consts::KV_B2_CREDENTIALS)?,
            key_id: env.var(consts::ENV_B2_KEY_ID)?.to_string(),
            application_key: env.var(consts::ENV_B2_APPLICATION_KEY)?.to_string(),
        })
    }

    async fn get_auth_details(&self, force_refresh: bool) -> Result<AuthDetails> {
        if !force_refresh {
            if let Some(auth_details) = self.kv.get("credentials").cache_ttl(3_600).json().await? {
                log::debug!("using cached auth details");
                return Ok(auth_details);
            }
        }

        log::info!(
            "--> requesting auth details{}",
            if force_refresh { ", forced" } else { "" }
        );
        let auth_details = self.get_new_auth_details().await?;
        log::info!("<-- got auth details");

        // TODO: maybe persist creation time with the key?
        self.kv
            .put("credentials", &auth_details)?
            .expiration_ttl(12 * 3_600)
            .execute()
            .await?;

        Ok(auth_details)
    }

    async fn get_new_auth_details(&self) -> Result<AuthDetails> {
        let mut headers = Headers::new();
        headers.set(
            "Authorization",
            &crate::utils::basic_auth(&self.key_id, &self.application_key)?,
        )?;

        let request = Request::new_with_init(
            AUTH_DETAILS_URL,
            &RequestInit {
                method: Method::Get,
                headers,
                ..Default::default()
            },
        )?;

        let mut r = Fetch::Request(request).send().await?;
        match r.status_code() {
            200 => Ok(r.json().await?),
            status => Err(Error::RemoteFailed(status, "auth_details".to_owned())),
        }
    }
}
