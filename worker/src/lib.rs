use serde::Serialize;
use worker::{event, Env, Request, Response};

mod api;
mod assets;
mod b2;
mod bindgen;
mod consts;
mod crypto;
mod error;
mod utils;

pub use self::error::{Error, Result};
use assets::EnvAssetExt;

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
pub async fn main(req: Request, env: Env) -> worker::Result<Response> {
    #[cfg(feature = "debug")]
    {
        LAST_LOG_MSG.with(|last| last.set(worker::Date::now().as_millis()));
        LOG_INIT.call_once(setup_logging);
    }

    let err = match try_main(req, env).await {
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

async fn try_main(mut req: Request, env: Env) -> Result<Response> {
    // TODO: caching header

    if let Some(response) = api::try_handle(&mut req, &env).await? {
        return Ok(response);
    }

    if req.path() == "/oembed.json" {
        return Ok(Response::from_json(&Oembed {
            provider_name: "Paste of Exile",
            provider_url: &format!("https://{}", req.url()?.host_str().unwrap()),
        })?);
    }

    if let Some(response) = assets::try_handle(&mut req, &env).await? {
        return Ok(response);
    }

    let route = app::Route::resolve(&req.path());
    let ctx = build_context(&req, &env, route).await?;

    let head = app::render_head(ctx.clone());
    let (app, rctx) = app::render_to_string(ctx);

    let index = env.get_asset("index.html")?.text().await?.unwrap();
    let index = index.replace("<!-- %head% -->", &head);
    let index = index.replace("<!-- %app% -->", &app);

    #[allow(clippy::redundant_clone)]
    Ok(Response::from_html(index.clone())?.with_status(rctx.status_code()))
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
