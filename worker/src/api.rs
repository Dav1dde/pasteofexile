use std::{borrow::Cow, time::Duration};

use crate::{
    consts, crypto, poe_api,
    request_context::RequestContext,
    route::{self, DeleteEndpoints, GetEndpoints, PostEndpoints},
    utils::{self, is_valid_id, CacheControl, ResponseExt},
    Error, Result,
};
use pob::{PathOfBuilding, PathOfBuildingExt, SerdePathOfBuilding};
use serde::{Deserialize, Serialize};
use shared::model::{PasteId, PasteMetadata};
use worker::{Headers, Response};

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

pub async fn try_handle(rctx: &mut RequestContext, route: route::Api) -> Result<Option<Response>> {
    match route {
        route::Api::Get(get) => match get {
            GetEndpoints::Oembed => handle_oembed(rctx).await.map(Some),
            GetEndpoints::User(user) => handle_user(rctx, user).await.map(Some),
            GetEndpoints::PobPaste(id) => handle_download_text(rctx, id).await.map(Some),
            GetEndpoints::PobUserPaste(user, id) => {
                handle_download_text(rctx, PasteId::new_user(user, id))
                    .await
                    .map(Some)
            }
            GetEndpoints::Paste(id) => handle_download_text(rctx, PasteId::Paste(id))
                .await
                .map(Some),
            GetEndpoints::UserPaste(user, id) => {
                handle_download_text(rctx, PasteId::new_user(user, id))
                    .await
                    .map(Some)
            }
            GetEndpoints::PasteJson(id) => handle_download_json(rctx, PasteId::Paste(id))
                .await
                .map(Some),
            GetEndpoints::UserPasteJson(user, id) => {
                handle_download_json(rctx, PasteId::new_user(user, id))
                    .await
                    .map(Some)
            }
            GetEndpoints::Login => handle_login(rctx).await.map(Some),
            GetEndpoints::Oauht2Poe => handle_oauth2_poe(rctx).await.map(Some),
            GetEndpoints::NotFound => Ok(None),
        },
        route::Api::Post(post) => match post {
            PostEndpoints::Upload() => handle_upload(rctx).await.map(Some),
            PostEndpoints::PobUpload() => handle_pob_upload(rctx).await.map(Some),
            PostEndpoints::NotFound => Ok(None),
        },
        route::Api::Delete(delete) => match delete {
            DeleteEndpoints::DeletePaste(id) => handle_delete_paste(rctx, id).await.map(Some),
            DeleteEndpoints::NotFound => Ok(None),
        },
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

    worker::Response::from_json(&oembed)?.cache_for(Duration::from_secs(12 * 3_600))
}

#[tracing::instrument(skip(rctx))]
async fn handle_download_text(rctx: &RequestContext, id: PasteId) -> Result<Response> {
    let paste = rctx
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

#[tracing::instrument(skip(rctx))]
async fn handle_download_json(rctx: &RequestContext, id: PasteId) -> Result<Response> {
    let paste = rctx
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

#[tracing::instrument(skip(rctx))]
async fn handle_delete_paste(rctx: &RequestContext, id: PasteId) -> Result<Response> {
    rctx.storage()?.delete(&id).await?;
    crate::cache::on_paste_change(rctx, id);
    Ok(Response::empty()?)
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
    let mut content = data.content.into_bytes();

    tracing::info!(?data.id, data.as_user, ?data.title, ?data.custom_id, size = content.len(), "upload");

    let pob = validate_pob(&content)?;
    let mut metadata = to_metadata(&pob);

    let sha1 = crypto::sha1(&mut content).await?;

    let id = if data.as_user {
        let session = rctx.session().await?.ok_or(Error::AccessDenied)?;

        validate!(data.title.is_some(), "Title is required");
        let title = data.title.unwrap();
        validate!(title.len() < 90, "Title too long");
        validate!(title.len() > 5, "Title too short");

        metadata.title = title;

        if let Some(id) = data.id {
            validate_access!(Some(session.name.as_str()) == id.user());
            validate!(is_valid_id(id.id()), "Invalid id");
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
            validate!(is_valid_id(&id), "Invalid id");

            PasteId::new_user(session.name, id)
        }
    } else {
        validate_access!(data.id.is_none());
        // TODO: should unused fields (like title) be validated?
        // validate!(data.title.is_none(), "Cannot set title");
        validate!(data.custom_id.is_none(), "Cannot set custom id");

        PasteId::Paste(utils::hash_to_short_id(&sha1, 9)?)
    };

    tracing::debug!("--> uploading paste '{}'", id);
    rctx.storage()?
        .put(&id, &sha1, &mut content, Some(metadata))
        .await?;
    tracing::debug!("<-- paste uploaded");

    let body = serde_json::to_vec(&id)?;

    crate::cache::on_paste_change(rctx, id);

    Response::from_bytes(body)?.with_content_type("application/json")
}

#[tracing::instrument(skip(rctx))]
async fn handle_pob_upload(rctx: &mut RequestContext) -> Result<Response> {
    let mut data = rctx.req_mut().bytes().await?;

    let pob = validate_pob(&data)?;
    let metadata = to_metadata(&pob);

    let sha1 = crypto::sha1(&mut data).await?;
    let id = PasteId::new_id(utils::hash_to_short_id(&sha1, 9)?);

    tracing::debug!("--> uploading paste '{}'", id);
    rctx.storage()?
        .put_async(rctx.ctx(), &id, &sha1, data, Some(metadata))
        .await?;
    tracing::debug!("<-- paste uploaing ...");

    let response = Response::ok(id.to_string())?;

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
    SerdePathOfBuilding::from_xml(&s).map_err(move |e| Error::InvalidPoB(e.to_string(), s))
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
async fn handle_user(rctx: &RequestContext, user: String) -> Result<Response> {
    // TODO: code duplication with lib.rs
    let mut pastes = rctx
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
                ascendancy_or_class: metadata.ascendancy_or_class,
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

#[tracing::instrument(skip(rctx))]
async fn handle_login(rctx: &RequestContext) -> Result<Response> {
    let _url = rctx.url()?;
    let host = crate::utils::if_debug!("preview.pobb.in", _url.host_str().unwrap());

    let state = utils::random_string::<16>()?;

    let redirect_uri = format!("https://{host}/oauth2/authorization/poe");
    let login_uri = rctx
        .oauth()?
        .get_login_url(&redirect_uri, &state, consts::OAUTH_SCOPE);

    Response::redirect_temp(&login_uri)?.with_state_cookie(&state)
}

async fn handle_oauth2_poe(rctx: &RequestContext) -> Result<Response> {
    let url = rctx.url()?;

    let grant = match poe_api::AuthorizationGrant::try_from(&url) {
        Ok(grant) => grant,
        // TODO: render error page
        _ => return Ok(Response::error("no authorization grant", 403)?),
    };

    crate::utils::if_debug!({}, {
        use crate::utils::RequestExt;
        if rctx.cookie("state").unwrap_or_default() != grant.state {
            // TODO: render error page
            return Ok(Response::error("invalid session state", 403)?);
        }
    });

    // TODO: error handling -> show error page
    let token = rctx.oauth()?.fetch_token(&grant.code).await?;

    let profile = poe_api::PoeApi::new(token.access_token)
        .fetch_profile()
        .await?;

    let user = app::User { name: profile.name };
    let session = rctx.dangerous()?.sign(&user).await?;

    // TODO: redirect back to where the user actually came from
    Response::redirect_temp(&format!("/u/{}", user.name))?
        .with_delete_state_cookie()?
        .with_new_session(&session)
}
