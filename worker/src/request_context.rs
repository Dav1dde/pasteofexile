use std::ops::Deref;
use worker::kv;

use crate::{consts, route, sentry, utils::RequestExt};

pub struct RequestContext {
    req: worker::Request,
    env: worker::Env,
    ctx: worker::Context,
    route: route::Route,
}

impl RequestContext {
    pub fn new(req: worker::Request, env: worker::Env, ctx: worker::Context) -> Self {
        let route = route::Route::new(&req);
        Self {
            req,
            env,
            ctx,
            route,
        }
    }

    pub fn req(&self) -> &worker::Request {
        &self.req
    }

    pub fn req_mut(&mut self) -> &mut worker::Request {
        &mut self.req
    }

    #[allow(dead_code)]
    pub fn env(&self) -> &worker::Env {
        &self.env
    }

    pub fn ctx(&self) -> &worker::Context {
        &self.ctx
    }

    pub fn route(&self) -> &route::Route {
        &self.route
    }

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

    pub fn storage(&self) -> crate::Result<crate::storage::DefaultStorage> {
        crate::storage::DefaultStorage::from_env(&self.env)
    }

    pub fn oauth(&self) -> crate::Result<crate::poe_api::Oauth> {
        Ok(crate::poe_api::Oauth::new(
            self.env
                .var(crate::consts::ENV_OAUTH_CLIENT_ID)?
                .to_string(),
            self.env
                .var(crate::consts::ENV_OAUTH_CLIENT_SECRET)?
                .to_string(),
        ))
    }

    pub fn dangerous(&self) -> crate::Result<crate::dangerous::Dangerous> {
        let secret = self.env.var(crate::consts::ENV_SECRET_KEY)?.to_string();
        Ok(crate::dangerous::Dangerous::new(secret.into_bytes()))
    }

    pub fn get_sentry_options(&self) -> Option<sentry::Options> {
        let project = self.env.var(consts::ENV_SENTRY_PROJECT).ok()?.to_string();
        let token = self.env.var(consts::ENV_SENTRY_TOKEN).ok()?.to_string();

        Some(sentry::Options { project, token })
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

    pub async fn get_sentry_request(&self) -> sentry::Request {
        sentry::Request {
            url: self.req.url().ok(),
            method: Some(self.req.inner().method()),
            headers: self.req.headers().into_iter().collect(),
            ..Default::default()
        }
    }

    pub fn get_asset(&self, name: &str) -> crate::Result<kv::GetOptionsBuilder> {
        let kv = self.get_assets()?;
        let name = crate::assets::resolve(name);
        Ok(kv.get(&name))
    }

    pub fn get_assets(&self) -> crate::Result<kv::KvStore> {
        Ok(self.env.kv(consts::KV_STATIC_CONTENT)?)
    }

    pub async fn session(&self) -> crate::Result<Option<app::User>> {
        let session = match self.req().session() {
            Some(session) => session,
            None => return Ok(None),
        };

        Ok(Some(self.dangerous()?.verify::<app::User>(&session).await?))
    }
}

impl Deref for RequestContext {
    type Target = worker::Request;

    fn deref(&self) -> &Self::Target {
        &self.req
    }
}
