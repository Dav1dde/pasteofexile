use std::collections::HashMap;

use once_cell::sync::OnceCell;
use wasm_bindgen::prelude::*;
use worker::kv::KvStore;

use crate::{
    consts,
    request_context::{Env, FromEnv, RequestContext},
    response, Error, Response, Result,
};

#[tracing::instrument(skip_all)]
pub async fn handle(rctx: &RequestContext) -> response::Result {
    serve_asset(rctx).await.map_err(response::AppError)
}

async fn serve_asset(rctx: &RequestContext) -> Result<Response> {
    let path = rctx.path();

    let Some(mime_type) = get_mime(&path) else {
        return Err(Error::NotFound("asset", path.to_string()));
    };

    let assets = rctx.inject::<Assets>();
    let Some(value) = assets.get(&path).bytes().await? else {
        return Err(Error::NotFound("asset", path.to_string()));
    };

    Response::ok()
        .body(value)
        .content_type(mime_type)
        .cache_for(consts::CACHE_FOREVER)
        .result()
}

pub fn is_asset_path(path: &str) -> bool {
    get_mime(path).is_some()
}

struct Assets {
    kv: KvStore,
}

impl FromEnv for Assets {
    fn from_env(env: &Env) -> Option<Self> {
        let kv = env.kv(consts::KV_STATIC_CONTENT)?;
        Some(Self { kv })
    }
}

impl Assets {
    fn get(&self, path: &str) -> worker::kv::GetOptionsBuilder {
        let path = self.resolve(path.trim_start_matches('/'));
        self.kv.get(path)
    }

    fn resolve<'a>(&self, name: &'a str) -> &'a str {
        get_asset(name).unwrap_or(name)
    }
}

#[wasm_bindgen(module = "__STATIC_CONTENT_MANIFEST")]
extern "C" {
    #[wasm_bindgen(js_name = "default")]
    static STATIC_CONTENT_MANIFEST: String;
}

fn get_asset(name: &str) -> Option<&'static str> {
    static MANIFEST: OnceCell<HashMap<&str, &str>> = OnceCell::new();

    let manifest =
        MANIFEST.get_or_init(|| serde_json::from_str(&STATIC_CONTENT_MANIFEST).unwrap_throw());

    manifest.get(name).copied()
}

fn get_mime(path: &str) -> Option<&'static str> {
    let (_, ext) = path.rsplit_once('.')?;

    let ct = match ext {
        "css" => "text/css",
        "html" => "text/html",
        "ico" => "image/x-icon",
        "jpeg" => "image/jpeg",
        "jpg" => "image/jpeg",
        "js" => "text/javascript",
        "json" => "application/json",
        "png" => "image/png",
        "svg" => "image/svg+xml",
        "txt" => "text/plain",
        "wasm" => "application/wasm",
        _ => return None,
    };

    Some(ct)
}
