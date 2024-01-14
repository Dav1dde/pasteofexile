use session::provide_session;
use storage::provide_storage;
use sycamore::prelude::*;

mod api;
mod assets;
mod build;
mod components;
mod consts;
mod context;
mod error;
mod future;
mod meta;
mod pages;
pub mod pob;
mod progress;
mod response_context;
mod router;
mod session;
mod storage;
mod svg;
mod tree;
mod utils;

#[cfg(feature = "ssr")]
mod head;

pub use context::Context;
pub use error::{Error, Result};
pub use meta::{Meta, Prefetch};
pub use response_context::ResponseContext;
pub use router::Route;
pub use session::User;
pub use utils::PercentRoute;

#[cfg(feature = "ssr")]
pub fn render_to_string(context: Context) -> (String, ResponseContext) {
    ResponseContext::with(|| sycamore::render_to_string(|cx| view! { cx, App(Some(context)) }))
}

#[cfg(feature = "ssr")]
pub type Head = head::HeadArgs;

#[cfg(feature = "ssr")]
pub fn render_head(args: Head) -> String {
    let mut result = sycamore::render_to_string(|cx| view! { cx, head::Head(args) });

    // workaround to replace data-hk with data-xx to not interfer with hydration
    let bytes = unsafe { result.as_bytes_mut() };
    static DATA_HK: &[u8] = b"data-hk";
    for i in 0..(bytes.len() - DATA_HK.len()) {
        if &bytes[i..i + DATA_HK.len()] == DATA_HK {
            bytes[i + 5] = b'x';
            bytes[i + 6] = b'x';
        }
    }

    result
}

#[component]
pub fn App<G: Html>(cx: Scope, ctx: Option<Context>) -> View<G> {
    provide_session::<G>(cx);
    provide_storage::<G>(cx);

    let view: View<G> = view! { cx,
        progress::Progress()
        div(class="min-h-screen flex flex-col gap-10") {
            nav(class="bg-slate-900 shadow-lg") {
                div(class="flex justify-between	p-4 lg:px-8 mx-auto max-w-[1920px]") {
                    a(href="/") {
                        span() { "POB" }
                        span(class="text-sky-400") { "b.in" }
                    }
                    div(class="flex items-center gap-3") {
                        components::LoginStatus()
                        div(class="bg-slate-300 w-px h-3/5") {}
                        components::PasteHistory()
                    }
                }
            }
            main(class="max-w-screen-xl px-5 xl:px-0 w-full flex-auto self-center") {
                router::Router(ctx)
            }
            footer(class="bg-slate-900 text-slate-400 text-xs self-center w-full
                    flex flex-wrap justify-between items-center gap-2
                    shadow-lg shadow-slate-100/50 py-2 px-4 lg:px-8") {
                div() { "pobb.in isn't affiliated with or endorsed by Grinding Gear Games in any way" }
                a(href="https://github.com/Dav1dde/pasteofexile", target="_blank",
                    class="w-4 h-4", aria-label="Source code on GitHub",
                    dangerously_set_inner_html=svg::GITHUB) {}
            }
        }
    };

    if G::IS_BROWSER {
        // We need to hack the router together here,
        // since a bunch of stuff is not wrapped in the router, so the clicks don't get captured.
        use sycamore_router::Integration;
        let integration = sycamore_router::HistoryIntegration::new();
        let view = view.clone();
        create_effect_scoped(cx, move |cx| {
            for node in view.clone().flatten() {
                node.event(cx, "click", integration.click_handler());
            }
        });
    }

    view
}
