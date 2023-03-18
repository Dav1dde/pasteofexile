use session::use_session;
use sycamore::{prelude::*, web::NoSsr};

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
mod svg;
mod utils;

#[cfg(feature = "ssr")]
mod head;

pub use context::Context;
pub use error::{Error, Result};
pub use meta::{Meta, Prefetch};
pub use response_context::ResponseContext;
pub use router::Route;
pub use session::User;
use wasm_bindgen::{JsCast, UnwrapThrowExt};

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
    // we need to manually handle clicking here, since the nav isn't wrapped in a router
    let navigate_index = |ev: web_sys::Event| {
        sycamore_router::navigate("/");
        ev.prevent_default();
    };

    use_session::<G>(cx);

    let ad = ["/assets/yourad.webp", "/assets/chaoiscoinscam.webp"]
        [(js_sys::Date::new_0().get_time() % 2.0) as usize];

    let s = |ev: web_sys::Event| {
        let ev = ev.unchecked_into::<web_sys::KeyboardEvent>();

        if ev.key_code() == 13 {
            let _ = web_sys::window().unwrap_throw().alert_with_message(
                "I didn't implement this ... Give me a break, this is just an April Fools Joke.",
            );
        }
    };

    view! { cx,
        progress::Progress()
        div(class="bg-[#f1f3f4] sticky top-0 h-7 border-white border-b-2 text-[#af6025] flex items-center px-3") {
            span() {
                "Path Of Exile Bar"
            }
            span(class="ml-2 pl-2 border-l-2 border-slate-200") {
                "Wiki"
            }
            input(class="ml-1 bg-white h-[20px] text-black", placeholder="Search", on:keyup=s) {}
            span(class="ml-2 pl-2 border-l-2 border-slate-200") {
                a(class="text-blue-700 underline", href="https://pathofexile.com/trade", target="_blank") {
                    "Trade Site"
                }
            }
            span(class="ml-2 pl-2 border-l-2 border-slate-200") {
                a(class="text-blue-700 underline", href="https://pathofbuilding.community/", target="_blank") {
                    "Path of Building"
                }
            }
        }
        div(class="h-screen flex flex-col gap-10") {
            nav(class="bg-slate-200 dark:bg-slate-900 dark:shadow-lg") {
                div(class="flex justify-between	p-4 lg:px-8 mx-auto max-w-[1920px]") {
                    a(href="/", on:click=navigate_index) {
                        span() { "POB" }
                        span(class="text-sky-500 dark:text-sky-400") { "b.in" }
                    }
                    components::LoginStatus()
                }
            }
            div(class="flex justify-center") {
                NoSsr {
                    img(class="w-[500px]", src=ad) {}
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
    }
}
