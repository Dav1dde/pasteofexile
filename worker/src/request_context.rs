use std::ops::Deref;

use wasm_bindgen::JsCast as _;
use worker::Bucket;

use crate::{cache::CacheEntry, route, utils::RequestExt};

pub struct RequestContext {
    req: worker::Request,
    env: Env,
    ctx: worker::Context,
    route: route::Route,
    trace_id: sentry::TraceId,
    session: Option<app::User>,
}

// TODO this could/should be a Session() type
pub type Session<'a> = Option<&'a app::User>;

impl RequestContext {
    pub async fn new(req: worker::Request, env: worker::Env, ctx: worker::Context) -> Self {
        let route = route::Route::new(&req);
        let env = Env::new(env);
        let session = parse_session(&req, &env).await;
        Self {
            req,
            env,
            ctx,
            route,
            trace_id: sentry::TraceId::default(),
            session,
        }
    }

    pub fn trace_id(&self) -> sentry::TraceId {
        self.trace_id
    }

    pub fn req(&self) -> &worker::Request {
        &self.req
    }

    pub fn req_mut(&mut self) -> &mut worker::Request {
        &mut self.req
    }

    pub fn ctx(&self) -> &worker::Context {
        &self.ctx
    }

    pub fn owned_ctx(&self) -> worker::Context {
        // `worker::Context` doesn't implement Clone, but there is no reason it shouldn't.
        let ctx = self.ctx.as_ref().unchecked_ref::<js_sys::Object>().clone();
        let ctx = ctx.unchecked_into();
        worker::Context::new(ctx)
    }

    pub fn route(&self) -> &route::Route {
        &self.route
    }

    /// Creates an instance of `T` and returns it.
    ///
    /// This does not cache instances, it creates a new instance on every access.
    /// There is also no magic injection, it simply creates a new instance.
    ///
    /// # Panics
    ///
    /// This will panic if `T` cannot be created because of missing env vars.
    /// Use [`inject_opt`] if injection is expected to fail.
    #[track_caller]
    pub fn inject<T: FromEnv>(&self) -> T {
        // It's fine to panic here, failing to inject means there is a bug in the code
        // or outside configuration.
        if let Some(t) = T::from_env(&self.env) {
            return t;
        }

        tracing::error!(
            "failed to inject instance for type {}",
            std::any::type_name::<T>()
        );
        panic!(
            "failed to inject instance for type {}",
            std::any::type_name::<T>()
        )
    }

    /// Creates an optional `T` and returns it.
    ///
    /// This behaves like [`inject`] but does not panic,
    /// if the environment is mssing variables.
    pub fn inject_opt<T: FromEnv>(&self) -> Option<T> {
        T::from_env(&self.env)
    }
}

impl RequestContext {
    pub fn transaction(&self) -> String {
        use route::{Api::*, Route::*};
        match self.route {
            Asset => "asset".to_owned(),
            App(ref app) => format!("app::{}", <&str>::from(app)),
            Api(ref api) => match api {
                Get(ref get) => format!("api::get::{}", <&str>::from(get)),
                Post(ref post) => format!("api::post::{}", <&str>::from(post)),
                Delete(ref delete) => format!("api::delete::{}", <&str>::from(delete)),
            },
            NotFound => "not_found".to_owned(),
        }
    }

    pub async fn get_sentry_user(&self) -> sentry::User {
        sentry::User {
            username: self.session().map(|user| user.name.clone().into()),
            ip_address: self.req.headers().get("cf-connecting-ip").ok().flatten(),
            country: self.req.cf().and_then(|cf| cf.country()),
        }
    }

    pub fn get_sentry_request(&self) -> sentry::Request {
        sentry::Request {
            url: self.req.url().ok(),
            method: Some(self.req.inner().method()),
            headers: self.req.headers().into_iter().collect(),
            ..Default::default()
        }
    }

    pub fn session(&self) -> Session<'_> {
        self.session.as_ref()
    }

    pub fn is_logged_in(&self) -> bool {
        self.session.is_some()
    }

    pub fn cache_entry(&self) -> CacheEntry<'_> {
        self.into()
    }
}

impl Deref for RequestContext {
    type Target = worker::Request;

    fn deref(&self) -> &Self::Target {
        &self.req
    }
}

pub struct Env {
    inner: worker::Env,
}

impl Env {
    fn new(inner: worker::Env) -> Self {
        Self { inner }
    }

    pub fn kv(&self, name: &str) -> Option<worker::kv::KvStore> {
        self.inner.kv(name).ok()
    }

    pub fn var(&self, name: &str) -> Option<String> {
        self.inner.var(name).ok().map(|v| v.to_string())
    }

    pub fn bucket(&self, name: &str) -> Option<Bucket> {
        self.inner.bucket(name).ok()
    }
}

pub trait FromEnv: Sized {
    fn from_env(env: &Env) -> Option<Self>;
}

async fn parse_session(req: &worker::Request, env: &Env) -> Option<app::User> {
    let session = req.session()?;

    let dangerous = crate::dangerous::Dangerous::from_env(env).expect("failed to create Dangerous");
    match dangerous
        .verify::<app::User>(&session, app::consts::MAX_SESSION_DURATION)
        .await
    {
        Ok(user) => Some(user),
        Err(err) => {
            tracing::warn!("failed to decode session: {err:?}");
            None
        }
    }
}
