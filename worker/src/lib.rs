use std::future::Future;

use serde::Serialize;
use worker::{event, Cache, Env, Method, Request, Response};

mod api;
mod assets;
mod b2;
mod consts;
mod crypto;
mod error;
mod retry;
mod utils;

pub use self::error::{Error, Result};
use assets::EnvAssetExt;
use utils::ResponseExt;

async fn build_context(req: &Request, env: &Env, route: app::Route) -> Result<app::Context> {
    // TODO: refactor this context garbage, maybe make it into a trait?
    let host = req.url()?.host_str().unwrap().to_owned();
    use app::{Context, Route::*};
    let ctx = match route {
        Index => Context::index(host),
        NotFound => Context::not_found(host),
        Paste(name) => {
            let b2 = b2::B2::from_env(env)?;

            match utils::to_path(&name) {
                Err(_) => Context::not_found(host),
                Ok(path) => {
                    let mut response = b2.download(&path).await?;
                    if response.status_code() == 200 {
                        let content = response.text().await?;
                        Context::paste(host, name, content)
                    } else {
                        Context::not_found(host)
                    }
                }
            }
        }
    };

    Ok(ctx)
}

#[derive(Serialize)]
struct Oembed<'a> {
    provider_name: &'a str,
    provider_url: &'a str,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    code: u16,
    message: String,
}

#[cfg(feature = "debug")]
thread_local!(static LAST_LOG_MSG: std::cell::Cell<u64> = std::cell::Cell::new(0));
#[cfg(feature = "debug")]
static LOG_INIT: std::sync::Once = std::sync::Once::new();

#[event(fetch)]
pub async fn main(mut req: Request, env: Env, ctx: worker::Context) -> worker::Result<Response> {
    #[cfg(feature = "debug")]
    {
        LAST_LOG_MSG.with(|last| last.set(worker::Date::now().as_millis()));
        LOG_INIT.call_once(setup_logging);
    }

    let err = match cached(&mut req, &env, &ctx, try_main).await {
        Ok(response) => return Ok(response),
        Err(err) => err,
    };

    let err = match err {
        err @ Error::NotFound(_, _) => ErrorResponse {
            code: 404,
            message: err.to_string(),
        },
        err => ErrorResponse {
            code: 500,
            message: err.to_string(),
        },
    };

    let mut headers = worker::Headers::new();
    headers.set("Content-Type", "application/json")?;
    let response = Response::ok(serde_json::to_string(&err)?)?
        .with_status(err.code)
        .with_headers(headers);
    Ok(response)
}

async fn try_main(req: &mut Request, env: &Env, _ctx: &worker::Context) -> Result<Response> {
    if let Some(response) = api::try_handle(req, env).await? {
        return Ok(response);
    }

    if req.path() == "/oembed.json" && req.method() == Method::Get {
        let oembed = Oembed {
            provider_name: "Paste of Exile - POB B.in",
            provider_url: &format!("https://{}", req.url()?.host_str().unwrap()),
        };
        return Response::from_json(&oembed)?.cache_for(12 * 3_600);
    }

    if let Some(response) = assets::try_handle(req, env).await? {
        return Ok(response);
    }

    let route = app::Route::resolve(&req.path());
    let ctx = build_context(req, env, route).await?;

    let head = app::render_head(ctx.clone());
    let (app, rctx) = app::render_to_string(ctx);

    let index = env.get_asset("index.html")?.text().await?.unwrap();
    let index = index.replace("<!-- %head% -->", &head);
    let index = index.replace("<!-- %app% -->", &app);

    let response = Response::from_html(index)?
        .with_status(rctx.status_code())
        .cache_for(3_600)?;
    Ok(response)
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
            // TODO: 304 handling?
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

    if use_cache {
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
                format!(
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
