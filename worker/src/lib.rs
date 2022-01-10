use worker::{event, Env, Method, Request, Response, Result};

mod assets;
mod b2;
mod crypto;
mod utils;

async fn handle_upload(mut req: Request, env: &Env) -> Result<Response> {
    let b2 = b2::B2::from_env(env)?;
    let auth = b2.get_auth_details().await?;
    let upload = b2.get_upload_url(&auth).await?;

    let mut data = req.bytes().await?;
    let response = b2
        .upload(
            &upload,
            &b2::UploadSettings {
                file_name: "foo",
                content_type: "text/plain",
            },
            &mut data,
        )
        .await?;
    Response::from_json(&response)
}

async fn build_context(env: &Env, route: app::Route) -> Result<app::Context> {
    use app::{Context, Route::*};
    let ctx = match route {
        Index => Context::index(),
        NotFound => Context::not_found(),
        Paste(name) => {
            let b2 = b2::B2::from_env(env)?;

            let mut response = b2.download(&name).await?;
            if response.status_code() == 200 {
                let content = response.text().await?;
                Context::paste(name, content)
            } else {
                Context::not_found()
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
