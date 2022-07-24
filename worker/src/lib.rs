use serde::Serialize;
use shared::model::{PasteId, PasteSummary};
use std::{future::Future, time::Duration};
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
use sentry::Sentry;
use utils::{CacheControl, EnvExt, ResponseExt};

struct ResponseInfo {
    cache_control: CacheControl,
    etag: Option<String>,
}

impl Default for ResponseInfo {
    fn default() -> Self {
        Self {
            cache_control: CacheControl::default().max_age(Duration::from_secs(3_600)),
            etag: None,
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

            let info = ResponseInfo {
                cache_control: CacheControl::default().s_max_age(consts::CACHE_FOREVER),
                etag: Some(etag),
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
            let id = PasteId::new_user(user, id);
            // TODO: handle 404

            let mut info = ResponseInfo {
                cache_control: CacheControl::default().s_max_age(consts::CACHE_FOREVER),
                ..Default::default()
            };

            if let Some(paste) = env.storage()?.get(&id).await? {
                info.etag = paste.entity_id.clone();
                (
                    info,
                    Context::user_paste_edit(host, id.unwrap_user(), paste),
                )
            } else {
                info.etag = Some("not_found".to_owned());
                (info, Context::not_found(host))
            }
        }
    };

    Ok((info, ctx))
}

#[derive(Serialize)]
struct Oembed<'a> {
    provider_name: &'a str,
    provider_url: &'a str,
}

thread_local!(static SENTRY: std::cell::RefCell<Option<Sentry>> = std::cell::RefCell::new(None));
#[cfg(feature = "debug")]
thread_local!(static LAST_LOG_MSG: std::cell::Cell<u64> = std::cell::Cell::new(0));
#[cfg(feature = "debug")]
static LOG_INIT: std::sync::Once = std::sync::Once::new();

#[macro_export]
macro_rules! sentry {
    ($sentry:ident, $block:expr) => {{
        $crate::SENTRY.with(|ctx| {
            if let Some($sentry) = ctx.borrow_mut().as_mut() {
                $block
            }
        })
    }};
}

#[event(fetch)]
pub async fn main(mut req: Request, env: Env, ctx: Context) -> worker::Result<Response> {
    #[cfg(feature = "debug")]
    {
        LAST_LOG_MSG.with(|last| last.set(worker::Date::now().as_millis()));
        LOG_INIT.call_once(setup_logging);
    }

    if let Some(sentry) = Sentry::from_env(&env, ctx.clone(), req.inner()) {
        SENTRY.with(|ctx| {
            *ctx.borrow_mut() = Some(sentry);
        });
    }

    let err: ErrorResponse = match cached(&mut req, &env, &ctx, try_main).await {
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
            sentry!(sentry, sentry.capture_err(&err));

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

async fn try_main(req: &mut Request, env: &Env, ctx: &Context) -> Result<Response> {
    if let Some(response) = api::try_handle(ctx, req, env).await? {
        return Ok(response);
    }

    if req.path() == "/oembed.json" && req.method() == Method::Get {
        let oembed = Oembed {
            provider_name: "Paste of Exile - POBb.in",
            provider_url: &format!("https://{}", req.url()?.host_str().unwrap()),
        };
        return Response::from_json(&oembed)?.cache_for(Duration::from_secs(12 * 3_600));
    }

    if let Some(response) = assets::try_handle(req, env).await? {
        return Ok(response);
    }

    let route = app::Route::resolve(&req.path());
    let (info, ctx) = build_context(req, env, route).await?;

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

#[cfg(feature = "debug")]
fn setup_logging() {
    console_error_panic_hook::set_once();
    let _ = fern::Dispatch::new()
        .format(|out, message, record| {
            let now = worker::Date::now().as_millis();
            let last = LAST_LOG_MSG.with(|last| last.replace(now));

            out.finish(format_args!(
                "[+ {:>5}] <{:<25}> {:>5}: {}",
                now - last,
                format_args!(
                    "{}:{}",
                    record.file().unwrap_or_else(|| record.target()),
                    record.line().unwrap_or(0)
                ),
                record.level(),
                message,
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::Output::call(console_log::log))
        .apply();
}
