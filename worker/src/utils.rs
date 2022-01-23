use worker::wasm_bindgen::JsCast;
use worker::worker_sys::WorkerGlobalScope;
use worker::{js_sys, worker_sys, Response, Result};

pub fn hex(data: &[u8]) -> String {
    data.iter().map(|x| format!("{:02X}", x)).collect()
}

pub fn btoa(s: &str) -> Result<String> {
    let worker: WorkerGlobalScope = js_sys::global().unchecked_into();
    Ok(worker.btoa(s)?)
}

pub fn basic_auth(username: &str, password: &str) -> Result<String> {
    let mut s = username.to_owned();
    s.push(':');
    s.push_str(password);

    let mut result = "Basic ".to_owned();
    result.push_str(&btoa(&s)?);
    Ok(result)
}

// pub fn random_id<const N: usize>() -> Result<String> {
//     let random = crate::crypto::get_random_values::<N>()?;
//     Ok(base64::encode_config(random, base64::URL_SAFE_NO_PAD))
// }

pub fn hash_to_short_id(hash: &[u8], bytes: usize) -> Result<String> {
    hash.get(0..bytes)
        .map(|data| base64::encode_config(data, base64::URL_SAFE_NO_PAD))
        .ok_or_else(|| "Hash too small for id".into())
}

pub fn to_path(id: &str) -> Result<String> {
    if !id.is_ascii() || id.len() < 3 {
        return Err("invalid id".into());
    }
    let mut result = String::with_capacity(4 + id.len());
    result.push_str(unsafe { id.get_unchecked(0..1) });
    result.push('/');
    result.push_str(unsafe { id.get_unchecked(1..2) });
    result.push('/');
    result.push_str(unsafe { id.get_unchecked(2..) });
    Ok(result)
}

pub trait ResponseExt: Sized {
    fn cache_for(self, ttl: u32) -> crate::Result<Self> {
        self.with_header("Cache-Control", &format!("max-age={}", ttl))
    }
    fn with_content_type(self, content_type: &str) -> crate::Result<Self> {
        self.with_header("Content-Type", content_type)
    }
    fn with_etag(self, entity_id: &str) -> crate::Result<Self> {
        let entity_id = format!("\"{}\"", entity_id.trim_matches('"'));
        self.with_header("Etag", &entity_id)
    }

    fn dup_headers(self) -> Self;
    fn with_header(self, name: &str, value: &str) -> crate::Result<Self>;

    fn cloned(self) -> crate::Result<(Self, Self)>;
}

impl ResponseExt for Response {
    fn dup_headers(self) -> Self {
        let headers = self.headers().clone();
        self.with_headers(headers)
    }

    fn with_header(mut self, name: &str, value: &str) -> crate::Result<Self> {
        self.headers_mut().set(name, value)?;
        Ok(self)
    }

    fn cloned(self) -> crate::Result<(Self, Self)> {
        let headers = self.headers().clone();

        let response1: worker_sys::Response = self.into();
        let response2 = response1.clone()?;

        let body1 = worker::ResponseBody::Stream(response1);
        let body2 = worker::ResponseBody::Stream(response2);

        let response1 = worker::Response::from_body(body1)?.with_headers(headers.clone());
        let response2 = worker::Response::from_body(body2)?.with_headers(headers);

        Ok((response1, response2))
    }
}
