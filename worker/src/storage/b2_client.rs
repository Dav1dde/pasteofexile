use std::{borrow::Cow, collections::HashMap};

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};
use serde_json::json;
use worker::{kv::KvStore, Env};

use crate::{
    consts,
    crypto::sha1,
    net,
    retry::{retry, Retry},
    utils::hex,
    Error, Result,
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
            tracing::warn!(
                status_code, err = $err,
                "request failed {}, retrying '{}' if more attempts are available", status_code, $err
            );
            Retry::err(Error::RemoteFailed(status_code, $err.into()))
        } else if status_code >= 300 {
            tracing::warn!(status_code, err = $err, "request failed {}, not retrying '{}'", status_code, $err);
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

    #[tracing::instrument(skip(self, content), fields(size = content.len()))]
    pub async fn upload(
        &self,
        settings: &UploadSettings<'_>,
        content: &[u8],
    ) -> Result<UploadResponse> {
        let filename = utf8_percent_encode(settings.filename, NON_ALPHANUMERIC).to_string();
        let sha1 = match settings.sha1 {
            Some(sha1) => Cow::Borrowed(sha1),
            None => Cow::Owned(hex(&sha1(content).await?)),
        };

        retry(5, |_| async {
            let upload = self.get_upload_url().await?;

            let mut response = net::Request::post(&upload.upload_url)
                .header("Authorization", &upload.authorization_token)
                .header("X-Bz-File-Name", &filename)
                .header("Content-Type", settings.content_type)
                .header("X-Bz-Content-Sha1", &sha1)
                .header_opt("X-Bz-Info-Metadata", settings.metadata)
                .body_u8(content)
                .send()
                .await?;

            retry_if!(response, "upload", 401 | 503 | 521, response.json().await?)
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
            let response = net::Request::get(&url).send().await?;
            if response.status_code() >= 500 {
                tracing::info!(
                    status_code = response.status_code(),
                    "download failed {}",
                    response.status_code()
                );
                Retry::err(Error::RemoteFailed(response.status_code(), "upload".into()))
            } else {
                Retry::ok(response)
            }
        })
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn list_files(&self, prefix: &str, max_file_count: u16) -> Result<ListResponse> {
        retry(3, |_| async move {
            let auth = self.credentials.get_auth_details(false).await?;

            let mut url = auth.api_url.to_owned();
            url.push_str("/b2api/v2/b2_list_file_names");

            let mut r = net::Request::post(url)
                .header("Authorization", &auth.authorization_token)
                .body(serde_json::to_string(&json!({
                    "bucketId": auth.allowed.bucket_id,
                    "prefix": prefix,
                    "maxFileCount": max_file_count
                }))?)
                .send()
                .await?;

            retry_if!(r, "list_files", 401 | 503 | 521, r.json().await?)
        })
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn hide(&self, path: &str) -> Result<File> {
        retry(3, |_| async move {
            let auth = self.credentials.get_auth_details(false).await?;

            let mut url = auth.api_url.to_owned();
            url.push_str("/b2api/v2/b2_hide_file");

            let mut r = net::Request::post(url)
                .header("Authorization", &auth.authorization_token)
                .body(serde_json::to_string(&json!({
                    "bucketId": auth.allowed.bucket_id,
                    "fileName": path
                }))?)
                .send()
                .await?;

            retry_if!(r, "list_files", 401 | 503 | 521, r.json().await?)
        })
        .await
    }

    #[tracing::instrument(skip(self))]
    async fn get_upload_url(&self) -> Result<UploadDetails> {
        // Retry in case the credentials expired and on the 2nd attempt force new credentials.
        retry(5, |attempt| async move {
            let auth = self.credentials.get_auth_details(attempt > 2).await?;

            let mut url = auth.api_url.to_owned();
            url.push_str("/b2api/v2/b2_get_upload_url");

            let mut r = net::Request::post(url)
                .header("Authorization", &auth.authorization_token)
                .body(serde_json::to_string(&json!({
                    "bucketId": auth.allowed.bucket_id
                }))?)
                .send()
                .await?;

            retry_if!(r, "upload_url", 401 | 503 | 521, r.json().await?)
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

        tracing::info!(force_refresh, "requesting auth details");
        let auth_details = self.get_new_auth_details().await?;
        tracing::info!("got new auth detauls");

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
        let authorization = crate::utils::basic_auth(&self.key_id, &self.application_key)?;

        let mut response = net::Request::get(AUTH_DETAILS_URL)
            .header("Authorization", &authorization)
            .send()
            .await?;

        match response.status_code() {
            200 => Ok(response.json().await?),
            status => Err(Error::RemoteFailed(status, "auth_details".to_owned())),
        }
    }
}
