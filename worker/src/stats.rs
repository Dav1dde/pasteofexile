use git_version::git_version;
use serde::Serialize;

use crate::{
    consts, net,
    request_context::{Env, FromEnv, RequestContext},
    Response,
};

pub async fn record(rctx: &RequestContext, response: &Response) {
    let Some(meta) = response.get_meta() else { return; };
    let Some(stats) = rctx.inject_opt::<Stats>() else { return };

    let user = rctx.session().map(|u| u.name.clone());

    let headers = rctx.headers();

    let body = serde_json::json!({
        "transaction": rctx.transaction(),
        "trace_id": rctx.trace_id(),
        "app_version": git_version!(),
        "url": rctx.req().url().ok(),
        "method": rctx.method().to_string(),
        "status_code": response.status_code(),
        "content_length": headers.get("Content-Length").ok().flatten(),
        "user_agent": headers.get("User-Agent").ok().flatten(),
        "referrer": headers.get("Referer").ok().flatten(),
        "client_ip": headers.get("Cf-Connecting-Ip").ok().flatten(),
        "client_country": headers.get("Cf-IPCountry").ok().flatten(),
        "cached": response.was_cached(),
        "session": user,

        "user_id": meta.user_id,
        "paste_id": meta.paste_id,
        "ascendancy_or_class": meta.ascendancy_or_class,
        "main_skill_name": meta.main_skill_name,
        "version": meta.version,
        "last_modified": meta.last_modified,
    });

    rctx.ctx().wait_until(async move {
        let response = stats.send(&body).await;

        match response {
            Err(err) => worker::console_log!("failed to record stats: {err:?}"),
            Ok(response) => {
                if response.status_code() != 200 {
                    worker::console_log!("failed to record stats {}", response.status_code());
                }
            }
        }
    });
}

struct Stats {
    url: String,
    token: Option<String>,
}

impl FromEnv for Stats {
    fn from_env(env: &Env) -> Option<Self> {
        let url = env
            .var(consts::ENV_STATS_URL)
            .filter(|s| !s.trim().is_empty())?;
        let token = env.var(consts::ENV_STATS_TOKEN);
        Some(Self { url, token })
    }
}

impl Stats {
    async fn send(&self, body: &impl Serialize) -> worker::Result<worker::Response> {
        let body = serde_json::to_string(body)?;

        let token = self.token.as_ref().map(|token| format!("Bearer {token}"));
        net::Request::post(&self.url)
            .header("Content-Type", "application/json")
            .header_opt("Authorization", token.as_deref())
            .body(body)
            .no_sentry()
            .send()
            .await
    }
}
