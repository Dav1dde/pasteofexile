use pob::{PathOfBuilding, SerdePathOfBuilding};
use serde::Serialize;
use worker::{console_log, event, Env, Method, Request, Response, Result};

mod assets;
mod b2;
mod crypto;
mod utils;

async fn handle_upload(mut req: Request, env: &Env) -> Result<Response> {
    let mut data = req.bytes().await?;

    // TODO: proper error handling
    // TODO: maybe shortcut this without actually parsing
    SerdePathOfBuilding::from_export(std::str::from_utf8(&data).unwrap()).unwrap();

    let b2 = b2::B2::from_env(env)?;
    let auth = b2.get_auth_details().await?;
    let upload = b2.get_upload_url(&auth).await?;

    let sha1 = crypto::sha1(&mut data).await?;
    let id = utils::hash_to_short_id(&sha1, 9)?;
    let filename = utils::to_path(&id)?;

    b2.upload(
        &upload,
        &b2::UploadSettings {
            filename: &filename,
            content_type: "text/plain",
            sha1: Some(&utils::hex(&sha1)),
        },
        &mut data,
    )
    .await?;

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

async fn download(env: &Env, id: &str) -> worker::Result<String> {
    let b2 = b2::B2::from_env(env)?;
    console_log!("Downloading: {}", id);

    let path = utils::to_path(id)?;
    let mut response = b2.download(&path).await?;
    if response.status_code() == 200 {
        Ok(response.text().await?)
    } else {
        Err(worker::Error::RustError(
            "failed to download paste".to_owned(),
        ))
    }
}

fn is_raw_url(path: &str) -> Option<&str> {
    path.trim_start_matches('/')
        .split_once('/')
        .filter(|(_, raw)| *raw == "raw")
        .map(|(id, _)| id)
}

#[event(fetch)]
pub async fn main(req: Request, env: Env) -> Result<Response> {
    utils::set_panic_hook();

    // TODO: error handling and error responses
    // TODO: caching header

    // TODO: use a sycamore router for this?
    if req.path() == "/api/v1/paste/" && req.method() == Method::Post {
        return handle_upload(req, &env).await;
    }

    if req.method() != Method::Get {
        return Response::error("Invalid Method", 405);
    }

    if req.path() == "/oembed.json" {
        return Response::from_json(&Oembed {
            provider_name: "Paste of Exile",
            provider_url: &format!("https://{}", req.url()?.host_str().unwrap()),
        });
    }
    if let Some(id) = is_raw_url(&req.path()) {
        let content = download(&env, id).await?;
        return Response::ok(content);
    }

    let kv = env.kv("__STATIC_CONTENT")?;

    if assets::is_asset_path(&req.path()) {
        return assets::serve_asset(req, kv).await;
    }

    let route = app::Route::resolve(&req.path());
    let ctx = build_context(&req, &env, route).await?;

    let index = kv
        .get(&assets::resolve("index.html"))
        .text()
        .await?
        .ok_or_else(|| worker::Error::RustError("index.html does not exist".to_owned()))?;
    let index = index.replace("<!-- %head% -->", &app::render_head(ctx.clone()));
    let index = index.replace("<!-- %app% -->", &app::render_to_string(ctx));

    #[allow(clippy::redundant_clone)]
    Response::from_html(index.clone())
}
