use cfg_if::cfg_if;
use sycamore::prelude::*;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use sycamore::context::use_context;

        fn get_content<G: 'static>() -> String {
            let ctx = use_context::<crate::Context>();
            ctx.get_paste().map(|paste| paste.content().to_owned()).unwrap()
        }
    } else {
        fn get_content<G: 'static>() -> String {
            if let Some(hk) = sycamore::utils::hydrate::get_current_id() {
                web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .query_selector(&format!("[data-hk=\"{}.{}\"] > textarea", hk.0, hk.1))
                    .unwrap()
                    .unwrap()
                    .inner_html() // inner_text would be better, but inner_html is good enough
            } else {
                "dynamic not implemented".to_owned()
            }
        }
    }
}

#[component(PastePage<G>)]
pub fn paste_page(_content: String) -> View<G> {
    let content = get_content::<G>();

    view! {
        div {
            h1 { "Paste" }
            textarea(readonly=true) {
                (content)
            }
        }
    }
}
