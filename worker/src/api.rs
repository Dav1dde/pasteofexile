use crate::{
    consts, crypto, poe_api,
    utils::{self, is_valid_id, CacheControl, EnvExt, RequestExt, ResponseExt},
    Error, Result,
};
use pob::{PathOfBuilding, PathOfBuildingExt, SerdePathOfBuilding};
use serde::Deserialize;
use shared::model::{PasteId, PasteMetadata};
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
    #[to("/<id>/json")]
    PasteJson(String),
    #[to("/u/<name>/<id>/json")]
    UserPasteJson(String, String),
    /// Path of Building endpoint for importing builds.
    /// This supports the anonymous and user scoped paste IDs.
    /// User scoped paste IDs are used in `pob://` protocol links.
    /// Anonymous paste IDs are coming from importing an anonymous build URL in PoB.
    #[to("/pob/<id>")]
    PobPaste(PasteId),
    /// Path of Building endpoint for importing user paste URLs.
    #[to("/pob/u/<name>/<id>")]
    PobUserPaste(String, String),
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
            GetEndpoints::PobPaste(id) => handle_download_text(env, id).await.map(Some),
            GetEndpoints::PobUserPaste(user, id) => {
                handle_download_text(env, PasteId::new_user(user, id))
                    .await
                    .map(Some)
            }
            GetEndpoints::Paste(id) => handle_download_text(env, PasteId::Paste(id))
                .await
                .map(Some),
            GetEndpoints::UserPaste(user, id) => {
                handle_download_text(env, PasteId::new_user(user, id))
                    .await
                    .map(Some)
            }
            GetEndpoints::PasteJson(id) => handle_download_json(env, PasteId::Paste(id))
                .await
                .map(Some),
            GetEndpoints::UserPasteJson(user, id) => {
                handle_download_json(env, PasteId::new_user(user, id))
                    .await
                    .map(Some)
            }
            GetEndpoints::Login() => handle_login(req, env).await.map(Some),
            GetEndpoints::Oauht2Poe() => handle_oauth2_poe(req, env).await.map(Some),
            GetEndpoints::NotFound => Ok(None),
        }
    } else if req.method() == Method::Delete {
        match DeleteEndpoints::match_path(&req.path()) {
            DeleteEndpoints::DeletePaste(id) => {
                handle_delete_paste(ctx, req, env, id).await.map(Some)
            }
            DeleteEndpoints::NotFound => Ok(None),
        }
    } else {
        Ok(None)
    }
}

async fn handle_download_text(env: &Env, id: PasteId) -> Result<Response> {
    let paste = env
        .storage()?
        .get(&id)
        .await?
        .ok_or_else(|| Error::NotFound("paste", id.to_string()))?;

    worker::Response::ok(paste.content)?
        .with_headers(Headers::new())
        .with_content_type("text/plain")?
        .with_etag_opt(paste.entity_id.as_deref())?
        .with_cache_control(
            CacheControl::default()
                .public()
                .s_max_age(consts::CACHE_FOREVER),
        )
}

async fn handle_download_json(env: &Env, id: PasteId) -> Result<Response> {
    let paste = env
        .storage()?
        .get(&id)
        .await?
        .ok_or_else(|| Error::NotFound("paste", id.to_string()))?;

    worker::Response::from_json(&paste)?
        .with_headers(Headers::new())
        .with_content_type("application/json")?
        .with_etag_opt(paste.entity_id.as_deref())?
        .with_cache_control(
            CacheControl::default()
                .public()
                .s_max_age(consts::CACHE_FOREVER),
        )
}

async fn handle_delete_paste(
    ctx: &Context,
    req: &Request,
    env: &Env,
    id: PasteId,
) -> Result<Response> {
    env.storage()?.delete(&id).await?;
    crate::cache::on_paste_change(ctx, req, id);
    Ok(Response::empty()?)
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

async fn handle_upload(ctx: &Context, req: &mut Request, env: &Env) -> Result<Response> {
    let data = req.json::<UploadRequest>().await?;
    let mut content = data.content.into_bytes();

    let pob = validate_pob(&content)?;
    let mut metadata = to_metadata(&pob);

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
            PasteId::new_user(session.name, utils::random_string::<9>()?)
        }
    } else {
        validate_access!(data.id.is_none());
        // TODO: should unused fields (like title) be validated?
        // validate!(data.title.is_none(), "Cannot set title");

        PasteId::Paste(utils::hash_to_short_id(&sha1, 9)?)
    };

    log::debug!("--> uploading paste '{}'", id);
    env.storage()?
        .put(&id, &sha1, &mut content, Some(metadata))
        .await?;
    log::debug!("<-- paste uploaded");

    let body = serde_json::to_vec(&id)?;

    crate::cache::on_paste_change(ctx, req, id);

    Response::from_bytes(body)?.with_content_type("application/json")
}

async fn handle_pob_upload(ctx: &Context, req: &mut Request, env: &Env) -> Result<Response> {
    let mut data = req.bytes().await?;

    let pob = validate_pob(&data)?;
    let metadata = to_metadata(&pob);

    let sha1 = crypto::sha1(&mut data).await?;
    let id = PasteId::new_id(utils::hash_to_short_id(&sha1, 9)?);

    log::debug!("--> uploading paste '{}'", id);
    env.storage()?
        .put_async(ctx, &id, &sha1, data, Some(metadata))
        .await?;
    log::debug!("<-- paste uploaing ...");

    let response = Response::ok(id.to_string())?;

    crate::cache::on_paste_change(ctx, req, id);

    Ok(response)
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

fn to_metadata(pob: &SerdePathOfBuilding) -> PasteMetadata {
    PasteMetadata {
        title: app::pob::title(pob),
        ascendancy: pob.ascendancy_name().map(String::from),
        version: pob.max_tree_version(),
        main_skill_name: pob.main_skill_name().map(|x| x.to_owned()),
    }
}

async fn handle_user(env: &Env, user: String) -> Result<Response> {
    // TODO: code duplication with lib.rs
    let mut pastes = env
        .storage()?
        .list(format!("user/{user}/pastes/"))
        .await?
        .into_iter()
        .map(|item| {
            let metadata = item.metadata.unwrap_or_default();
            let id = item.name.rsplit_once('/').unwrap().1.to_owned();

            // TODO: properly do this
            // TODO: code duplication with lib.rs
            shared::model::PasteSummary {
                id,
                user: Some(user.clone()),
                title: metadata.title,
                ascendancy: metadata.ascendancy.unwrap_or_default(),
                version: metadata.version.unwrap_or_default(),
                main_skill_name: metadata.main_skill_name.unwrap_or_default(),
                last_modified: item.last_modified,
            }
        })
        .collect::<Vec<_>>();
    pastes.sort_unstable_by(|a, b| b.last_modified.cmp(&a.last_modified));

    // We can calculate the etag based on the latest entry and the total amount of entries.
    // If something was deleted or added the count changes, if something was deleted and
    // added to keep the count equal, the latest last modified changed.
    let etag = pastes
        .first()
        .map(|f| format!("{}-{}", pastes.len(), f.last_modified))
        .unwrap_or_else(|| "empty".to_owned());

    Response::from_json(&pastes)?
        .with_etag(&etag)?
        .with_cache_control(
            CacheControl::default()
                .public()
                .s_max_age(consts::CACHE_FOREVER),
        )
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
