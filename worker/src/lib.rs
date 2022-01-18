use pob::{PathOfBuilding, SerdePathOfBuilding};
use serde::Serialize;
use worker::{event, Env, Method, Request, Response};

mod assets;
mod b2;
mod bindgen;
mod consts;
mod crypto;
mod error;
mod utils;

pub use self::error::{Error, Result};
use assets::KvAssetExt;

async fn handle_upload(mut req: Request, env: &Env) -> Result<Response> {
    let mut data = req.bytes().await?;

    // TODO: proper error handling
    // TODO: maybe shortcut this without actually parsing
    SerdePathOfBuilding::from_export(std::str::from_utf8(&data).unwrap()).unwrap();

    let b2 = b2::B2::from_env(env)?;

    let sha1 = crypto::sha1(&mut data).await?;
    let id = utils::hash_to_short_id(&sha1, 9)?;
    let filename = utils::to_path(&id)?;

    log::debug!("--> uploading paste '{}' to '{}'", id, filename);
    b2.upload(
        &b2::UploadSettings {
            filename: &filename,
            content_type: "text/plain",
            sha1: Some(&utils::hex(&sha1)),
        },
        &mut data,
    )
    .await?;
    log::debug!("<-- paste uploaded");

    // Weird garbage data bug, but this seems to make it better
    let response = serde_json::to_string(&Upload { id })?;
    let mut response = Response::from_bytes(response.into_bytes())?;
    response
        .headers_mut()
        .set("Content-Type", "application/json")?;
    Ok(response)
}

#[derive(Serialize)]
struct Upload {
    id: String,
}

async fn build_context(req: &Request, env: &Env, route: app::Route) -> Result<app::Context> {
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

async fn download(env: &Env, id: &str) -> Result<Response> {
    let b2 = b2::B2::from_env(env)?;

    let path = utils::to_path(id)?;
    let response = b2.download(&path).await?;
    match response.status_code() {
        200 => {
            let mut headers = worker::Headers::new();
            headers.set("Content-Type", "text/plain")?;
            headers.set("Cache-Control", "max-age=31536000")?;

            bindgen::Response::dup(response, &headers)
        }
        404 => Err(Error::NotFound("paste", id.to_owned())),
        status => Err(Error::RemoteFailed(
            status,
            "failed to get paste".to_owned(),
        )),
    }
}

fn is_raw_url(path: &str) -> Option<&str> {
    path.trim_start_matches('/')
        .split_once('/')
        .filter(|(_, raw)| *raw == "raw")
        .map(|(id, _)| id)
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    code: u16,
    message: String,
}

#[cfg(feature = "debug")]
thread_local!(static LAST_LOG_MSG: std::cell::Cell<u64> = std::cell::Cell::new(0));

#[event(fetch)]
pub async fn main(req: Request, env: Env) -> worker::Result<Response> {
    #[cfg(feature = "debug")]
    {
        LAST_LOG_MSG.with(|last| last.set(worker::Date::now().as_millis()));

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

async fn try_main(req: Request, env: Env) -> Result<Response> {
    // TODO: error handling and error responses
    // TODO: caching header

    // TODO: use a sycamore router for this?
    if req.path() == "/api/v1/paste/" && req.method() == Method::Post {
        return handle_upload(req, &env).await;
    }

    if req.method() != Method::Get {
        return Ok(Response::error("Invalid Method", 405)?);
    }

    if req.path() == "/oembed.json" {
        return Ok(Response::from_json(&Oembed {
            provider_name: "Paste of Exile",
            provider_url: &format!("https://{}", req.url()?.host_str().unwrap()),
        })?);
    }
    if let Some(id) = is_raw_url(&req.path()) {
        return download(&env, id).await;
    }

    let kv = env.kv(consts::KV_STATIC_CONTENT)?;

    if assets::is_asset_path(&req.path()) {
        return assets::serve_asset(req, kv).await;
    }

    let route = app::Route::resolve(&req.path());
    let ctx = build_context(&req, &env, route).await?;

    let head = app::render_head(ctx.clone());
    let (app, rctx) = app::render_to_string(ctx);

    let index = kv.get_asset("index.html").text().await?.unwrap();
    let index = index.replace("<!-- %head% -->", &head);
    let index = index.replace("<!-- %app% -->", &app);

    #[allow(clippy::redundant_clone)]
    Ok(Response::from_html(index.clone())?.with_status(rctx.status_code()))
}
