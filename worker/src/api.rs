use crate::{b2, bindgen, crypto, utils, Error, Result};
use pob::{PathOfBuilding, SerdePathOfBuilding};
use serde::Serialize;
use worker::{Env, Method, Request, Response};

#[derive(Serialize)]
struct Upload {
    id: String,
}

pub async fn try_handle(req: &mut Request, env: &Env) -> Result<Option<Response>> {
    // TODO: use a sycamore router for this?
    if req.path() == "/api/v1/paste/" && req.method() == Method::Post {
        return handle_upload(req, env).await.map(Some);
    }
    if let Some(id) = is_download_url(&req.path(), req) {
        return handle_download(env, id).await.map(Some);
    }

    Ok(None)
}

fn is_download_url<'a>(path: &'a str, req: &Request) -> Option<&'a str> {
    if req.method() != Method::Get {
        return None;
    }
    path.trim_start_matches('/')
        .split_once('/')
        .filter(|(_, raw)| *raw == "raw")
        .map(|(id, _)| id)
}

async fn handle_download(env: &Env, id: &str) -> Result<Response> {
    let b2 = b2::B2::from_env(env)?;

    let path = utils::to_path(id)?;
    let response = b2.download(&path).await?;
    match response.status_code() {
        200 => {
            let mut headers = worker::Headers::new();
            headers.set("Content-Type", "text/plain")?;
            headers.set("Cache-Control", "max-age=31536000")?;

            bindgen::Response::dup(response, headers)
        }
        404 => Err(Error::NotFound("paste", id.to_owned())),
        status => Err(Error::RemoteFailed(
            status,
            "failed to get paste".to_owned(),
        )),
    }
}

async fn handle_upload(req: &mut Request, env: &Env) -> Result<Response> {
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
