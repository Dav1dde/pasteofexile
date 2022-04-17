use std::collections::HashMap;

use git_version::git_version;
use serde::Serialize;
use wasm_bindgen::JsValue;
use worker::{
    worker_sys::Request as EdgeRequest, Context, Env, Fetch, Headers, Method, RequestInit,
};

use crate::{consts, Error, Result};

#[derive(Serialize)]
struct Store<'a> {
    logger: &'a str,
    platform: &'a str,
    level: &'a str,
    // extra: Option<HashMap<&'a str, >,
    exception: &'a Exception<'a>,
    request: &'a Request<'a>,
    user: &'a User<'a>,
    server_name: &'a str,
    release: &'a str,
    transaction: &'a str,
}

#[derive(Serialize)]
struct Exception<'a> {
    values: &'a [ExceptionValue<'a>],
}

#[derive(Serialize)]
struct ExceptionValue<'a> {
    r#type: &'a str,
    value: &'a str,
    // stacktrace
}

#[derive(Serialize)]
struct Request<'a> {
    url: &'a str,
    method: &'a str,
    headers: HashMap<String, String>,
    data: Option<&'a str>,
}

#[derive(Serialize)]
struct User<'a> {
    ip_address: &'a str,
    country: &'a str,
}

pub struct Sentry {
    ctx: Context,
    req: EdgeRequest,
    token: String,
    store_url: String,
}

impl Sentry {
    pub fn from_env(env: &Env, ctx: Context, req: &EdgeRequest) -> Option<Self> {
        let project = env.var(consts::ENV_SENTRY_PROJECT).ok()?.to_string();
        let store_url = format!("https://sentry.io/api/{}/store/", project);
        Some(Self {
            ctx,
            req: Clone::clone(req),
            token: env.var(consts::ENV_SENTRY_TOKEN).ok()?.to_string(),
            store_url,
        })
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

    fn do_capture_err(&self, err: &Error, level: &'static str) -> Result<()> {
        let mut headers = Headers::new();
        headers.set("Content-Type", "application/json")?;
        headers.set("User-Agent", "pobb.bin/1.0")?;
        headers.set(
            "X-Sentry-Auth",
            &format!(
                "Sentry sentry_version=7, sentry_client=pobb.in/1.0, sentry_key={}",
                self.token,
            ),
        )?;

        let req: worker::Request = Clone::clone(&self.req).into();
        let url = req.url()?;

        let body = JsValue::from_str(&serde_json::to_string(&Store {
            logger: "worker",
            platform: "other",
            level,
            exception: &Exception {
                values: &[ExceptionValue {
                    r#type: err.name(),
                    value: &err.to_string(),
                }],
            },
            request: &Request {
                url: &req.inner().url(),
                method: &req.inner().method(),
                headers: req.headers().into_iter().collect(),
                data: err.payload(),
            },
            user: &User {
                ip_address: &req.headers().get("cf-connecting-ip")?.unwrap_or_default(),
                country: &req.headers().get("cf-ipcountry")?.unwrap_or_default(),
            },
            release: git_version!(),
            server_name: url.host_str().unwrap_or(""),
            // TODO: use route information here
            transaction: "",
        })?);

        let request = worker::Request::new_with_init(
            &self.store_url,
            &RequestInit {
                method: Method::Post,
                headers,
                body: Some(body),
                ..Default::default()
            },
        )?;

        self.ctx.wait_until(async move {
            let r = Fetch::Request(request).send().await;
            if let Err(err) = r {
                log::warn!("failed to caputre error with sentry: {:?}", err);
            } else {
                log::info!("successfully captured error");
            }
        });

        Ok(())
    }
}
