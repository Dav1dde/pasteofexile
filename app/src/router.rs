use crate::pages;
use sycamore::prelude::*;
use sycamore_router::{
    HistoryIntegration, Router as DynRouter, RouterProps, StaticRouter, StaticRouterProps,
};

#[derive(Clone, Debug, PartialEq, Eq, sycamore_router::Route)]
pub enum Route {
    #[to("/")]
    Index,
    #[to("/<id>")]
    Paste(String),
    #[not_found]
    NotFound,
}

impl Route {
    pub fn resolve(path: &str) -> Self {
        use sycamore_router::Route;
        Self::match_path(path)
    }
}

#[component(Router<G>)]
pub fn router(route: Option<Route>) -> View<G> {
    route
        .map(|route| view! {
            StaticRouter(StaticRouterProps::new(route, |route: Route| switch(Signal::new(route).handle())))
        })
        .unwrap_or_else(|| view! {
            DynRouter(RouterProps::new(HistoryIntegration::new(), switch))
        })
}

fn switch<G: Html>(route: ReadSignal<Route>) -> View<G> {
    view! {
        div {
        (match route.get().as_ref() {
            Route::Index => view! {
                pages::IndexPage()
            },
            Route::Paste(name) => view! {
                pages::PastePage(name.to_owned())
            },
            Route::NotFound => view! {
                "404 Not Found"
            },
        })
    }
    }
}
