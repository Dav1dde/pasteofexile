use sycamore::context::{ContextProvider, ContextProviderProps};
use sycamore::prelude::*;

mod components;
mod context;
mod pages;
mod router;

pub use context::Context;
pub use router::Route;

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
                    nav {
                        "Navigation"
                    }
                    router::Router(route)
                }
            }
        })
    }
}
