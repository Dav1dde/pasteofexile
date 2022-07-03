use crate::{
    consts, crypto, poe_api,
    utils::{self, is_valid_id, EnvExt, RequestExt, ResponseExt},
    Error, Result,
};
use pob::{PathOfBuilding, PathOfBuildingExt, SerdePathOfBuilding};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use sycamore_router::Route;
use worker::{Context, Env, Headers, Method, Request, Response};

macro_rules! validate {
    ($e:expr, $msg:expr) => {
        if !$e {
            return Err(Error::BadRequest($msg.into()));
        }
    };
}

macro_rules! validate_access {
    ($e:expr) => {
        if !$e {
            return Err(Error::AccessDenied);
        }
    };
}

#[derive(sycamore_router::Route)]
enum GetEndpoints {
    #[to("/api/internal/user/<user>")]
    User(String),
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

#[derive(sycamore_router::Route)]
enum PostEndpoints {
    #[to("/api/internal/paste/")]
    Upload(),
    #[to("/pob/")]
    PobUpload(),
    #[not_found]
    NotFound,
}

#[derive(sycamore_router::Route)]
enum DeleteEndpoints {
    #[to("/api/internal/paste/<id>")]
    DeletePaste(PasteId),
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
            GetEndpoints::User(user) => handle_user(env, user).await.map(Some),
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
    } else if req.method() == Method::Delete {
        match DeleteEndpoints::match_path(&req.path()) {
            DeleteEndpoints::DeletePaste(id) => handle_delete_paste(env, id).await.map(Some),
            DeleteEndpoints::NotFound => Ok(None),
        }
    } else {
        Ok(None)
    }
}

// TODO: use app::model::PasteId
#[derive(Deserialize)]
#[serde(untagged)]
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

    pub fn user(&self) -> Option<&str> {
        match self {
            Self::UserPaste(user, _) => Some(user),
            _ => None,
        }
    }

    pub fn id(&self) -> &str {
        match self {
            Self::UserPaste(_, id) => id,
            Self::Paste(id) => id,
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

impl FromStr for PasteId {
    type Err = crate::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let r = s
            .split_once(':')
            .map(|(user, id)| Self::UserPaste(user.to_owned(), id.to_owned()))
            .unwrap_or_else(|| Self::Paste(s.to_owned()));
        Ok(r)
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

async fn handle_delete_paste(env: &Env, id: PasteId) -> Result<Response> {
    env.storage()?.delete(&id.to_path()?).await?;
    Ok(Response::empty()?)
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct PasteMetadata {
    pub title: String,
    pub ascendancy: Option<String>,
    pub version: Option<String>,
    pub main_skill_name: Option<String>,
    pub last_modified: u64,
}

impl PasteMetadata {
    fn new(pob: &SerdePathOfBuilding) -> Self {
        Self {
            title: app::pob::title(pob),
            ascendancy: pob.ascendancy_name().map(String::from),
            version: pob.max_tree_version(),
            main_skill_name: pob.main_skill_name().map(|x| x.to_owned()),
            last_modified: worker::Date::now().as_millis(),
        }
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
    id: Option<PasteId>,
    #[serde(default)]
    as_user: bool,
    #[serde(default)]
    title: Option<String>,
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

        validate!(data.title.is_some(), "Title is required");
        let title = data.title.unwrap();
        validate!(title.len() < 90, "Title too long");
        validate!(title.len() > 5, "Title too short");

        metadata.title = title;

        if let Some(id) = data.id {
            validate_access!(Some(session.name.as_str()) == id.user());
            validate!(is_valid_id(id.id()), "Invalid id");

            id
        } else {
            PasteId::UserPaste(session.name, utils::random_string::<9>()?)
        }
    } else {
        validate_access!(data.id.is_none());
        // TODO: should unused fields (like title) be validated?
        // validate!(data.title.is_none(), "Cannot set title");

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

async fn handle_user(env: &Env, user: String) -> Result<Response> {
    let mut pastes = env
        .storage()?
        .list(format!("user/{user}/pastes/"))
        .await?
        .into_iter()
        .map(|item: crate::storage::ListItem<PasteMetadata>| {
            let metadata = item.metadata.unwrap_or_default();
            let id = item.name.rsplit_once('/').unwrap().1.to_owned();

            // TODO: properly do this
            // TODO: code duplication with lib.rs
            app::model::PasteSummary {
                id,
                user: Some(user.clone()),
                title: metadata.title,
                ascendancy: metadata.ascendancy.unwrap_or_default(),
                version: metadata.version.unwrap_or_default(),
                main_skill_name: metadata.main_skill_name.unwrap_or_default(),
                last_modified: metadata.last_modified,
            }
        })
        .collect::<Vec<_>>();
    pastes.sort_unstable_by(|a, b| b.last_modified.cmp(&a.last_modified));

    // TODO: caching
    Ok(Response::from_json(&pastes)?)
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
