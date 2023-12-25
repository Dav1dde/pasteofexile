use sentry::WithSentry;
use statsd::Counters;
use worker::{event, Context, Env, Request, Response as WorkerResponse};

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
mod sentry_impl;
mod stats;
mod statsd;
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
    LOG_INIT.call_once(|| {
        use tracing_subscriber::prelude::*;
        tracing_subscriber::registry()
            .with(sentry::Layer {})
            .with(layer::Layer {})
            .init();
    });

    let mut rctx = RequestContext::new(req, env, ctx).await;

    let mut sentry = sentry::new(
        sentry_impl::Transport(rctx.ctx().clone()),
        rctx.inject_opt(),
    );

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
    if response.is_skip_sentry() {
        sentry.ignore();
    }

    stats::record(&rctx, &response).await;

    Ok(worker::Response::from(response))
}

#[tracing::instrument(skip_all)]
async fn cached(rctx: &mut RequestContext) -> Response {
    sentry::counter(Counters::Request).inc(1);

    let cache_entry = rctx.cache_entry();

    if let Some(response) = cache_entry.load().await {
        tracing::debug!("cache hit");
        sentry::counter(Counters::CacheHit).inc(1);
        return response;
    }

    let response = handle(rctx).await;

    cache_entry.store(response).await
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
        sentry::capture_err(err.inner(), err.inner().level());
    }

    match response {
        Ok(response) => response,
        Err(response::ApiError(err)) => {
            sentry::counter(Counters::RequestError)
                .inc(1)
                .tag("type", "api");
            err.into()
        }
        Err(response::AppError(err)) => {
            sentry::counter(Counters::RequestError)
                .inc(1)
                .tag("type", "app");
            app::handle_err(err).await
        }
    }
}
