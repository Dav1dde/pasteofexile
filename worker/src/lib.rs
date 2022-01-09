use worker::{console_log, event, Date, Env, Method, Request, Response, Result};

mod assets;
mod b2;
mod crypto;
mod utils;

fn log_request(req: &worker::Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf()
            .region()
            .unwrap_or_else(|| "unknown region".to_owned())
    );
}

async fn handle_upload(mut req: Request, env: Env) -> Result<Response> {
    let b2 = b2::B2::from_env(&env)?;
    let auth = b2.get_auth_details().await?;
    let upload = b2.get_upload_url(&auth).await?;

    let mut data = req.bytes().await?;
    let response = b2.upload(
        &upload,
        &b2::UploadSettings {
            file_name: "foo",
            content_type: "text/plain",
        },
        &mut data,
    ).await?;
    return Response::from_json(&response);
}

#[event(fetch)]
pub async fn main(req: Request, env: Env) -> Result<Response> {
    log_request(&req);

    utils::set_panic_hook();

    if req.path() == "/api/v1/paste/" && req.method() == Method::Post {
        return handle_upload(req, env).await;
    }
    if req.path() == "/api/v1/paste/" && req.method() == Method::Get {
        let b2 = b2::B2::from_env(&env)?;
        return b2.download("foo").await;
    }

    if req.method() != Method::Get {
        return Response::error("Invalid Method", 405);
    }

    let kv = env.kv("__STATIC_CONTENT")?;

    if assets::is_asset_path(&req.path()) {
        return assets::serve_asset(req, kv).await;
    }

    let index = kv.get("index.html").text().await?.expect("index html");
    let index = index.replace("%app%", &app::render_to_string(req.path()));

    // for some reason this clone is required to not turn the html to garbage ???
    #[allow(clippy::redundant_clone)]
    Response::from_html(index.clone())
}
