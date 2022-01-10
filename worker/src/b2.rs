use crate::crypto::{hex, sha1};
use serde::{Deserialize, Serialize};
use serde_json::json;
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

#[derive(Clone, Debug)]
pub struct UploadSettings<'a> {
    pub file_name: &'a str,
    pub content_type: &'a str,
}

const AUTH_DETAILS_URL: &str = "https://api.backblazeb2.com/b2api/v2/b2_authorize_account";

pub struct B2 {
    key_id: String,
    application_key: String,
    public_file_url: String,
}

impl B2 {
    pub fn new(
        key_id: impl Into<String>,
        application_key: impl Into<String>,
        public_file_url: impl Into<String>,
    ) -> Self {
        Self {
            key_id: key_id.into(),
            application_key: application_key.into(),
            public_file_url: public_file_url.into(),
        }
    }

    pub fn from_env(env: &Env) -> Result<Self> {
        Ok(Self::new(
            env.var("B2_KEY_ID")?.to_string(),
            env.var("B2_APPLICATION_KEY")?.to_string(),
            env.var("B2_PUBLIC_FILE_URL")?.to_string(),
        ))
    }

    pub async fn get_auth_details(&self) -> Result<AuthDetails> {
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

    pub async fn get_upload_url(&self, auth: &AuthDetails) -> Result<UploadDetails> {
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

    pub async fn upload(
        &self,
        upload: &UploadDetails,
        settings: &UploadSettings<'_>,
        content: &mut [u8],
    ) -> Result<UploadResponse> {
        let sha1 = hex(&sha1(content).await?);

        let mut headers = Headers::new();
        headers.set("Authorization", &upload.authorization_token)?;
        headers.set("X-Bz-File-Name", settings.file_name)?;
        headers.set("Content-Type", settings.content_type)?;
        headers.set("X-Bz-Content-Sha1", &sha1)?;

        let request = Request::new_with_init(
            &upload.upload_url,
            &RequestInit {
                method: Method::Post,
                headers,
                body: Some(unsafe { worker::js_sys::Uint8Array::view(content) }.into()),
                ..Default::default()
            },
        )?;
        // TODO: check for 200
        Ok(Fetch::Request(request).send().await?.json().await?)
    }

    pub async fn download(&self, path: &str) -> Result<worker::Response> {
        let mut url = self.public_file_url.to_owned();
        url.push('/');
        url.push_str(path);

        let request = Request::new(&url, Method::Get)?;
        Ok(Fetch::Request(request).send().await?)
    }
}
