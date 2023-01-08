use std::{borrow::Cow, rc::Rc, time::Duration};

use pob::{PathOfBuilding, PathOfBuildingExt, SerdePathOfBuilding};
use serde::{Deserialize, Serialize};
use shared::{
    model::{PasteId, PasteMetadata},
    validation, User,
};

use crate::{
    consts, crypto, poe_api,
    request_context::RequestContext,
    response,
    route::{self, DeleteEndpoints, GetEndpoints, PostEndpoints},
    sentry,
    utils::{self, CacheControl, Etag, RequestExt},
    Error, Response, Result,
};

macro_rules! validate {
    ($e:expr, $msg:expr) => {
        if !$e {
            let msg = $msg.into();
            tracing::warn!(expr = stringify!($e), "validation failed: {}", msg);
            return Err(Error::BadRequest(msg));
        }
    };
}

macro_rules! validate_v {
    ($e:expr) => {
        match $e {
            validation::Validation::Valid => (),
            validation::Validation::Invalid(msg) => {
                tracing::warn!(expr = stringify!($e), "validation failed: {}", msg);
                return Err(Error::BadRequest(msg.into()));
            }
        }
    };
}

macro_rules! validate_access {
    ($e:expr) => {
        if !$e {
            tracing::warn!(expr = stringify!($e), "access denied");
            return Err(Error::AccessDenied);
        }
    };
}

pub async fn handle(rctx: &mut RequestContext, route: route::Api) -> response::Result {
    use route::{Api::*, DeleteEndpoints::*, GetEndpoints::*, PostEndpoints::*};

    // Whether this is a user facing API call.
    //
    // Currently this can happen on some API endpoints related to login/auth,
    // these are handled as API endpoints but are user facing, meaning
    // the user would expect a proper error page not just some JSON.
    let is_user_api = matches!(&route, Get(Login) | Get(Oauht2Poe));

    let r = match route {
        // Get
        Get(Oembed) => handle_oembed(rctx).await,
        Get(User(user)) => handle_user(rctx, user).await,
        Get(PobPaste(id)) => handle_download_text(rctx, id).await,
        Get(PobUserPaste(user, id)) => {
            handle_download_text(rctx, PasteId::new_user(user, id)).await
        }
        Get(Paste(id)) => handle_download_text(rctx, PasteId::Paste(id)).await,
        Get(UserPaste(user, id)) => handle_download_text(rctx, PasteId::new_user(user, id)).await,
        Get(PasteJson(id)) => handle_download_json(rctx, PasteId::Paste(id)).await,
        Get(UserPasteJson(user, id)) => {
            handle_download_json(rctx, PasteId::new_user(user, id)).await
        }
        Get(PasteXml(id)) => handle_download_xml(rctx, PasteId::Paste(id)).await,
        Get(UserPasteXml(user, id)) => handle_download_xml(rctx, PasteId::new_user(user, id)).await,
        Get(Login) => handle_login(rctx).await,
        Get(Oauht2Poe) => handle_oauth2_poe(rctx).await,
        // Post
        Post(Upload) => handle_upload(rctx).await,
        Post(PobUpload) => handle_pob_upload(rctx).await,
        // Delete
        Delete(DeletePaste(id)) => handle_delete_paste(rctx, id).await,
        // Not Found Routes - these should never happen,
        // but they are there because sycamore_router requires them.
        Get(GetEndpoints::NotFound)
        | Post(PostEndpoints::NotFound)
        | Delete(DeleteEndpoints::NotFound) => Ok(Response::not_found()),
    };

    match is_user_api {
        true => r.map_err(response::AppError),
        false => r.map_err(response::ApiError),
    }
}

#[derive(Default, Serialize)]
struct Oembed<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    author_name: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    author_url: Option<Cow<'a, str>>,
    provider_name: &'a str,
    provider_url: &'a str,
}

#[tracing::instrument(skip(rctx))]
async fn handle_oembed(rctx: &RequestContext) -> Result<Response> {
    let mut oembed = Oembed {
        provider_name: "Paste of Exile - POBb.in",
        provider_url: &format!("https://{}", rctx.url()?.host_str().unwrap()),
        ..Default::default()
    };

    let url = rctx.url()?;
    if let Some(author) = url
        .query_pairs()
        .find_map(|(k, v)| (k == "user").then_some(v))
    {
        oembed.author_url = Some(format!("{}/u/{author}", oembed.provider_url).into());
        oembed.author_name = Some(author);
    }

    Ok(Response::ok()
        .json(&oembed)
        .cache_for(Duration::from_secs(12 * 3600)))
}

#[tracing::instrument(skip(rctx))]
async fn handle_download_text(rctx: &RequestContext, id: PasteId) -> Result<Response> {
    let storage = rctx.inject::<crate::storage::Storage>();
    let paste = storage
        .get(&id)
        .await?
        .ok_or_else(|| Error::NotFound("paste", id.to_string()))?;

    Response::ok()
        .meta_paste(id, &paste)
        .body(paste.content)
        .content_type("text/plain")
        .etag(Etag::strong(&paste.entity_id))
        .cache(
            CacheControl::default()
                .public()
                .s_max_age(consts::CACHE_FOREVER),
        )
        .result()
}

#[tracing::instrument(skip(rctx))]
async fn handle_download_json(rctx: &RequestContext, id: PasteId) -> Result<Response> {
    let pastes = rctx.inject::<crate::pastes::Pastes>();
    let (meta, paste) = pastes
        .get_paste(&id)
        .await?
        .ok_or_else(|| Error::NotFound("paste", id.to_string()))?;

    Response::ok()
        .json(&paste)
        .meta_paste(id, paste)
        .content_type("application/json")
        .etag(Etag::strong(&meta.etag))
        .cache(
            CacheControl::default()
                .public()
                .s_max_age(consts::CACHE_FOREVER),
        )
        .result()
}

#[tracing::instrument(skip(rctx))]
async fn handle_download_xml(rctx: &RequestContext, id: PasteId) -> Result<Response> {
    let storage = rctx.inject::<crate::storage::Storage>();
    let paste = storage
        .get(&id)
        .await?
        .ok_or_else(|| Error::NotFound("paste", id.to_string()))?;

    let content = pob::decompress(&paste.content).map_err(|e| Error::BadRequest(e.to_string()))?;

    Response::ok()
        .meta_paste(id, &paste)
        .body(content)
        .content_type("application/xml")
        .etag(Etag::strong(&paste.entity_id))
        .cache(
            CacheControl::default()
                .public()
                .s_max_age(consts::CACHE_FOREVER),
        )
        .result()
}

#[tracing::instrument(skip(rctx))]
async fn handle_delete_paste(rctx: &RequestContext, id: PasteId) -> Result<Response> {
    let storage = rctx.inject::<crate::storage::Storage>();
    storage.delete(&id).await?;
    crate::cache::on_paste_change(rctx, id);
    Ok(Response::ok())
}

#[derive(Deserialize)]
struct UploadRequest {
    /// Existing id to update a paste.
    #[serde(default)]
    id: Option<PasteId>,

    /// Whether to create a new id as a user scoped paste.
    #[serde(default)]
    as_user: bool,

    /// Custom title for the paste, currently only supported
    /// for user pastes.
    #[serde(default)]
    title: Option<String>,
    /// Custom id for user pastes. Ignored when an id is already supplied.
    #[serde(default)]
    custom_id: Option<String>,

    content: String,
}

#[tracing::instrument(skip(rctx))]
async fn handle_upload(rctx: &mut RequestContext) -> Result<Response> {
    let data = rctx.req_mut().json::<UploadRequest>().await?;
    let content: Rc<[u8]> = data.content.into_bytes().into();

    tracing::info!(?data.id, data.as_user, ?data.title, ?data.custom_id, size = content.len(), "upload");
    sentry::add_attachment_plain(content.clone(), "pob.txt");

    let pob = validate_pob(&content)?;
    let mut metadata = to_metadata(&pob);

    let sha1 = crypto::sha1(&content).await?;

    let id = if data.as_user {
        let session = rctx.session().await?.ok_or_else(|| {
            tracing::warn!("missing user session");
            Error::AccessDenied
        })?;

        validate!(data.title.is_some(), "Title is required");
        let title = data.title.unwrap();
        validate_v!(validation::user::is_valid_custom_title(&title));

        metadata.title = title;

        if let Some(id) = data.id {
            validate_access!(Some(session.name.as_str()) == id.user().map(|user| user.as_str()));
            validate_v!(validation::user::is_valid_custom_id(id.id()));
            validate!(
                data.custom_id.as_deref() == Some(id.id()),
                "Custom id does not match paste id"
            );

            id
        } else {
            let id = match data.custom_id {
                Some(id) => id,
                None => utils::random_string::<9>()?,
            };
            validate_v!(validation::user::is_valid_custom_id(&id));

            PasteId::new_user(session.name, id)
        }
    } else {
        validate_access!(data.id.is_none());
        // TODO: should unused fields (like title) be validated?
        // Currently not validated becuse frontend may send old values
        // validate!(data.title.is_none(), "Cannot set title");
        // validate!(data.custom_id.is_none(), "Cannot set custom id");

        PasteId::Paste(utils::hash_to_short_id(&sha1, 9)?)
    };

    tracing::debug!("--> uploading paste '{}'", id);
    let storage = rctx.inject::<crate::storage::Storage>();
    storage.put(&id, &sha1, &content, Some(&metadata)).await?;
    tracing::debug!("<-- paste uploaded");

    let response = Response::ok().json(&id).meta_paste(&id, metadata);

    crate::cache::on_paste_change(rctx, id);

    Ok(response)
}

#[tracing::instrument(skip(rctx))]
async fn handle_pob_upload(rctx: &mut RequestContext) -> Result<Response> {
    let data: Rc<[u8]> = rctx.req_mut().bytes().await?.into();

    tracing::info!(size = data.len(), "pob upload");
    sentry::add_attachment_plain(data.clone(), "pob.txt");

    let pob = validate_pob(&data)?;
    let metadata = to_metadata(&pob);

    let sha1 = crypto::sha1(&data).await?;
    let id = PasteId::new_id(utils::hash_to_short_id(&sha1, 9)?);

    tracing::debug!("--> uploading paste '{}'", id);
    let storage = rctx.inject::<crate::storage::Storage>();
    storage
        .put_async(rctx.ctx(), &id, &sha1, data, Some(&metadata))
        .await?;
    tracing::debug!("<-- paste uploaing ...");

    let response = Response::ok()
        .body(id.to_string())
        .meta_paste(&id, metadata);

    crate::cache::on_paste_change(rctx, id);

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
    SerdePathOfBuilding::from_xml(&s).map_err(move |e| Error::InvalidPoB(e, s))
}

fn to_metadata(pob: &SerdePathOfBuilding) -> PasteMetadata {
    PasteMetadata {
        title: app::pob::title(pob),
        ascendancy_or_class: pob.ascendancy_or_class_name().to_owned(),
        version: pob.max_tree_version(),
        main_skill_name: pob.main_skill_name().map(|x| x.to_owned()),
    }
}

#[tracing::instrument(skip(rctx))]
async fn handle_user(rctx: &RequestContext, user: User) -> Result<Response> {
    let pastes = rctx.inject::<crate::pastes::Pastes>();
    let (meta, pastes) = pastes.list_pastes(&user).await?;

    Response::ok()
        .json(&pastes)
        .meta_list(user)
        .etag(Etag::strong(&meta.etag))
        .cache(
            CacheControl::default()
                .public()
                .s_max_age(consts::CACHE_FOREVER),
        )
        .result()
}

#[tracing::instrument(skip(rctx))]
async fn handle_login(rctx: &RequestContext) -> Result<Response> {
    let req_url = rctx.url()?;
    let host = crate::utils::if_debug!("preview.pobb.in", req_url.host_str().unwrap());

    let state = create_oauth_state(&req_url, rctx.referrer().as_ref())?;
    let redirect_uri = format!("https://{host}/oauth2/authorization/poe");
    let login_uri = rctx.inject::<crate::poe_api::Oauth>().get_login_url(
        &redirect_uri,
        &state,
        consts::OAUTH_SCOPE,
    );

    tracing::info!(%redirect_uri, %state, "redirecting for login");

    Ok(Response::redirect_temp(&login_uri).state_cookie(&state))
}

#[tracing::instrument(skip(rctx))]
async fn handle_oauth2_poe(rctx: &RequestContext) -> Result<Response> {
    let url = rctx.url()?;

    let grant = match poe_api::AuthorizationGrant::try_from(&url) {
        Ok(grant) => grant,
        Err(poe_api::AuthorizationGrantParseError::UserDeniedAccess(state)) => {
            tracing::info!("user denied access for login");
            return Response::redirect_temp(redirect_from_oauth_state(&state))
                .delete_state_cookie()
                .result();
        }
        Err(poe_api::AuthorizationGrantParseError::Error { name, description }) => {
            return Err(Error::AuthorizationGrantError(format!(
                "{name}: {description:?}"
            )))
        }
        Err(poe_api::AuthorizationGrantParseError::MissingAuthorizationGrant) => {
            return Err(Error::MissingAuthorizationGrant)
        }
    };

    tracing::info!(%grant.state, "logging in");

    crate::utils::if_debug!({}, {
        use crate::utils::RequestExt;
        let cookie_state = rctx.cookie("state").unwrap_or_default();
        if cookie_state != grant.state {
            tracing::warn!(%cookie_state, %grant.state, "grant state does not match cookie state");
            return Err(Error::InvalidSessionState);
        }
    });

    let oauth = rctx.inject::<crate::poe_api::Oauth>();
    let token = oauth.fetch_token(&grant.code).await?;

    let profile = poe_api::PoeApi::new(token.access_token)
        .fetch_profile()
        .await?;

    sentry::update_username(&profile.name);

    let user = app::User {
        name: User::new_unchecked(profile.name),
    };
    let session = rctx
        .inject::<crate::dangerous::Dangerous>()
        .sign(&user)
        .await?;

    Response::redirect_temp(redirect_from_oauth_state(&grant.state))
        .delete_state_cookie()
        .new_session(&session)
        .result()
}

fn create_oauth_state(req_url: &url::Url, referrer: Option<&url::Url>) -> Result<String> {
    let path = referrer
        .filter(|url| url.host_str() == req_url.host_str())
        .map(|url| &url[url::Position::BeforePath..])
        .unwrap_or("/");
    Ok(format!("{}.{}", utils::random_string::<12>()?, path))
}

fn redirect_from_oauth_state(state: &str) -> &str {
    state
        .split_once('.')
        .map(|(_, path)| path)
        .filter(|path| !path.is_empty())
        .unwrap_or("/")
}
