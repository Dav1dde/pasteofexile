use crate::{consts, utils::ResponseExt, Error, Result};
use std::borrow::Cow;
use wasm_bindgen::prelude::*;
use worker::{
    kv::{self, KvStore},
    Env, Request, Response,
};

pub trait KvAssetExt {
    fn get_asset(&self, name: &str) -> kv::GetOptionsBuilder;
}

impl KvAssetExt for KvStore {
    fn get_asset(&self, name: &str) -> kv::GetOptionsBuilder {
        self.get(&resolve(name))
    }
}

pub trait EnvAssetExt {
    fn get_asset(&self, name: &str) -> Result<kv::GetOptionsBuilder>;
    fn get_assets(&self) -> Result<kv::KvStore>;
}

impl EnvAssetExt for Env {
    fn get_asset(&self, name: &str) -> Result<kv::GetOptionsBuilder> {
        Ok(self.get_assets()?.get_asset(name))
    }

    fn get_assets(&self) -> Result<kv::KvStore> {
        Ok(self.kv(consts::KV_STATIC_CONTENT)?)
    }
}

pub async fn try_handle(req: &mut Request, env: &Env) -> Result<Option<Response>> {
    // does the last segment contain a '.'
    let is_asset_path = req
        .path()
        .rsplit_once('/')
        .map(|x| x.1)
        .unwrap_or(&req.path())
        .contains('.');

    if is_asset_path {
        Ok(Some(serve_asset(req, env).await?))
    } else {
        Ok(None)
    }
}

async fn serve_asset(req: &Request, env: &Env) -> Result<Response> {
    let path = req.path();
    let path = path.trim_start_matches('/');
    let path = resolve(path);
    let value = match env.get_asset(&path)?.bytes().await? {
        Some(value) => value,
        None => return Err(Error::NotFound("asset", path.to_string())),
    };

    Response::from_bytes(value)?
        .with_content_type(get_mime(&path).unwrap_or("text/plain"))?
        .cache_for(consts::CACHE_ASSETS)?
        .with_etag(&path)
}

#[wasm_bindgen(raw_module = "./assets.mjs")]
extern "C" {
    fn get_asset(name: &str) -> Option<String>;
}

fn resolve(name: &str) -> Cow<'_, str> {
    match get_asset(name) {
        Some(name) => Cow::Owned(name),
        None => Cow::Borrowed(name),
    }
}

fn get_mime(path: &str) -> Option<&'static str> {
    let ext = if let Some((_, ext)) = path.rsplit_once('.') {
        ext
    } else {
        return None;
    };

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
