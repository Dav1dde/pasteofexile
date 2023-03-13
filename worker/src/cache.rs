use shared::PasteId;

use crate::request_context::RequestContext;

pub(crate) fn on_paste_change(rctx: &RequestContext, id: PasteId) {
    let url = rctx.url().unwrap();
    rctx.ctx().wait_until(on_paste_change_async(url, id));
}

pub(crate) async fn on_paste_change_async(mut url: url::Url, id: PasteId) {
    url.set_path("");
    url.set_query(None);
    url.set_fragment(None);
    let prefix = url.to_string();

    let cache = worker::Cache::default();

    macro_rules! clear {
        ($e:expr) => {{
            let _ = cache
                .delete(format!("{prefix}{}", $e.trim_start_matches('/')), true)
                .await;
        }};
    }

    tracing::info!("resetting cached URLs for {id}");
    clear!(id.to_url());
    clear!(id.to_raw_url());
    clear!(id.to_json_url());
    clear!(id.to_pob_load_url());

    if let PasteId::UserPaste(up) = id {
        clear!(up.to_pob_long_load_url());
        clear!(up.to_paste_edit_url());
        clear!(up.to_user_url());
        clear!(up.to_user_api_url());
    }
    tracing::info!("done resetting caches");
}
