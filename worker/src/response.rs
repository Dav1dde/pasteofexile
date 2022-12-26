use std::time::Duration;

use serde::Serialize;

pub use self::ResponseError::{ApiError, AppError};
use crate::utils::CacheControl;

pub type Result = std::result::Result<Response, ResponseError>;

#[derive(Debug)]
pub enum ResponseError {
    ApiError(crate::Error),
    AppError(crate::Error),
}

impl ResponseError {
    pub fn inner(&self) -> &crate::Error {
        match self {
            Self::ApiError(err) => err,
            Self::AppError(err) => err,
        }
    }
}

pub struct Response {
    status_code: u16,
    headers: worker::Headers,
    body: worker::ResponseBody,
}

impl Response {
    pub fn ok() -> Self {
        Self::status(200)
    }

    pub fn not_found() -> Self {
        Self::status(404)
    }

    pub fn status(status_code: u16) -> Self {
        Self {
            status_code,
            headers: worker::Headers::new(),
            body: worker::ResponseBody::Empty,
        }
    }

    pub fn redirect_temp(location: &str) -> Self {
        Self::status(307).header("Location", location)
    }

    pub fn redirect_perm(location: &str) -> Self {
        Self::status(301).header("Location", location)
    }

    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = worker::ResponseBody::Body(body.into());
        self
    }

    pub fn json(self, body: &impl Serialize) -> Self {
        self.body(serde_json::to_vec(body).unwrap()) // TODO: unwrap
    }

    pub fn html(self, body: impl Into<String>) -> Self {
        self.body(body.into().into_bytes())
    }

    pub fn header(mut self, name: &str, value: &str) -> Self {
        let r = self.headers.set(name, value);
        debug_assert!(r.is_ok());
        self
    }

    pub fn append_header(mut self, name: &str, value: &str) -> Self {
        let r = self.headers.append(name, value);
        debug_assert!(r.is_ok());
        self
    }

    pub fn content_type(self, content_type: &str) -> Self {
        self.header("Content-Type", content_type)
    }

    pub fn etag<'a, T>(self, etag: T) -> Self
    where
        T: Into<Option<&'a str>>,
    {
        let Some(etag) = etag.into() else { return self };
        // TODO: modify etag with git commit hash?
        self.header("Etag", &format!("\"{}\"", etag.trim_matches('"')))
    }

    pub fn state_cookie(self, state: &str) -> Self {
        self.append_header(
            "Set-Cookie",
            &format!("state={state}; Max-Age=600; Secure; Same-Site=Lax; Path=/"),
        )
    }

    pub fn delete_state_cookie(self) -> Self {
        self.append_header(
            "Set-Cookie",
            "state=none; Max-Age=0; Secure; Same-Site=Lax; Path=/",
        )
    }

    pub fn new_session(self, session: &str) -> Self {
        self.append_header(
            "Set-Cookie",
            &format!("session={session}; Max-Age=1209600; Secure; SameSite=Lax; Path=/"),
        )
    }

    pub fn cache(self, cache_control: CacheControl) -> Self {
        self.header("Cache-Control", &cache_control.to_string())
    }

    pub fn cache_for(self, ttl: Duration) -> Self {
        self.cache(CacheControl::default().public().max_age(ttl))
    }

    pub fn result<T>(self) -> std::result::Result<Self, T> {
        Ok(self)
    }
}

// Utility Methods that do not modify the response.
impl Response {
    /// Whether the response currently can be cached.
    ///
    /// A response without caching headers is not cacheable,
    /// becaue it wouldn't be cached for any duration.
    ///
    /// Also returns false when the response has status code 206
    /// or contains the `Vary: *` header.
    ///
    /// See also: https://developers.cloudflare.com/workers/runtime-apis/cache/#parameters
    pub fn is_cacheable(&self) -> bool {
        if self.status_code == 206 {
            return false;
        }
        if self.headers.get("Vary").unwrap().as_deref() == Some("*") {
            return false;
        }
        ["Cache-Control", "ETag", "Expires", "Last-Modified"]
            .into_iter()
            .any(|hn| self.headers.has(hn).unwrap())
    }
}

impl Clone for Response {
    fn clone(&self) -> Self {
        let body = match &self.body {
            worker::ResponseBody::Empty => worker::ResponseBody::Empty,
            worker::ResponseBody::Body(v) => worker::ResponseBody::Body(v.clone()),
            worker::ResponseBody::Stream(s) => {
                worker::ResponseBody::Stream(s.clone().expect("response body already used?"))
            }
        };

        Self {
            status_code: self.status_code,
            // Cloning headers maybe sucks here, but is also good
            // because headers can actually be read only, so cloning get's rid of that limitation
            headers: self.headers.clone(),
            body,
        }
    }
}

impl From<Response> for worker::Response {
    fn from(r: Response) -> worker::Response {
        worker::Response::from_body(r.body)
            .unwrap()
            .with_status(r.status_code)
            .with_headers(r.headers)
    }
}

impl From<worker::Response> for Response {
    fn from(wr: worker::Response) -> Response {
        let headers = wr.headers().clone();

        let mut r = Response::status(wr.status_code());
        r.headers = headers;
        r.body = worker::ResponseBody::Stream(worker::worker_sys::Response::from(wr));

        r
    }
}
