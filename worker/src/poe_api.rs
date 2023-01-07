use std::borrow::Cow;

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

    // TODO: better errors
    #[tracing::instrument(skip(self, code))]
    pub async fn fetch_token(&self, code: &str) -> crate::Result<OauthToken> {
        let payload = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("client_id", &self.client_id)
            .append_pair("client_secret", &self.client_secret)
            .append_pair("grant_type", "authorization_code")
            .append_pair("code", code)
            .finish();

        let mut response = net::Request::post("https://www.pathofexile.com/oauth/token")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(&payload)
            .send()
            .await?;

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

    // TODO: better errors
    #[tracing::instrument(skip(self))]
    pub async fn fetch_profile(&self) -> crate::Result<Profile> {
        let mut response = net::Request::get("https://api.pathofexile.com/profile")
            .header("Authorization", &format!("Bearer {}", self.access_token))
            .header("User-Agent", POE_API_USER_AGENT)
            .send()
            .await?;

        Ok(response.json().await?)
    }
}
