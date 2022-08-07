use crate::{
    consts,
    crypto::sha1,
    retry::{retry, Retry},
    utils::hex,
    Error, Result,
};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{borrow::Cow, collections::HashMap};
use worker::{
    kv::KvStore, wasm_bindgen::JsValue, Env, Fetch, Headers, Method, Request, RequestInit,
};

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
pub struct UploadDetails {
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResponse {
    pub files: Vec<File>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub file_name: String,
    pub file_info: HashMap<String, String>,
    pub upload_timestamp: u64,
}

#[derive(Clone, Debug, Default)]
pub struct UploadSettings<'a> {
    pub filename: &'a str,
    pub content_type: &'a str,
    pub sha1: Option<&'a str>,
    pub metadata: Option<&'a str>,
}

const AUTH_DETAILS_URL: &str = "https://api.backblazeb2.com/b2api/v2/b2_authorize_account";

macro_rules! retry_if {
    ($response:ident, $err:expr, $($status:literal)|+, $map:expr) => {{
        let status_code = $response.status_code();
        if $($status == status_code)||+ {
            tracing::info!("request failed {}, retrying '{}' if more attempts are available", status_code, $err);
            Retry::err(Error::RemoteFailed(status_code, $err.into()))
        } else if status_code >= 300 {
            tracing::info!("request failed {}, not retrying '{}'", status_code, $err);
            Err(Error::RemoteFailed(status_code, $err.into()))
        } else {
            Retry::ok($map)
        }
    }};
}

pub struct B2 {
    credentials: Credentials,
    public_file_url: String,
}

impl B2 {
    pub fn from_env(env: &Env) -> Result<Self> {
        Ok(Self {
            credentials: Credentials::from_env(env)?,
            public_file_url: env.var(consts::ENV_B2_PUBLIC_FILE_URL)?.to_string(),
        })
    }

    #[tracing::instrument(skip(self, content, upload), fields(size = content.len()))]
    pub async fn upload(
        &self,
        settings: &UploadSettings<'_>,
        content: &mut [u8],
        upload: UploadDetails,
    ) -> Result<UploadResponse> {
        let filename = utf8_percent_encode(settings.filename, NON_ALPHANUMERIC).to_string();
        let sha1 = match settings.sha1 {
            Some(sha1) => Cow::Borrowed(sha1),
            None => Cow::Owned(hex(&sha1(content).await?)),
        };

        retry(5, |_| async {
            let mut headers = Headers::new();
            headers.set("Authorization", &upload.authorization_token)?;
            headers.set("X-Bz-File-Name", &filename)?;
            headers.set("Content-Type", settings.content_type)?;
            headers.set("X-Bz-Content-Sha1", &sha1)?;
            if let Some(metadata) = settings.metadata {
                headers.set("X-Bz-Info-Metadata", metadata)?;
            }

            let request = Request::new_with_init(
                &upload.upload_url,
                &RequestInit {
                    method: Method::Post,
                    headers,
                    body: Some(unsafe { js_sys::Uint8Array::view(content) }.into()),
                    ..Default::default()
                },
            )?;

            let mut r = Fetch::Request(request).send().await?;
            retry_if!(r, "upload", 401 | 503, r.json().await?)
        })
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn download(&self, path: &str) -> Result<worker::Response> {
        // Requires a public url for now ...
        let mut url = self.public_file_url.to_owned();
        url.push('/');
        url.push_str(path);

        retry(3, |_| async {
            let request = Request::new(&url, Method::Get)?;
            let response = Fetch::Request(request).send().await?;
            if response.status_code() >= 500 {
                tracing::info!("download failed {}", response.status_code());
                Retry::err(Error::RemoteFailed(response.status_code(), "upload".into()))
            } else {
                Retry::ok(response)
            }
        })
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_upload_url(&self) -> Result<UploadDetails> {
        // Retry in case the credentials expired and on the 2nd attempt force new credentials.
        retry(5, |attempt| async move {
            let auth = self.credentials.get_auth_details(attempt > 2).await?;

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

    #[tracing::instrument(skip(self))]
    pub async fn list_files(&self, prefix: &str, max_file_count: u16) -> Result<ListResponse> {
        retry(3, |_| async move {
            let auth = self.credentials.get_auth_details(false).await?;

            let mut url = auth.api_url.to_owned();
            url.push_str("/b2api/v2/b2_list_file_names");

            let mut headers = Headers::new();
            headers.set("Authorization", &auth.authorization_token)?;

            let body = JsValue::from_str(&serde_json::to_string(
                &json!({"bucketId": auth.allowed.bucket_id, "prefix": prefix, "maxFileCount": max_file_count}),
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
            retry_if!(r, "list_files", 401 | 503, r.json().await?)
        })
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn hide(&self, path: &str) -> Result<File> {
        retry(3, |_| async move {
            let auth = self.credentials.get_auth_details(false).await?;

            let mut url = auth.api_url.to_owned();
            url.push_str("/b2api/v2/b2_hide_file");

            let mut headers = Headers::new();
            headers.set("Authorization", &auth.authorization_token)?;

            let body = JsValue::from_str(&serde_json::to_string(
                &json!({"bucketId": auth.allowed.bucket_id, "fileName": path}),
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
            retry_if!(r, "list_files", 401 | 503, r.json().await?)
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

    #[tracing::instrument(skip(self))]
    async fn get_auth_details(&self, force_refresh: bool) -> Result<AuthDetails> {
        if !force_refresh {
            if let Some(auth_details) = self.kv.get("credentials").cache_ttl(3_600).json().await? {
                tracing::debug!("using cached auth details");
                return Ok(auth_details);
            }
        }

        tracing::info!(
            "--> requesting auth details{}",
            if force_refresh { ", forced" } else { "" }
        );
        let auth_details = self.get_new_auth_details().await?;
        tracing::info!("<-- got auth details");

        // TODO: maybe persist creation time with the key?
        self.kv
            .put("credentials", &auth_details)?
            .expiration_ttl(12 * 3_600)
            .execute()
            .await?;

        Ok(auth_details)
    }

    #[tracing::instrument(skip(self))]
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
