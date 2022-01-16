use crate::{effect, future::LocalBoxFuture, pages, utils::is_hydrating, Context};
use anyhow::Result;
use sycamore::component::Component;
use sycamore::prelude::*;
use sycamore::DomNode;
use sycamore_router::{
    HistoryIntegration, Router as DynRouter, RouterProps, StaticRouter, StaticRouterProps,
};
use web_sys::Element;

#[derive(Clone, Debug, PartialEq, Eq, sycamore_router::Route)]
pub enum Route {
    #[to("/")]
    Index,
    #[to("/<id>")]
    Paste(<pages::paste::PastePage<DomNode> as RoutedComponent<DomNode>>::RouteArg),
    #[not_found]
    NotFound,
}

impl Route {
    pub fn resolve(path: &str) -> Self {
        use sycamore_router::Route;
        Self::match_path(path)
    }
}

pub trait RoutedComponent<G: Html>: Component<G> {
    type RouteArg: Clone;

    fn from_context(ctx: Context) -> anyhow::Result<<Self as Component<G>>::Props>;
    fn from_hydration(element: Element) -> anyhow::Result<<Self as Component<G>>::Props>;
    fn from_dynamic<'a>(
        args: Self::RouteArg,
    ) -> LocalBoxFuture<'a, anyhow::Result<<Self as Component<G>>::Props>>;
}

#[component(Router<G>)]
pub fn router(ctx: Option<Context>) -> View<G> {
    let route = ctx.as_ref().and_then(|ctx| ctx.route().cloned());

    route
        .map(|route| {
            view! {
                StaticRouter(StaticRouterProps::new(
                    route, move |route: Route| switch(Signal::new(route).handle(), ctx.clone())
                ))
            }
        })
        .unwrap_or_else(|| {
            view! {
                DynRouter(RouterProps::new(HistoryIntegration::new(), switch_none))
            }
        })
}

fn switch_none<G: Html>(route: ReadSignal<Route>) -> View<G> {
    switch(route, None)
}

fn switch<G: Html>(route: ReadSignal<Route>, ctx: Option<Context>) -> View<G> {
    // TODO: loading view?
    let view = Signal::new(View::empty());

    effect!(view, {
        let route = route.get();
        let ctx = ctx.clone();

        // TODO: error handling, error pages, let errors show 404 site (e.g. paste does not exist)
        if let Some(ctx) = ctx {
            view.set(render(Page::from_context(ctx).unwrap()));
        } else if is_hydrating() {
            view.set(render(Page::from_hydration(&route).unwrap()));
        } else {
            #[cfg(not(feature = "ssr"))]
            sycamore::futures::spawn_local_in_scope(cloned!(view => async move {
                let page = Page::from_dynamic(&route).await.unwrap();
                view.set(render(page));
            }));
        }
    });

    view! { div { (view.get().as_ref().clone()) } }
}

enum Page<G: Html> {
    Index,
    Paste(<pages::PastePage<G> as Component<G>>::Props),
    NotFound,
}

impl<G: Html> Page<G> {
    fn from_context(ctx: Context) -> Result<Self> {
        Ok(match ctx.route().unwrap() {
            Route::Index => Self::Index,
            Route::Paste(_) => Self::Paste(pages::PastePage::<G>::from_context(ctx).unwrap()),
            Route::NotFound => Self::NotFound,
        })
    }

    fn from_hydration(route: &Route) -> Result<Self> {
        let hk = sycamore::utils::hydrate::get_current_id().unwrap();
        let element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .body()
            .unwrap()
            // (hk.0 + 1, hk.1) to select the next component -> the actual child/page
            .query_selector(&format!("[data-hk=\"{}.{}\"]", hk.0 + 1, hk.1))
            .unwrap()
            .unwrap();

        Ok(match route {
            Route::Index => Self::Index,
            Route::Paste(_) => Self::Paste(pages::PastePage::<G>::from_hydration(element)?),
            Route::NotFound => Self::NotFound,
        })
    }

    #[cfg(not(feature = "ssr"))]
    async fn from_dynamic(route: &Route) -> Result<Self> {
        // TODO: do we need this arg.clone()?
        Ok(match route {
            Route::Index => Self::Index,
            Route::Paste(arg) => {
                Self::Paste(pages::PastePage::<G>::from_dynamic(arg.clone()).await?)
            }
            Route::NotFound => Self::NotFound,
        })
    }
}

fn render<G: Html>(page: Page<G>) -> View<G> {
    match page {
        Page::Index => view! {
            pages::IndexPage()
        },
        Page::Paste(props) => view! {
            pages::PastePage(props)
        },
        Page::NotFound => view! {
            "404 Not Found"
        },
    }
}
