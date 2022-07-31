use shared::model::PasteId;

use crate::request_context::RequestContext;

pub(crate) fn on_paste_change(rctx: &RequestContext, id: PasteId) {
    let user_paste = match id {
        PasteId::UserPaste(up) => up,
        _ => return,
    };

    let mut prefix = rctx.req().url().unwrap();
    prefix.set_path("");
    prefix.set_query(None);
    prefix.set_fragment(None);
    let prefix = prefix.to_string();

    rctx.ctx().wait_until(async move {
        let cache = worker::Cache::default();

        macro_rules! clear {
            ($e:expr) => {{
                let _ = cache
                    .delete(format!("{prefix}{}", $e.trim_start_matches('/')), true)
                    .await;
            }};
        }

        log::info!("resetting cached URLs for {user_paste}");
        clear!(user_paste.to_paste_url());
        clear!(user_paste.to_json_url());
        clear!(user_paste.to_raw_url());
        clear!(user_paste.to_pob_load_url());
        clear!(user_paste.to_pob_long_load_url());
        clear!(user_paste.to_paste_edit_url());
        clear!(user_paste.to_user_url());
        log::info!("done resetting caches");
    });
}
