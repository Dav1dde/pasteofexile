use crate::{Error, Result};
use std::borrow::Cow;
use wasm_bindgen::prelude::*;
use worker::{
    kv::{self, KvStore},
    Request, Response,
};

pub trait KvAssetExt {
    fn get_asset(&self, name: &str) -> kv::GetOptionsBuilder;
}

impl KvAssetExt for KvStore {
    fn get_asset(&self, name: &str) -> kv::GetOptionsBuilder {
        self.get(&resolve(name))
    }
}

pub fn is_asset_path(path: &str) -> bool {
    let last_segment = path.rsplit_once("/").map(|x| x.1).unwrap_or(path);
    last_segment.contains('.')
}

pub async fn serve_asset(req: Request, store: KvStore) -> Result<Response> {
    let path = req.path();
    let path = path.trim_start_matches('/');
    let path = resolve(path);
    let value = match store.get(&path).bytes().await? {
        Some(value) => value,
        None => return Err(Error::NotFound("asset", path.to_string())),
    };
    #[allow(clippy::redundant_clone)]
    let mut response = Response::from_bytes(value.clone())?;
    response
        .headers_mut()
        .set("Content-Type", get_mime(&path).unwrap_or("text/plain"))?;
    Ok(response)
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
    let ext = if let Some((_, ext)) = path.rsplit_once(".") {
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
        "wasm" => "application/wasm",
        _ => return None,
    };

    Some(ct)
}
