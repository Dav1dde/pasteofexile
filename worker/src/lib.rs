use sentry::WithSentry;
use worker::{event, Cache, Context, Env, Method, Request, Response as WorkerResponse};

mod api;
mod app;
mod assets;
mod cache;
mod consts;
mod crypto;
mod dangerous;
mod error;
mod layer;
mod net;
mod pastes;
mod poe_api;
mod request_context;
mod response;
mod retry;
mod route;
mod sentry;
mod stats;
mod storage;
mod utils;

mod app_metadata {
    include!(concat!(env!("OUT_DIR"), "/app_metadata.rs"));
}

use request_context::RequestContext;
use utils::CacheControl;

pub use self::error::{Error, ErrorResponse, Result};
pub use self::response::Response;

static LOG_INIT: std::sync::Once = std::sync::Once::new();

#[event(fetch)]
pub async fn main(req: Request, env: Env, ctx: Context) -> worker::Result<WorkerResponse> {
    let mut rctx = RequestContext::new(req, env, ctx);

    LOG_INIT.call_once(|| {
        use tracing_subscriber::prelude::*;
        tracing_subscriber::registry()
            .with(sentry::Layer {})
            .with(layer::Layer {})
            .init();
    });

    let sentry = sentry::new(rctx.ctx(), rctx.inject_opt());

    sentry
        .set_trace_id(rctx.trace_id())
        .set_user(rctx.get_sentry_user().await)
        .set_request(rctx.get_sentry_request())
        .start_transaction(sentry::TransactionContext {
            op: "http.server".to_owned(),
            name: rctx.transaction(),
        });

    let response = cached(&mut rctx).with_sentry(&sentry).await;

    sentry.update_transaction(sentry::Status::from(response.status_code()));

    stats::record(&rctx, &response).await;

    Ok(worker::Response::from(response))
}

#[tracing::instrument(skip_all)]
async fn cached(rctx: &mut RequestContext) -> Response {
    let cache = Cache::default();
    let use_cache = rctx.method() == Method::Get;

    if use_cache {
        if let Some(response) = cache.get(rctx.req(), true).await.expect("cache api") {
            tracing::debug!("cache hit");
            return Response::from_cache(response);
        }
    }

    let mut response = handle(rctx).await;

    if use_cache && response.is_cacheable() {
        let for_cache = response.for_cache();

        // Unwrap can only fail if the body was already consumed,
        // this is a GET request without a body at this point.
        let key = rctx.req().inner().clone().unwrap();
        rctx.ctx().wait_until(async move {
            tracing::debug!("--> caching response");
            let r = cache.put(&key, for_cache).await;
            debug_assert!(r.is_ok(), "failed to cache response: {r:?}");
            tracing::debug!("<-- response cached");
        });

        response.header("Cf-Cache-Status", "MISS")
    } else {
        response
    }
}

#[tracing::instrument(skip_all)]
async fn handle(rctx: &mut RequestContext) -> Response {
    let response = match rctx.route() {
        route::Route::Api(route) => api::handle(rctx, route.clone()).await,
        route::Route::App(route) => app::handle(rctx, route.clone()).await,
        route::Route::Asset => assets::handle(rctx).await,
        route::Route::NotFound => app::handle(rctx, ::app::Route::NotFound).await,
    };

    if let Err(ref err) = response {
        tracing::warn!("error: {err:?}");
        sentry::capture_err(err.inner());
    }

    match response {
        Ok(response) => response,
        Err(response::ApiError(err)) => err.into(),
        Err(response::AppError(err)) => app::handle_err(err).await,
    }
}
