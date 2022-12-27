use git_version::git_version;

use crate::{consts, net, request_context::RequestContext, Response};

pub async fn record(rctx: &RequestContext, response: &Response) {
    if !response.is_2xx() {
        return;
    }
    let Some(meta) = response.get_meta() else { return; };
    let Some((stats_url, stats_token)) = stats_data(rctx) else { return; };

    let user = rctx.session().await.ok().flatten().map(|u| u.name);

    let headers = rctx.headers();

    let body = serde_json::json!({
        "transaction": rctx.transaction(),
        "trace_id": crate::sentry::current_trace_id(),
        "app_version": git_version!(),
        "url": rctx.req().url().ok(),
        "method": rctx.method().to_string(),
        "status_code": response.status_code(),
        "user_agent": headers.get("User-Agent").ok().flatten(),
        "referer": headers.get("Referer").ok().flatten(),
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
        let token = stats_token.map(|token| format!("Bearer {token}"));
        let response = net::Request::post(stats_url)
            .header("Content-Type", "application/json")
            .header_opt("Authorization", token.as_deref())
            .body(serde_json::to_string(&body).expect("serialize stats"))
            .no_sentry()
            .send()
            .await;

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

fn stats_data(rctx: &RequestContext) -> Option<(String, Option<String>)> {
    let stats_url = rctx
        .env()
        .var(consts::ENV_STATS_URL)
        .ok()
        .map(|s| s.to_string())
        .filter(|s| !s.trim().is_empty())?;

    let stats_token = rctx
        .env()
        .var(consts::ENV_STATS_TOKEN)
        .ok()
        .map(|s| s.to_string());

    Some((stats_url, stats_token))
}
