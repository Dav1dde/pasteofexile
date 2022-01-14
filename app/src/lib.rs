use sycamore::context::{ContextProvider, ContextProviderProps};
use sycamore::prelude::*;

mod components;
mod context;
mod pages;
mod router;

pub use context::Context;
pub use router::Route;

use components::ThemeToggle;

#[cfg(feature = "ssr")]
pub fn render_to_string(context: Context) -> String {
    sycamore::render_to_string(|| view! { App(Some(context)) })
}

#[component(App<G>)]
pub fn app(ctx: Option<Context>) -> View<G> {
    let ctx = ctx.unwrap_or_else(Context::empty);
    let route = ctx.route().cloned();

    view! {
        ContextProvider(ContextProviderProps {
            value: ctx,
            children: || view! {
                div {
                    nav(class="flex py-4 border-b border-slate-900/10 lg:px-8 dark:border-slate-300/10 mx-4 lg:mx-0 mb-10") {
                        a(class="flex-auto", href="/") { "Paste of Exile" }
                        ThemeToggle()
                    }
                    div(class="max-w-screen-xl mx-auto px-5 xl:px-0") {
                        router::Router(route)
                    }
                }
            }
        })
    }
}
