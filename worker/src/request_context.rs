use std::ops::Deref;

use crate::{
    route,
    sentry::{self, TraceId},
    utils::RequestExt,
};

pub struct RequestContext {
    req: worker::Request,
    env: Env,
    ctx: worker::Context,
    route: route::Route,
    trace_id: TraceId,
}

impl RequestContext {
    pub fn new(req: worker::Request, env: worker::Env, ctx: worker::Context) -> Self {
        let route = route::Route::new(&req);
        Self {
            req,
            env: Env::new(env),
            ctx,
            route,
            trace_id: TraceId::default(),
        }
    }

    pub fn trace_id(&self) -> TraceId {
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
            username: self
                .session()
                .await
                .ok()
                .flatten()
                .map(|user| user.name.into()),
            ip_address: self.req.headers().get("cf-connecting-ip").ok().flatten(),
            country: self.req.headers().get("cf-ipcountry").ok().flatten(),
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

    pub async fn session(&self) -> crate::Result<Option<app::User>> {
        let session = match self.req().session() {
            Some(session) => session,
            None => return Ok(None),
        };

        let dangerous = self.inject::<crate::dangerous::Dangerous>();
        Ok(Some(dangerous.verify::<app::User>(&session).await?))
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
}

pub trait FromEnv: Sized {
    fn from_env(env: &Env) -> Option<Self>;
}
