use std::borrow::Cow;

use wasm_bindgen::JsValue;
pub use worker::Method;

use crate::statsd::Counters;

pub struct Request<'a> {
    url: Cow<'a, str>,
    init: worker::RequestInit,
    sentry: bool,
    tag: Option<&'static str>,
}

impl<'a> Request<'a> {
    fn new(url: Cow<'a, str>) -> Self {
        Self {
            url,
            init: worker::RequestInit::new(),
            sentry: true,
            tag: None,
        }
    }

    pub fn get(url: impl Into<Cow<'a, str>>) -> Self {
        Self::new(url.into()).method(Method::Get)
    }

    pub fn post(url: impl Into<Cow<'a, str>>) -> Self {
        Self::new(url.into()).method(Method::Post)
    }

    pub fn method(mut self, method: Method) -> Self {
        self.init.method = method;
        self
    }

    pub fn tag(mut self, tag: &'static str) -> Self {
        self.tag = Some(tag);
        self
    }

    pub fn header(mut self, name: &str, value: &str) -> Self {
        let r = self.init.headers.set(name, value);
        debug_assert!(r.is_ok());
        self
    }

    pub fn header_opt(self, name: &str, value: Option<&str>) -> Self {
        if let Some(value) = value {
            self.header(name, value)
        } else {
            self
        }
    }

    pub fn body(mut self, body: impl Into<JsValue>) -> Self {
        self.init.body = Some(body.into());
        self
    }

    pub fn body_u8(self, body: &'a [u8]) -> Self {
        // I think this is safe - the lifetime of the body is tied to the request, so it *should*
        // live long enough ...
        self.body(unsafe { js_sys::Uint8Array::view(body) })
    }

    pub fn no_sentry(mut self) -> Self {
        self.sentry = false;
        self
    }

    pub async fn send(self) -> worker::Result<worker::Response> {
        let request = worker::Request::new_with_init(&self.url, &self.init)?;
        let response = worker::Fetch::Request(request).send().await;

        if self.sentry {
            let status_code = response
                .as_ref()
                .ok()
                .map(|r| r.status_code())
                .unwrap_or(570);

            let mut data = sentry::Map::new();
            data.insert("url".into(), self.url.into());
            data.insert("method".into(), self.init.method.to_string().into());
            data.insert("status_code".into(), status_code.into());
            // reason is not directly exposed from response

            sentry::add_breadcrumb(sentry::Breadcrumb {
                ty: Some("http".into()),
                category: Some("fetch".into()),
                message: response.as_ref().err().map(|e| e.to_string()),
                data,
                ..Default::default()
            });

            sentry::counter(Counters::Fetch)
                .inc(1)
                .tag("status", status_code)
                .tag("method", method_as_str(self.init.method))
                .tag_opt("tag", self.tag);
        }

        response
    }
}

fn method_as_str(method: worker::Method) -> &'static str {
    match method {
        Method::Head => "head",
        Method::Get => "get",
        Method::Post => "post",
        Method::Put => "put",
        Method::Patch => "patch",
        Method::Delete => "delete",
        Method::Options => "options",
        Method::Connect => "connect",
        Method::Trace => "trace",
    }
}
