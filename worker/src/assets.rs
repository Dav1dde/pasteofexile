use crate::{consts, request_context::RequestContext, utils::ResponseExt, Error, Result};
use std::borrow::Cow;
use wasm_bindgen::prelude::*;
use worker::Response;

pub async fn try_handle(rctx: &RequestContext) -> Result<Option<Response>> {
    if is_asset_path(&rctx.path()) {
        Ok(Some(serve_asset(rctx).await?))
    } else {
        Ok(None)
    }
}

pub fn is_asset_path(path: &str) -> bool {
    get_mime(path).is_some()
}

#[tracing::instrument(skip_all)]
async fn serve_asset(rctx: &RequestContext) -> Result<Response> {
    let path = rctx.path();
    let path = path.trim_start_matches('/');
    let path = resolve(path);
    let value = match rctx.get_asset(&path)?.bytes().await? {
        Some(value) => value,
        None => return Err(Error::NotFound("asset", path.to_string())),
    };

    Response::from_bytes(value)?
        .with_content_type(get_mime(&path).unwrap_or("text/plain"))?
        .cache_for(consts::CACHE_FOREVER)
}

#[wasm_bindgen(raw_module = "./assets.mjs")]
extern "C" {
    fn get_asset(name: &str) -> Option<String>;
}

pub(crate) fn resolve(name: &str) -> Cow<'_, str> {
    match get_asset(name) {
        Some(name) => Cow::Owned(name),
        None => Cow::Borrowed(name),
    }
}

fn get_mime(path: &str) -> Option<&'static str> {
    let (_, ext) = path.rsplit_once('.')?;

    let ct = match ext {
        "html" => "text/html",
        "css" => "text/css",
        "js" => "text/javascript",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        "ico" => "image/x-icon",
        "svg" => "image/svg+xml",
        "wasm" => "application/wasm",
        _ => return None,
    };

    Some(ct)
}
