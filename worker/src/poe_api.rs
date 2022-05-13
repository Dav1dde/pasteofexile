use std::borrow::Cow;
use worker::Url;

const OAUTH_AUTHORIZE_URL: &str = "https://www.pathofexile.com/oauth/authorize";

pub struct AuthorizationGrant<'a> {
    pub code: Cow<'a, str>,
    pub state: Cow<'a, str>,
}

impl<'a> TryFrom<&'a Url> for AuthorizationGrant<'a> {
    type Error = ();

    fn try_from(value: &'a Url) -> Result<Self, Self::Error> {
        let mut code = None;
        let mut state = None;
        for (k, v) in value.query_pairs() {
            match &*k {
                "code" => code = Some(v),
                "state" => state = Some(v),
                _ => {}
            }
        }

        code.zip(state)
            .map(|(code, state)| Self { code, state })
            .ok_or(())
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

impl Oauth {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
        }
    }

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
    pub async fn fetch_token(&self, code: &str) -> crate::Result<OauthToken> {
        let payload = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("client_id", &self.client_id)
            .append_pair("client_secret", &self.client_secret)
            .append_pair("grant_type", "authorization_code")
            .append_pair("code", code)
            .finish();

        let mut headers = worker::Headers::new();
        headers.set("Content-Type", "application/x-www-form-urlencoded")?;

        let request = worker::Request::new_with_init(
            "https://www.pathofexile.com/oauth/token",
            &worker::RequestInit {
                method: worker::Method::Post,
                headers,
                body: Some(wasm_bindgen::JsValue::from_str(&payload)),
                ..Default::default()
            },
        )?;

        Ok(worker::Fetch::Request(request).send().await?.json().await?)
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
    pub async fn fetch_profile(&self) -> crate::Result<Profile> {
        let mut headers = worker::Headers::new();
        headers.set("Authorization", &format!("Bearer {}", self.access_token))?;
        headers.set("User-Agent", "OAuth pobbin/1.0 (contact: TODO)")?; // TODO

        let request = worker::Request::new_with_init(
            "https://api.pathofexile.com/profile",
            &worker::RequestInit {
                method: worker::Method::Get,
                headers,
                ..Default::default()
            },
        )?;

        Ok(worker::Fetch::Request(request).send().await?.json().await?)
    }
}
