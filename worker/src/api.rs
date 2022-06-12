use crate::{
    consts, crypto, poe_api,
    storage::Metadata,
    utils::{self, EnvExt, RequestExt, ResponseExt},
    Error, Result,
};
use pob::{PathOfBuilding, SerdePathOfBuilding};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap, fmt};
use sycamore_router::Route;
use worker::{Context, Env, Headers, Method, Request, Response};

macro_rules! validate {
    ($e:expr, $msg:expr) => {
        if !$e {
            return Err(Error::BadRequest($msg.into()));
        }
    };
}

#[derive(Clone, Debug, PartialEq, Eq, sycamore_router::Route)]
enum GetEndpoints {
    #[to("/<id>/raw")]
    Paste(String),
    #[to("/u/<name>/<id>/raw")]
    UserPaste(String, String),
    #[to("/pob/<id>")]
    PobPaste(String),
    #[to("/login")]
    Login(),
    #[to("/oauth2/authorization/poe")]
    Oauht2Poe(),
    #[not_found]
    NotFound,
}

#[derive(Clone, Debug, PartialEq, Eq, sycamore_router::Route)]
enum PostEndpoints {
    #[to("/api/internal/paste/")]
    Upload(),
    #[to("/pob/")]
    PobUpload(),
    #[not_found]
    NotFound,
}

pub async fn try_handle(ctx: &Context, req: &mut Request, env: &Env) -> Result<Option<Response>> {
    // TODO: some of these need to render a error page not fallback to JSON (e.g. failed login)

    if req.method() == Method::Post {
        match PostEndpoints::match_path(&req.path()) {
            PostEndpoints::Upload() => handle_upload(ctx, req, env).await.map(Some),
            PostEndpoints::PobUpload() => handle_pob_upload(ctx, req, env).await.map(Some),
            PostEndpoints::NotFound => Ok(None),
        }
    } else if req.method() == Method::Get {
        match GetEndpoints::match_path(&req.path()) {
            GetEndpoints::Paste(id) | GetEndpoints::PobPaste(id) => {
                handle_download(env, PasteId::Paste(id)).await.map(Some)
            }
            GetEndpoints::UserPaste(user, id) => handle_download(env, PasteId::UserPaste(user, id))
                .await
                .map(Some),
            GetEndpoints::Login() => handle_login(req, env).await.map(Some),
            GetEndpoints::Oauht2Poe() => handle_oauth2_poe(req, env).await.map(Some),
            GetEndpoints::NotFound => Ok(None),
        }
    } else {
        Ok(None)
    }
}

pub enum PasteId {
    Paste(String),
    UserPaste(String, String),
}

impl PasteId {
    pub fn to_path(&self) -> Result<String> {
        match self {
            Self::Paste(id) => Ok(utils::to_path(id)?),
            Self::UserPaste(user, id) => Ok(format!("user/{user}/pastes/{id}")),
        }
    }

    pub fn unwrap_paste(self) -> String {
        match self {
            Self::Paste(id) => id,
            _ => panic!("unwrap_paste"),
        }
    }

    pub fn unwrap_user_paste(self) -> (String, String) {
        match self {
            Self::UserPaste(user, id) => (user, id),
            _ => panic!("unwrap_paste"),
        }
    }
}

impl fmt::Display for PasteId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Paste(id) => write!(f, "{id}"),
            Self::UserPaste(user, id) => write!(f, "{user}:{id}"),
        }
    }
}

async fn handle_download(env: &Env, id: PasteId) -> Result<Response> {
    let path = id.to_path()?;

    let response = env
        .storage()?
        .get(&path)
        .await?
        .ok_or_else(|| Error::NotFound("paste", id.to_string()))?;

    response
        .with_headers(Headers::new())
        .with_content_type("text/plain")?
        .cache_for(31536000)
}

#[derive(Debug)]
pub struct PasteMetadata {
    pub title: String,
    pub ascendancy: Option<String>,
    pub last_modified: u64,
}

impl PasteMetadata {
    fn new(pob: &SerdePathOfBuilding) -> Self {
        Self {
            title: app::pob::title(pob),
            ascendancy: pob.ascendancy_name().map(String::from),
            last_modified: worker::Date::now().as_millis(),
        }
    }
}

impl Metadata for PasteMetadata {
    fn from_key_value(mut kv: HashMap<String, String>) -> Option<Self> {
        let title = kv.remove("title")?;
        let ascendancy = kv.remove("ascendancy");
        let last_modified = kv
            .remove("last_modified")
            .and_then(|x| x.parse().ok())
            .unwrap_or(0);
        Some(Self {
            title,
            ascendancy,
            last_modified,
        })
    }

    fn as_key_value(&self) -> HashMap<&str, Cow<'_, str>> {
        let mut kv = HashMap::new();
        kv.insert("title", self.title.as_str().into());
        if let Some(ref ascendancy) = self.ascendancy {
            kv.insert("ascendancy", ascendancy.into());
        }
        kv.insert("last_modified", self.last_modified.to_string().into());
        kv
    }
}

#[derive(Serialize)]
struct UploadResponse {
    id: String,
    user: Option<String>,
}

impl UploadResponse {
    fn new(id: PasteId) -> Self {
        match id {
            PasteId::Paste(id) => Self { id, user: None },
            PasteId::UserPaste(user, id) => Self {
                id,
                user: Some(user),
            },
        }
    }
}

#[derive(Deserialize)]
struct UploadRequest {
    #[serde(default)]
    as_user: bool,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    slug: Option<String>, // TODO: maybe rename this to id
    content: String,
}

async fn handle_upload(_ctx: &Context, req: &mut Request, env: &Env) -> Result<Response> {
    let data = req.json::<UploadRequest>().await?;
    let mut content = data.content.into_bytes();

    let pob = validate_pob(&content)?;
    let mut metadata = PasteMetadata::new(&pob);

    let sha1 = crypto::sha1(&mut content).await?;

    let id = if data.as_user {
        let session = req.session().ok_or(Error::AccessDenied)?;
        let session = env.dangerous()?.verify::<app::User>(&session).await?;

        // TODO slug (or id) should be required
        validate!(data.slug.is_none(), "Not implemented");

        validate!(data.title.is_some(), "Title is required");
        let title = data.title.unwrap();
        validate!(title.len() < 50, "Title too long");
        validate!(title.len() > 3, "Title too short");

        metadata.title = title;

        PasteId::UserPaste(session.name, utils::random_string::<9>()?)
    } else {
        // TODO: validate here or just ignore?
        // validate!(data.title.is_none(), "Cannot set title");
        // validate!(data.slug.is_none(), "Cannot set slug");

        PasteId::Paste(utils::hash_to_short_id(&sha1, 9)?)
    };

    let filename = id.to_path()?;

    log::debug!("--> uploading paste '{}' to '{}'", id, filename);
    env.storage()?
        .put(&filename, &sha1, &mut content, Some(metadata))
        .await?;
    log::debug!("<-- paste uploaded");

    let response = serde_json::to_vec(&UploadResponse::new(id))?;
    let mut response = Response::from_bytes(response)?;
    response
        .headers_mut()
        .set("Content-Type", "application/json")?;
    Ok(response)
}

async fn handle_pob_upload(ctx: &Context, req: &mut Request, env: &Env) -> Result<Response> {
    let mut data = req.bytes().await?;

    let pob = validate_pob(&data)?;
    let metadata = PasteMetadata::new(&pob);

    let sha1 = crypto::sha1(&mut data).await?;
    let id = utils::hash_to_short_id(&sha1, 9)?;
    let filename = utils::to_path(&id)?;

    log::debug!("--> uploading paste '{}' to '{}'", id, filename);
    env.storage()?
        .put_async(ctx, filename, &sha1, data, Some(metadata))
        .await?;
    log::debug!("<-- paste uploaing ...");

    Ok(Response::ok(id)?)
}

fn validate_pob(data: &[u8]) -> Result<SerdePathOfBuilding> {
    if data.len() > consts::MAX_UPLOAD_SIZE {
        return Err(Error::BadRequest("Paste too large".to_owned()));
    }

    let s = std::str::from_utf8(data)
        .map_err(|_| "invalid content".to_owned())
        .map_err(Error::BadRequest)?;

    // Generic 401, probably just actually bad data
    let s = pob::decompress(s).map_err(|e| Error::BadRequest(e.to_string()))?;
    // More specific error for a separate Sentry categoy
    SerdePathOfBuilding::from_xml(&s).map_err(move |e| Error::InvalidPoB(e.to_string(), s))
}

async fn handle_login(req: &Request, env: &Env) -> Result<Response> {
    let _url = req.url()?;
    let host = crate::utils::if_debug!("preview.pobb.in", _url.host_str().unwrap());

    let state = utils::random_string::<16>()?;

    let redirect_uri = format!("https://{host}/oauth2/authorization/poe");
    let login_uri = env
        .oauth()?
        .get_login_url(&redirect_uri, &state, consts::OAUTH_SCOPE);

    Response::redirect2(&login_uri)?.with_state_cookie(&state)
}

async fn handle_oauth2_poe(req: &Request, env: &Env) -> Result<Response> {
    let url = req.url()?;

    let grant = match poe_api::AuthorizationGrant::try_from(&url) {
        Ok(grant) => grant,
        // TODO: render error page
        _ => return Ok(Response::error("no authorization grant", 403)?),
    };

    crate::utils::if_debug!({}, {
        use crate::utils::RequestExt;
        if req.cookie("state").unwrap_or_default() != grant.state {
            // TODO: render error page
            return Ok(Response::error("invalid session state", 403)?);
        }
    });

    // TODO: error handling -> show error page
    let token = env.oauth()?.fetch_token(&grant.code).await?;

    let profile = poe_api::PoeApi::new(token.access_token)
        .fetch_profile()
        .await?;

    let user = app::User { name: profile.name };
    let session = env.dangerous()?.sign(&user).await?;

    // TODO: redirect back to where the user actually came from
    Response::redirect2(&format!("/u/{}", user.name))?
        .with_delete_state_cookie()?
        .with_new_session(&session)
}
