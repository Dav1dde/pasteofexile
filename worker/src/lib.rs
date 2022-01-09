use worker::{console_log, event, Date, Env, Method, Request, Response, Result};

mod assets;
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

#[event(fetch)]
pub async fn main(mut req: Request, env: Env) -> Result<Response> {
    log_request(&req);

    utils::set_panic_hook();

    if req.path() == "/api/v1/paste/" && req.method() == Method::Post {
        console_log!("Got content: {:?}", req.text().await);
        return Response::ok("YEP");
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
