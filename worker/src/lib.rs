use pob::{PathOfBuilding, SerdePathOfBuilding};
use serde::Serialize;
use worker::{event, Env, Method, Request, Response, Result};

mod assets;
mod b2;
mod crypto;
mod utils;

async fn handle_upload(mut req: Request, env: &Env) -> Result<Response> {
    let mut data = req.bytes().await?;

    worker::console_log!("{:?}", worker::Date::now().as_millis());
    // TODO: proper error handling
    // TODO: maybe shortcut this without actually parsing
    SerdePathOfBuilding::from_export(std::str::from_utf8(&data).unwrap()).unwrap();
    worker::console_log!("{:?}", worker::Date::now().as_millis());

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

    Response::from_json(&Upload { id })
}

#[derive(Serialize)]
struct Upload {
    id: String,
}

async fn build_context(env: &Env, route: app::Route) -> Result<app::Context> {
    use app::{Context, Route::*};
    let ctx = match route {
        Index => Context::index(),
        NotFound => Context::not_found(),
        Paste(name) => {
            let b2 = b2::B2::from_env(env)?;

            match utils::to_path(&name) {
                Err(_) => Context::not_found(),
                Ok(path) => {
                    let mut response = b2.download(&path).await?;
                    if response.status_code() == 200 {
                        let content = response.text().await?;
                        Context::paste(name, content)
                    } else {
                        Context::not_found()
                    }
                }
            }
        }
    };

    Ok(ctx)
}

#[event(fetch)]
pub async fn main(req: Request, env: Env) -> Result<Response> {
    utils::set_panic_hook();

    // TODO: use a sycamore router for this?
    if req.path() == "/api/v1/paste/" && req.method() == Method::Post {
        return handle_upload(req, &env).await;
    }

    if req.method() != Method::Get {
        return Response::error("Invalid Method", 405);
    }

    let kv = env.kv("__STATIC_CONTENT")?;

    if assets::is_asset_path(&req.path()) {
        return assets::serve_asset(req, kv).await;
    }

    let route = app::Route::resolve(&req.path());
    let ctx = build_context(&env, route).await?;

    let index = kv.get("index.html").text().await?.expect("index html");
    let index = index.replace("%app%", &app::render_to_string(ctx));

    Response::from_html(index)
}
