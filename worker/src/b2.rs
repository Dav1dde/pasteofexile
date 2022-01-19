use crate::utils::hex;
use crate::{consts, crypto::sha1};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::borrow::Cow;
use worker::kv::KvStore;
use worker::{wasm_bindgen::JsValue, Env, Fetch, Headers, Method, Request, RequestInit, Result};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthDetails {
    pub account_id: String,
    pub allowed: Bucket,
    pub api_url: String,
    pub authorization_token: String,
    pub download_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bucket {
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

#[derive(Clone, Debug, Default)]
pub struct UploadSettings<'a> {
    pub filename: &'a str,
    pub content_type: &'a str,
    pub sha1: Option<&'a str>,
}

const AUTH_DETAILS_URL: &str = "https://api.backblazeb2.com/b2api/v2/b2_authorize_account";

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

    pub async fn upload(
        &self,
        settings: &UploadSettings<'_>,
        content: &mut [u8],
    ) -> Result<UploadResponse> {
        // TODO: retry with new upload url up to 5 times
        log::debug!("--> getting upload url");
        let upload = self.get_upload_url().await?;
        log::debug!("<-- got upload url");

        let sha1 = match settings.sha1 {
            Some(sha1) => Cow::Borrowed(sha1),
            None => Cow::Owned(hex(&sha1(content).await?)),
        };

        let mut headers = Headers::new();
        headers.set("Authorization", &upload.authorization_token)?;
        headers.set("X-Bz-File-Name", settings.filename)?;
        headers.set("Content-Type", settings.content_type)?;
        headers.set("X-Bz-Content-Sha1", &sha1)?;

        let request = Request::new_with_init(
            &upload.upload_url,
            &RequestInit {
                method: Method::Post,
                headers,
                body: Some(unsafe { js_sys::Uint8Array::view(content) }.into()),
                ..Default::default()
            },
        )?;

        log::debug!("--> uploading file");
        // TODO: check for 200
        let r = Fetch::Request(request).send().await?.json().await?;
        log::debug!("<-- file uploaded");
        Ok(r)
    }

    pub async fn download(&self, path: &str) -> Result<worker::Response> {
        let mut url = self.public_file_url.to_owned();
        url.push('/');
        url.push_str(path);

        let request = Request::new(&url, Method::Get)?;
        Ok(Fetch::Request(request).send().await?)
    }

    async fn get_upload_url(&self) -> Result<UploadDetails> {
        // TODO: handle expired credentials
        let auth = self.credentials.get_auth_details().await?;

        let mut url = auth.api_url.to_owned();
        url.push_str("/b2api/v2/b2_get_upload_url");

        let mut headers = Headers::new();
        headers.set("Authorization", &auth.authorization_token)?;

        let request = Request::new_with_init(
            &url,
            &RequestInit {
                method: Method::Post,
                headers,
                body: Some(JsValue::from_str(&serde_json::to_string(
                    &json!({"bucketId": auth.allowed.bucket_id}),
                )?)),
                ..Default::default()
            },
        )?;
        // TODO: check for 200
        Ok(Fetch::Request(request).send().await?.json().await?)
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

    pub async fn get_auth_details(&self) -> Result<AuthDetails> {
        // TODO: handle expired credentials
        if let Some(auth_details) = self.kv.get("credentials").cache_ttl(3_600).json().await? {
            log::debug!("using cached auth details");
            return Ok(auth_details);
        }

        log::info!("requesting auth details");
        let auth_details = self.get_new_auth_details().await?;
        log::debug!("got auth details");

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
        // TODO: check for 200
        Ok(Fetch::Request(request).send().await?.json().await?)
    }
}
