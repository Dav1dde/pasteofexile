use crate::{
    consts, crypto, dangerous, storage,
    utils::{self, ResponseExt},
    Error, Result,
};
use pob::SerdePathOfBuilding;
use serde::Serialize;
use worker::{Context, Env, Headers, Method, Request, Response};

#[derive(Serialize)]
struct Upload {
    id: String,
}

pub async fn try_handle(ctx: &Context, req: &mut Request, env: &Env) -> Result<Option<Response>> {
    // TODO: use a sycamore router for this?
    if req.path() == "/api/v1/paste/" && req.method() == Method::Post {
        return handle_upload(ctx, req, env, false).await.map(Some);
    }
    if req.path() == "/pob/" && req.method() == Method::Post {
        return handle_upload(ctx, req, env, true).await.map(Some);
    }
    if let Some(id) = is_download_url(&req.path(), req) {
        return handle_download(env, id).await.map(Some);
    }
    if req.path() == "/login" && req.method() == Method::Get {
        return handle_login(req).await.map(Some);
    }
    if req.path() == "/oauth2/authorization/poe" && req.method() == Method::Get {
        return handle_oauth2_poe(env, req).await.map(Some);
    }

    Ok(None)
}

fn is_download_url<'a>(path: &'a str, req: &Request) -> Option<&'a str> {
    if req.method() != Method::Get {
        return None;
    }

    is_pob_download_url(path).or_else(|| is_raw_download_url(path))
}

fn is_raw_download_url(path: &str) -> Option<&str> {
    path.trim_start_matches('/')
        .split_once('/')
        .filter(|(_, raw)| *raw == "raw")
        .map(|(id, _)| id)
        .filter(|id| !id.is_empty())
}

fn is_pob_download_url(path: &str) -> Option<&str> {
    path.rsplit_once('/')
        .filter(|(path, _)| *path == "/pob")
        .map(|(_, id)| id)
        .filter(|id| !id.is_empty())
}

async fn handle_download(env: &Env, id: &str) -> Result<Response> {
    let path = utils::to_path(id)?;

    let storage = storage::DefaultStorage::from_env(env)?;

    let response = storage
        .get(&path)
        .await?
        .ok_or_else(|| Error::NotFound("paste", id.to_owned()))?;

    response
        .with_headers(Headers::new())
        .with_content_type("text/plain")?
        .cache_for(31536000)
}

async fn handle_upload(
    ctx: &Context,
    req: &mut Request,
    env: &Env,
    is_pob: bool,
) -> Result<Response> {
    let mut data = req.bytes().await?;

    if data.len() > consts::MAX_UPLOAD_SIZE {
        return Err(Error::BadRequest("Paste too large".to_owned()));
    }

    let s = std::str::from_utf8(&data)
        .map_err(|_| "invalid content".to_owned())
        .map_err(Error::BadRequest)?;

    // Generic 401, probably just actually bad data
    let s = pob::decompress(s).map_err(|e| Error::BadRequest(e.to_string()))?;
    // More specific error for a separate Sentry categoy
    let _ =
        SerdePathOfBuilding::from_xml(&s).map_err(move |e| Error::InvalidPoB(e.to_string(), s))?;

    let sha1 = crypto::sha1(&mut data).await?;
    let id = utils::hash_to_short_id(&sha1, 9)?;
    let filename = utils::to_path(&id)?;

    let storage = storage::DefaultStorage::from_env(env)?;

    log::debug!("--> uploading paste '{}' to '{}'", id, filename);

    if is_pob {
        storage.put_async(ctx, filename, &sha1, data).await?;
        log::debug!("<-- paste uploaing ...");
        return Ok(Response::ok(id)?);
    }

    log::debug!("--> uploading paste '{}' to '{}'", id, filename);
    storage.put(&filename, &sha1, &mut data).await?;
    log::debug!("<-- paste uploaded");

    let response = serde_json::to_string(&Upload { id })?;
    let mut response = Response::from_bytes(response.into_bytes())?;
    response
        .headers_mut()
        .set("Content-Type", "application/json")?;
    Ok(response)
}

async fn handle_login(req: &Request) -> Result<Response> {
    let url = req.url()?;
    let host = url.host_str().unwrap();
    let state = utils::random_string::<16>()?;
    let redirect = format!("https://www.pathofexile.com/oauth/authorize?client_id=pobbin&response_type=code&scope=account:profile&state={state}&redirect_uri=https://{host}/oauth2/authorization/poe&prompt=consent");

    Response::empty()?
        .with_status(307)
        .with_header("Location", &redirect)?
        .with_header(
            "Set-Cookie",
            &format!("state={state}; Max-Age=600; Secure; Same-Site=Lax; Path=/"),
        )
}

async fn handle_oauth2_poe(env: &Env, req: &Request) -> Result<Response> {
    let mut code = None;
    let mut state = None;
    let url = req.url()?;
    for (k, v) in url.query_pairs() {
        match &*k {
            "code" => code = Some(v),
            "state" => state = Some(v),
            _ => {}
        }
    }

    let (code, state) = match (code, state) {
        (Some(code), Some(state)) => (code, state),
        _ => return Ok(Response::error("lol", 403)?),
    };

    if !req
        .headers()
        .get("Cookie")?
        .unwrap_or_default()
        .contains(&*state)
    {
        return Ok(Response::error("missing stuff", 403)?);
    }

    let mut headers = Headers::new();
    headers.set("Content-Type", "application/x-www-form-urlencoded")?;

    let secret = env.var(consts::ENV_OAUTH_SECRET_POE)?.to_string();

    let body = wasm_bindgen::JsValue::from_str(&format!(
        "client_id=pobbin&client_secret={secret}&grant_type=authorization_code&code={code}&scope=account:profile"
    ));

    let request = Request::new_with_init(
        "https://www.pathofexile.com/oauth/token",
        &worker::RequestInit {
            method: Method::Post,
            headers,
            body: Some(body),
            ..Default::default()
        },
    )?;

    #[derive(serde::Deserialize)]
    struct OauthTokenResponse {
        access_token: String,
    }

    let r: OauthTokenResponse = worker::Fetch::Request(request).send().await?.json().await?;

    let mut headers = Headers::new();
    headers.set("Authorization", &format!("Bearer {}", r.access_token))?;
    headers.set("User-Agent", "User-Agent: OAuth pobbin/1.0 (contact: TODO)")?; // TODO

    #[derive(serde::Deserialize, serde::Serialize, Debug)]
    struct Profile {
        name: String,
    }

    let profile = worker::Fetch::Request(Request::new_with_init(
        "https://api.pathofexile.com/profile",
        &worker::RequestInit {
            method: Method::Get,
            headers,
            ..Default::default()
        },
    )?)
    .send()
    .await?
    .json::<Profile>()
    .await?;

    let secret = env.var(consts::ENV_SECRET_KEY)?.to_string();
    let session = dangerous::Dangerous::new(secret.into_bytes())
        .sign(&profile)
        .await?;

    Response::empty()?
        .with_status(307)
        .with_header(
            "Set-Cookie",
            "state=none; Max-Age=0; Secure; Same-Site=Lax; Path=/",
        )?
        .with_header(
            "Set-Cookie",
            &format!("session={session}; Max-Age=1209600; Secure; SameSite=Lax; Path=/"),
        )?
        .with_header("Location", &format!("/u/{}", profile.name))
}
