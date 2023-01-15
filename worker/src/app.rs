use shared::model::{PasteId, UserPasteId};

use crate::{
    app_metadata, consts, response, sentry,
    utils::{to_link, Etag},
    CacheControl, Error, RequestContext, Response, Result,
};

pub async fn handle(rctx: &RequestContext, route: app::Route) -> response::Result {
    handle_inner(rctx, route).await.map_err(response::AppError)
}

pub async fn handle_err(err: crate::Error) -> Response {
    let err = match err {
        crate::Error::NotFound(typ, id) => app::Error::NotFound(typ, id),
        err => app::Error::ServerError(err.to_string()),
    };

    render(ResponseInfo::default(), app::Context::error(err)).await
}

async fn handle_inner(rctx: &RequestContext, route: app::Route) -> Result<Response> {
    let (info, ctx) = build_context(rctx, route).await.unwrap_or_else(|err| {
        sentry::capture_err(&err);
        let err = match err {
            Error::InvalidPoB(err, _) => app::Error::PobError(err),
            err => app::Error::ServerError(err.to_string()),
        };

        (ResponseInfo::default(), app::Context::error(err))
    });

    if let Some(location) = info.redirect {
        return Ok(Response::redirect_perm(&location));
    }

    Ok(render(info, ctx).await)
}

async fn render(info: ResponseInfo, ctx: app::Context) -> Response {
    let (app, resp_ctx) = app::render_to_string(ctx);
    let link_preload = to_link(&resp_ctx.preload, "preload");

    let head = app::render_head(app::Head {
        meta: resp_ctx.meta.unwrap_or_default(),
        prefetch: resp_ctx.prefetch,
        preload: resp_ctx.preload,
    });

    // Not sure if I like that, this requries trunk to run before building the worker.
    let index = include_str!("../../app/dist/index.html")
        .replace("<!-- %head% -->", &head)
        .replace("<!-- %app% -->", &app);

    let etag = info.etag.as_deref().map(|etag| Etag::weak(etag).git());

    Response::status(resp_ctx.status_code)
        .html(index)
        .meta(info.meta)
        .etag(etag)
        .append_header("Link", app_metadata::EARLY_HINTS)
        .append_header("Link", &link_preload)
        .cache(info.cache_control)
}

#[tracing::instrument(skip(rctx))]
async fn build_context(
    rctx: &RequestContext,
    route: app::Route,
) -> Result<(ResponseInfo, app::Context)> {
    // TODO: refactor this context garbage, maybe make it into a trait?
    use app::{Context, Route::*};
    let (info, ctx) = match route {
        Index => (ResponseInfo::default().with_etag("index"), Context::index()),
        NotFound => (
            ResponseInfo::default().with_etag("not_found"),
            Context::not_found(),
        ),
        Paste(id) => {
            let id = PasteId::new_id(id);

            paste_page(rctx, id, Context::paste).await?
        }
        User(user) => {
            let pastes = rctx.inject::<crate::pastes::Pastes>();
            let (meta, pastes) = pastes.list_pastes(&user).await?;

            let info = ResponseInfo {
                etag: Some(meta.etag),
                meta: Some(response::Meta::list(&user)),
                ..Default::default()
            };

            (info, Context::user(user, pastes))
        }
        UserPaste(user, id) => {
            let id = PasteId::new_user(user, id);

            paste_page(rctx, id, Context::user_paste).await?
        }
        UserEditPaste(user, id) => {
            let location = UserPasteId { user, id }.to_paste_url();
            (ResponseInfo::redirect(location), Context::not_found())
        }
    };

    Ok((info, ctx))
}

async fn paste_page(
    rctx: &RequestContext,
    id: PasteId,
    mapper: impl Fn(PasteId, shared::model::Paste) -> app::Context,
) -> Result<(ResponseInfo, app::Context)> {
    let pastes = rctx.inject::<crate::pastes::Pastes>();

    let r = match pastes.get_paste(&id).await {
        Ok(Some((meta, paste))) => {
            let info = ResponseInfo {
                etag: Some(meta.etag),
                meta: Some(response::Meta::paste(&id, &paste)),
                ..Default::default()
            };
            (info, mapper(id, paste))
        }
        Err(Error::InvalidId(..)) | Ok(None) => (
            ResponseInfo::default().with_etag("not_found"),
            app::Context::not_found(),
        ),
        Err(err) => return Err(err),
    };

    Ok(r)
}

struct ResponseInfo {
    cache_control: CacheControl,
    etag: Option<String>,
    redirect: Option<String>,
    meta: Option<response::Meta>,
}

impl ResponseInfo {
    pub fn redirect(location: String) -> Self {
        Self {
            redirect: Some(location),
            ..Default::default()
        }
    }

    pub fn with_etag(mut self, etag: impl Into<String>) -> Self {
        self.etag = Some(etag.into());
        self
    }
}

impl Default for ResponseInfo {
    fn default() -> Self {
        Self {
            cache_control: CacheControl::default()
                .public()
                .s_max_age(consts::CACHE_FOREVER),
            etag: None,
            redirect: None,
            meta: None,
        }
    }
}
