use std::{rc::Rc, cell::RefCell};

use git_version::git_version;
use wasm_bindgen::JsValue;
use worker::{
    worker_sys::Request as EdgeRequest, Context, Env, Fetch, Headers, Method, RequestInit,
};

use crate::{consts, Error, Result, sentry::protocol::Timestamp, utils::RequestExt};

use super::protocol;


#[derive(Clone)]
pub struct Sentry {
    ctx: Context,
    req: EdgeRequest,
    token: String,
    store_url: String,
    envelope_url: String,
    breadcrumbs: Vec<protocol::Breadcrumb>,
    trace_context: Vec<protocol::TraceContext>,
    pub(crate) trace_id: protocol::TraceId,
    transaction: protocol::Transaction<'static>,
}

impl Sentry {
    pub fn from_env(env: &Env, ctx: &Context, req: &worker::Request) -> Option<Self> {
        let project = env.var(consts::ENV_SENTRY_PROJECT).ok()?.to_string();
        let store_url = format!("https://sentry.io/api/{}/store/", project);
        let envelope_url = format!("https://sentry.io/api/{}/envelope/", project);
        let req_inner = req.inner();
        Some(Self {
            ctx: ctx.clone(),
            req: Clone::clone(req_inner),
            token: env.var(consts::ENV_SENTRY_TOKEN).ok()?.to_string(),
            store_url,
            envelope_url,
            breadcrumbs: Vec::new(),
            trace_context: Vec::new(),
            trace_id: protocol::TraceId::default(),
            transaction: protocol::Transaction {
                name: Some("test2".to_string()),
                platform: "other".into(),
                request: Some(protocol::Request {
                    url: req.url().ok(),
                    method: Some(req_inner.method()),
                    headers: req.headers().into_iter().collect(),
                    ..Default::default()
                }),
                ..Default::default()
            },
        })
    }

    pub(crate) fn push_trace_context(&mut self, trace_context: protocol::TraceContext) {
        self.trace_context.push(trace_context);
    }

    pub(crate) fn pop_trace_context(&mut self) {
        self.trace_context.pop();
    }

    pub(crate) fn add_span(&mut self, span: protocol::Span) {
        self.transaction.spans.push(span);
    }

    pub(crate) fn add_breadcrumb(&mut self, breadcrumb: protocol::Breadcrumb) {
        self.breadcrumbs.push(breadcrumb);
    }

    pub(crate) fn finish_transaction(&self) {
        let mut transaction = self.transaction.clone();
        transaction.timestamp = Some(protocol::Timestamp::now());

        transaction.contexts.insert("trace".into(), protocol::Context::Trace(Box::new(protocol::TraceContext {
            span_id: self.transaction.spans[1].span_id,
            trace_id: self.transaction.spans[1].trace_id,
            parent_span_id: Some(self.transaction.spans[0].span_id),
            op: Some("my_op".to_owned()),
            description: Some("my_desc".to_owned()),
            ..Default::default()
        })));

        let r = self.send_envelope(transaction.into());
        worker::console_log!("--> {:?} {r:?}", self.transaction);
    }

    pub(crate) fn capture_event(&self, event: protocol::Event) {
        // TODO add trace context to event
    }

    pub fn capture_err(&self, err: &Error) {
        if let Err(err) = self.do_capture_err(err, err.level()) {
            log::warn!("failed to caputre error with sentry: {:?}", err);
        }
    }

    pub fn capture_err_level(&self, err: &Error, level: &'static str) {
        if let Err(err) = self.do_capture_err(err, level) {
            log::warn!("failed to caputre error with sentry: {:?}", err);
        }
    }

    pub(crate) fn send_envelope(&self, envelope: protocol::Envelope) -> Result<()> {
        let mut headers = Headers::new();
        headers.set("Content-Type", "application/x-sentry-envelope")?;
        headers.set("User-Agent", "pobb.bin/1.0")?;
        headers.set(
            "X-Sentry-Auth",
            &format!(
                "Sentry sentry_version=7, sentry_client=pobb.in/1.0, sentry_key={}",
                self.token,
            ),
        )?;

        let mut body = Vec::new();
        envelope.to_writer(&mut body)?;

        worker::console_log!("{}", String::from_utf8_lossy(&body));

        let request = worker::Request::new_with_init(
            &self.envelope_url,
            &RequestInit {
                method: Method::Post,
                headers,
                body: Some(unsafe { js_sys::Uint8Array::view(&body) }.into()),
                ..Default::default()
            },
        )?;

        self.ctx.wait_until(async move {
            let r = Fetch::Request(request).send().await;
            match r {
                Err(err) => worker::console_log!("failed to caputre error with sentry: {:?}", err),
                Ok(mut r) => {
                    worker::console_log!("successfully captured error: {:?}", r);
                    worker::console_log!("-> {:?}", r.text().await.ok());
                }
            }
        });

        Ok(())
    }

    fn do_capture_err(&self, err: &Error, level: &'static str) -> Result<()> {
        let req: worker::Request = Clone::clone(&self.req).into();

        let mut event = protocol::Event {
            transaction: Some("test2".to_owned()),
            message: Some("message2".to_owned()),
            breadcrumbs: self.breadcrumbs.clone(),
            request: Some(protocol::Request {
                url: req.url().ok(),
                method: Some(self.req.method()),
                headers: req.headers().into_iter().collect(),
                ..Default::default()
            }),
            user: Some(protocol::User {
                username: None, // TODO get user name from session
                ip_address: req.headers().get("cf-connecting-ip")?,
                country: req.headers().get("cf-ipcountry")?,
            }),
            ..Default::default()
        };

        event.contexts.insert("trace".into(), protocol::Context::Trace(Box::new(protocol::TraceContext {
            span_id: self.transaction.spans[1].span_id,
            trace_id: self.transaction.spans[1].trace_id,
            parent_span_id: Some(self.transaction.spans[0].span_id),
            op: self.transaction.spans[1].op.clone(),
            description: self.transaction.spans[1].description.clone(),
            ..Default::default()
        })));

        let r = self.send_envelope(event.into());
        worker::console_log!("--> {:?} {r:?}", self.transaction);

        Ok(())
    }

    // fn do_capture_err(&self, err: &Error, level: &'static str) -> Result<()> {
    //     let mut headers = Headers::new();
    //     headers.set("Content-Type", "application/json")?;
    //     headers.set("User-Agent", "pobb.bin/1.0")?;
    //     headers.set(
    //         "X-Sentry-Auth",
    //         &format!(
    //             "Sentry sentry_version=7, sentry_client=pobb.in/1.0, sentry_key={}",
    //             self.token,
    //         ),
    //     )?;

    //     let req: worker::Request = Clone::clone(&self.req).into();
    //     let url = req.url()?;

    //     let body = JsValue::from_str(&serde_json::to_string(&protocol::Store {
    //         logger: "worker",
    //         platform: "other",
    //         level,
    //         // exception: &protocol::Exception {
    //         //     values: &[ExceptionValue {
    //         //         r#type: err.name(),
    //         //         value: &err.to_string(),
    //         //     }],
    //         // },
    //         // request: &protocol::Request {
    //         //     url: &req.inner().url(),
    //         //     method: &req.inner().method(),
    //         //     headers: req.headers().into_iter().collect(),
    //         //     data: err.payload(),
    //         // },
    //         user: &protocol::User {
    //             ip_address: &req.headers().get("cf-connecting-ip")?.unwrap_or_default(),
    //             country: &req.headers().get("cf-ipcountry")?.unwrap_or_default(),
    //         },
    //         release: git_version!(),
    //         server_name: url.host_str().unwrap_or(""),
    //         // TODO: use route information here
    //         transaction: "",
    //     })?);

    //     let request = worker::Request::new_with_init(
    //         &self.store_url,
    //         &RequestInit {
    //             method: Method::Post,
    //             headers,
    //             body: Some(body),
    //             ..Default::default()
    //         },
    //     )?;

    //     self.ctx.wait_until(async move {
    //         let r = Fetch::Request(request).send().await;
    //         if let Err(err) = r {
    //             log::warn!("failed to caputre error with sentry: {:?}", err);
    //         } else {
    //             log::info!("successfully captured error");
    //         }
    //     });

    //     Ok(())
    // }
}
