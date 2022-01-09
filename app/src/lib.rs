use sycamore::prelude::*;
use sycamore_router::{
    HistoryIntegration, Route, Router, RouterProps, StaticRouter, StaticRouterProps,
};

mod components;
mod pages;

#[cfg(feature = "ssr")]
pub fn render_to_string(path: impl Into<String>) -> String {
    sycamore::render_to_string(|| view! { App(Some(path.into())) })
}

#[derive(Route)]
enum AppRoutes {
    #[to("/")]
    Index,
    #[to("/<id>")]
    Paste(String),
    #[not_found]
    NotFound,
}

#[component(App<G>)]
pub fn app(pathname: Option<String>) -> View<G> {
    match pathname {
        Some(pathname) => {
            let route = AppRoutes::match_path(&pathname);
            view! {
                StaticRouter(StaticRouterProps::new(route, |route: AppRoutes| switch(Signal::new(route).handle())))
            }
        }
        None => view! {
            Router(RouterProps::new(HistoryIntegration::new(), switch))
        },
    }
}

fn switch<G: Html>(route: ReadSignal<AppRoutes>) -> View<G> {
    view! {
        div {
            nav { "Navigation" }
            main {
                (match route.get().as_ref() {
                    AppRoutes::Index => view! {
                        pages::IndexPage()
                    },
                    AppRoutes::Paste(name) => view! {
                        pages::PastePage(name.to_owned())
                    },
                    AppRoutes::NotFound => view! {
                        "404 Not Found"
                    },
                })
            }
        }
    }
}
