use serde::Serialize;
use shared::model::{PasteId, PasteSummary, UserPasteId};
use std::{borrow::Cow, future::Future, time::Duration};
use worker::{event, Cache, Context, Env, Headers, Method, Request, Response};

mod api;
mod assets;
mod cache;
mod consts;
mod crypto;
mod dangerous;
mod error;
mod poe_api;
mod retry;
mod sentry;
mod storage;
mod utils;

pub use self::error::{Error, ErrorResponse, Result};
use assets::EnvAssetExt;
use utils::{CacheControl, EnvExt, ResponseExt};

struct ResponseInfo {
    cache_control: CacheControl,
    etag: Option<String>,
    redirect: Option<String>,
}

impl Default for ResponseInfo {
    fn default() -> Self {
        Self {
            cache_control: CacheControl::default().max_age(Duration::from_secs(3_600)),
            etag: None,
            redirect: None,
        }
    }
}

async fn build_context(
    req: &Request,
    env: &Env,
    route: app::Route,
) -> Result<(ResponseInfo, app::Context)> {
    // TODO: refactor this context garbage, maybe make it into a trait?
    let host = req.url()?.host_str().unwrap().to_owned();
    use app::{Context, Route::*};
    let (info, ctx) = match route {
        Index => (ResponseInfo::default(), Context::index(host)),
        NotFound => (ResponseInfo::default(), Context::not_found(host)),
        Paste(id) => {
            let id = PasteId::new_id(id);
            // TODO: handle 404

            // We can cache this forever because we know anonymous pastes will never change
            // For 404's this is technically incorrect, but what are the odds...
            let mut info = ResponseInfo {
                cache_control: CacheControl::default().max_age(consts::CACHE_FOREVER),
                ..Default::default()
            };

            if let Some(paste) = env.storage()?.get(&id).await? {
                info.etag = paste.entity_id.clone();
                (info, Context::paste(host, id.to_string(), paste))
            } else {
                info.etag = Some("not_found".to_owned());
                (info, Context::not_found(host))
            }
        }
        User(name) => {
            // TODO: code duplication with api.rs
            let mut pastes = env
                .storage()?
                .list(format!("user/{name}/pastes/"))
                .await?
                .into_iter()
                .map(|item| {
                    let metadata = item.metadata.unwrap_or_default();
                    let id = item.name.rsplit_once('/').unwrap().1.to_owned();

                    PasteSummary {
                        id,
                        user: Some(name.clone()),
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

            let info = ResponseInfo {
                cache_control: CacheControl::default().s_max_age(consts::CACHE_FOREVER),
                etag: Some(etag),
                ..Default::default()
            };

            (info, Context::user(host, name, pastes))
        }
        UserPaste(user, id) => {
            let id = PasteId::new_user(user, id);
            // TODO: handle 404

            let mut info = ResponseInfo {
                cache_control: CacheControl::default().s_max_age(consts::CACHE_FOREVER),
                ..Default::default()
            };

            if let Some(paste) = env.storage()?.get(&id).await? {
                info.etag = paste.entity_id.clone();
                (info, Context::user_paste(host, id.unwrap_user(), paste))
            } else {
                info.etag = Some("not_found".to_owned());
                (info, Context::not_found(host))
            }
        }
        UserEditPaste(user, id) => {
            let info = ResponseInfo {
                redirect: Some(UserPasteId { user, id }.to_paste_url()),
                ..Default::default()
            };

            (info, Context::not_found(host))
        }
    };

    Ok((info, ctx))
}

#[derive(Default, Serialize)]
struct Oembed<'a> {
    author_name: Option<Cow<'a, str>>,
    author_url: Option<Cow<'a, str>>,
    provider_name: &'a str,
    provider_url: &'a str,
}

static LOG_INIT: std::sync::Once = std::sync::Once::new();

#[event(fetch)]
pub async fn main(req: Request, env: Env, ctx: Context) -> worker::Result<Response> {
    LOG_INIT.call_once(|| {
        use tracing_subscriber::prelude::*;
        tracing_subscriber::registry()
            .with(sentry::Layer {})
            .init();
    });

    sentry::init(&env, &ctx, &req);
    let response = main_inner(req, env, ctx).await;
    sentry::finish();

    response
}

async fn main_inner(mut req: Request, env: Env, ctx: Context) -> worker::Result<Response> {
    let err: ErrorResponse = match cached(&mut req, &env, &ctx, try_main2).await {
        Ok(response) => {
            worker::console_log!(
                "{:?} {} {}",
                req.method(),
                req.path(),
                response.status_code()
            );
            return Ok(response);
        }
        Err(err) => {
            sentry::with_sentry(|sentry| sentry.capture_err(&err));

            err.into()
        }
    };

    // Don't use ResponseExt here, it returns crate::Result
    let mut headers = Headers::new();
    headers.set("Content-Type", "application/json")?;
    let response = Response::ok(serde_json::to_string(&err)?)?
        .with_status(err.code)
        .with_headers(headers);

    worker::console_warn!(
        "{:?} {} {} {}",
        req.method(),
        req.path(),
        response.status_code(),
        err.message
    );

    Ok(response)
}

#[tracing::instrument(skip_all)]
async fn try_main2(req: &mut Request, env: &Env, ctx: &Context) -> Result<Response> {
    try_main(req, env, ctx).await
}

#[tracing::instrument(skip_all)]
async fn try_main(req: &mut Request, env: &Env, ctx: &Context) -> Result<Response> {
    if let Some(response) = api::try_handle(ctx, req, env).await? {
        return Ok(response);
    }

    log::info!("test");

    if req.path() == "/oembed.json" && req.method() == Method::Get {
        let mut oembed = Oembed {
            provider_name: "Paste of Exile - POBb.in",
            provider_url: &format!("https://{}", req.url()?.host_str().unwrap()),
            ..Default::default()
        };

        let url = req.url()?;
        if let Some(author) = url
            .query_pairs()
            .find_map(|(k, v)| (k == "user").then_some(v))
        {
            oembed.author_url = Some(format!("{}/u/{author}", oembed.provider_url).into());
            oembed.author_name = Some(author);
        }

        return Response::from_json(&oembed)?.cache_for(Duration::from_secs(12 * 3_600));
    }

    if let Some(response) = assets::try_handle(req, env).await? {
        return Ok(response);
    }

    let route = app::Route::resolve(&req.path());
    let (info, ctx) = build_context(req, env, route).await?;

    if let Some(location) = info.redirect {
        return Response::redirect_perm(&location);
    }

    let (app, rctx) = app::render_to_string(ctx);
    let head = app::render_head(app::Head {
        meta: rctx.meta.unwrap_or_default(),
        prefetch: rctx.prefetch,
        preload: rctx.preload,
    });

    let index = env.get_asset("index.html")?.text().await?.unwrap();
    let index = index.replace("<!-- %head% -->", &head);
    let index = index.replace("<!-- %app% -->", &app);

    Response::from_html(index)?
        .with_status(rctx.status_code)
        .with_etag_opt(info.etag.as_deref())?
        .with_cache_control(info.cache_control)
}

async fn cached<'a, F, Fut>(
    req: &'a mut Request,
    env: &'a Env,
    ctx: &'a worker::Context,
    f: F,
) -> Result<Response>
where
    F: Fn(&'a mut Request, &'a Env, &'a worker::Context) -> Fut,
    Fut: Future<Output = Result<Response>> + 'a,
{
    let cache = Cache::default();
    let use_cache = req.method() == Method::Get;

    if use_cache {
        if let Some(response) = cache.get(&*req, true).await? {
            log::debug!("cache hit");
            return response
                .dup_headers() // cached response has immutable headers
                .with_header("Cf-Cache-Status", "HIT");
        }
    }

    // I think I cannot get around this clone in current rust,
    // if I use HRTBs for `F`, I cannot give `Fut` the correct lifetime.
    // except by using a `BoxFuture`, which isn't really better.
    // --> clone always (need to clone in most cases anyways)
    let request_for_cache = req.clone()?;
    let response = f(req, env, ctx).await?;
    let req = request_for_cache;

    if use_cache && should_cache(&response) {
        let (response, response_for_cache) = response.cloned()?;

        ctx.wait_until(async move {
            log::debug!("--> caching response");
            let _ = cache.put(&req, response_for_cache).await;
            log::debug!("<-- response cached");
        });

        response.with_header("Cf-Cache-Status", "MISS")
    } else {
        Ok(response)
    }
}

fn should_cache(response: &Response) -> bool {
    response.headers().has("Cache-Control").unwrap()
}
