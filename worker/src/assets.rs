use std::borrow::Cow;

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
        self.kv.get(&path)
    }

    fn resolve<'a>(&self, name: &'a str) -> Cow<'a, str> {
        match get_asset(name) {
            Some(name) => Cow::Owned(name),
            None => Cow::Borrowed(name),
        }
    }
}

#[wasm_bindgen(raw_module = "./assets.mjs")]
extern "C" {
    fn get_asset(name: &str) -> Option<String>;
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
