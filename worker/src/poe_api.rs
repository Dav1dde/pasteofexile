use std::borrow::Cow;

use serde::Deserialize;
use worker::Url;

use crate::{
    net,
    request_context::{Env, FromEnv},
};

const OAUTH_AUTHORIZE_URL: &str = "https://www.pathofexile.com/oauth/authorize";
const POE_API_USER_AGENT: &str = "OAuth pobbin/1.0 (contact: ggg@pobb.in)";

pub struct AuthorizationGrant<'a> {
    pub code: Cow<'a, str>,
    pub state: Cow<'a, str>,
}

pub enum AuthorizationGrantParseError<'a> {
    UserDeniedAccess(Cow<'a, str>),
    MissingAuthorizationGrant,
    Error {
        name: Cow<'a, str>,
        description: Option<Cow<'a, str>>,
    },
}

impl<'a> TryFrom<&'a Url> for AuthorizationGrant<'a> {
    type Error = AuthorizationGrantParseError<'a>;

    fn try_from(value: &'a Url) -> Result<Self, Self::Error> {
        let mut code = None;
        let mut state = None;
        let mut error = None;
        let mut error_description = None;
        for (k, v) in value.query_pairs() {
            match &*k {
                "code" => code = Some(v),
                "state" => state = Some(v),
                "error" => error = Some(v),
                "error_description" => error_description = Some(v),
                _ => {}
            }
        }

        #[allow(clippy::unnecessary_unwrap)]
        if error.as_deref() == Some("access_denied") && state.is_some() {
            return Err(Self::Error::UserDeniedAccess(state.unwrap()));
        }

        if let Some(name) = error {
            return Err(Self::Error::Error {
                name,
                description: error_description,
            });
        }

        code.zip(state)
            .map(|(code, state)| Self { code, state })
            .ok_or(Self::Error::MissingAuthorizationGrant)
    }
}

#[derive(serde::Deserialize)]
pub struct OauthToken {
    pub access_token: String,
}

pub struct Oauth {
    pub client_id: String,
    pub client_secret: String,
}

impl FromEnv for Oauth {
    fn from_env(env: &Env) -> Option<Self> {
        Some(Self::new(
            env.var(crate::consts::ENV_OAUTH_CLIENT_ID)?,
            env.var(crate::consts::ENV_OAUTH_CLIENT_SECRET)?,
        ))
    }
}

impl Oauth {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
        }
    }

    #[tracing::instrument(skip(self, redirect_uri))]
    pub fn get_login_url(&self, redirect_uri: &str, state: &str, scope: &str) -> String {
        let params = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("client_id", &self.client_id)
            .append_pair("response_type", "code")
            .append_pair("scope", scope)
            .append_pair("state", state)
            .append_pair("redirect_uri", redirect_uri)
            .append_pair("prompt", "consent")
            .finish();

        format!("{OAUTH_AUTHORIZE_URL}?{params}")
    }

    #[tracing::instrument(skip(self, code))]
    pub async fn fetch_token(&self, code: &str) -> crate::Result<OauthToken> {
        let payload = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("client_id", &self.client_id)
            .append_pair("client_secret", &self.client_secret)
            .append_pair("grant_type", "authorization_code")
            .append_pair("code", code)
            .finish();

        let mut response = net::Request::post("https://www.pathofexile.com/oauth/token")
            .tag("poe_token")
            .header("User-Agent", POE_API_USER_AGENT)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(&payload)
            .send()
            .await?;

        if response.status_code() != 200 {
            return Err(handle_error("Token", response).await.into());
        }

        Ok(response.json().await?)
    }
}

#[derive(serde::Deserialize)]
pub struct Profile {
    pub name: String,
}

pub struct PoeApi {
    access_token: String,
}

impl PoeApi {
    pub fn new(access_token: String) -> Self {
        Self { access_token }
    }

    #[tracing::instrument(skip(self))]
    pub async fn fetch_profile(&self) -> crate::Result<Profile> {
        let mut response = net::Request::get("https://api.pathofexile.com/profile")
            .tag("poe_profile")
            .header("Authorization", &format!("Bearer {}", self.access_token))
            .header("User-Agent", POE_API_USER_AGENT)
            .send()
            .await?;

        if response.status_code() != 200 {
            return Err(handle_error("Profile", response).await.into());
        }

        Ok(response.json().await?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PoEApiError {
    #[error("{call} failed with status code {status_code}. {error}: {description}")]
    Parsed {
        call: &'static str,
        error: String,
        description: String,
        status_code: u16,
    },
    #[error("{call} failed with status code {status_code}")]
    Unknown {
        call: &'static str,
        status_code: u16,
    },
}

async fn handle_error(name: &'static str, mut response: worker::Response) -> PoEApiError {
    tracing::warn!(
        "fetching {name} failed with status code {}",
        response.status_code()
    );

    let mut error = PoEApiError::Unknown {
        call: name,
        status_code: response.status_code(),
    };

    let content = match response.text().await {
        Ok(content) => content,
        Err(err) => {
            tracing::warn!("failed to capture response for {name}: {err:?}");
            return error;
        }
    };

    let content_type = response
        .headers()
        .get("content-type")
        .ok()
        .flatten()
        .map(Cow::Owned);

    if content_type.as_deref() == Some("application/json") {
        #[derive(Deserialize)]
        struct E {
            error: String,
            #[serde(default)]
            error_description: String,
        }
        if let Ok(json_error) = serde_json::from_str::<E>(&content) {
            error = PoEApiError::Parsed {
                call: name,
                error: json_error.error,
                description: json_error.error_description,
                status_code: response.status_code(),
            }
        }
    };

    let data = content.into_bytes().into();
    sentry::add_attachment(data, content_type, name);

    error
}
