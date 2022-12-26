use shared::User;
use sycamore::component::Component;
use sycamore::prelude::*;
use sycamore_router::{HistoryIntegration, Router as DynRouter, RouterProps};
use web_sys::Element;

use crate::{
    future::LocalBoxFuture,
    pages, try_block,
    utils::{deserialize_from_attribute, is_hydrating, serialize_for_attribute},
    Context, Error, Meta, ResponseContext, Result,
};

#[derive(Clone, Debug, sycamore_router::Route)]
#[cfg_attr(feature = "ssr", derive(strum::IntoStaticStr))]
pub enum Route {
    #[to("/")]
    Index,
    #[to("/<id>")]
    Paste(String),
    #[to("/u/<name>")]
    User(User),
    #[to("/u/<name>/<id>")]
    UserPaste(User, String),
    #[to("/u/<name>/<id>/edit")]
    UserEditPaste(User, String),
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

    fn from_context(args: Self::RouteArg, ctx: Context) -> Result<<Self as Component<G>>::Props>;
    fn from_hydration(
        args: Self::RouteArg,
        element: Element,
    ) -> Result<<Self as Component<G>>::Props>;
    fn from_dynamic<'a>(
        args: Self::RouteArg,
    ) -> LocalBoxFuture<'a, Result<<Self as Component<G>>::Props>>;

    fn meta(arg: &<Self as Component<G>>::Props) -> Result<Meta>;
}

#[component(Router<G>)]
pub fn router(ctx: Option<Context>) -> View<G> {
    ctx.map(|ctx| {
        // Fix hydration. During SSR there is no router component, while there is one
        // at and after hydration. Artificially introduce a component without actually
        // using a component.
        // Can't use the `StaticRouter` because then we'd have to clone the context.
        sycamore::utils::hydrate::hydrate_component(|| switch(Switch::Static(ctx)))
    })
    .unwrap_or_else(|| {
        view! {
            DynRouter(RouterProps::new(HistoryIntegration::new(), switch_browser))
        }
    })
}

#[allow(clippy::large_enum_variant)]
enum Switch {
    Static(Context),
    Dynamic(ReadSignal<Route>),
}

fn switch_browser<G: Html>(route: ReadSignal<Route>) -> View<G> {
    switch(Switch::Dynamic(route))
}

fn switch<G: Html>(switch: Switch) -> View<G> {
    // TODO: loading view?
    let view = Signal::new(View::empty());

    let mut stored_route = "";
    let mut stored_state = None;

    match switch {
        Switch::Static(ctx) => {
            let page = Page::from_context(ctx);
            // During SSR store the page, so we can recover it during hydration
            if let Some((route, state)) = page.store() {
                stored_route = route;
                stored_state = state;
            }
            view.set(render(page));
        }
        Switch::Dynamic(route) => {
            if is_hydrating() {
                view.set(render(Page::from_hydration(&route.get())));
            }

            // Always set up the effect even if hydrating to make sure
            // the reactive scope is tracked correctly.
            crate::effect!(view, {
                let route = route.get();

                // Don't actually have to fetch data, it's already there.
                // Could also check if the view changed, but this might be trouble
                // if you actually want to refresh the site/route.
                if is_hydrating() {
                    return;
                }

                sycamore::futures::spawn_local_in_scope(cloned!(view => async move {
                    view.set(render(Page::from_dynamic(&route).await))
                }));
            });
        }
    }

    let stored_state = stored_state.unwrap_or_default();
    view! {
        div(data-route=stored_route, data-state=stored_state) {
            (view.get().as_ref().clone())
        }
    }
}

enum Page<G: Html> {
    Index,
    Paste(<pages::PastePage<G> as Component<G>>::Props),
    User(<pages::UserPage<G> as Component<G>>::Props),
    UserPaste(<pages::UserPastePage<G> as Component<G>>::Props),
    UserEditPaste(<pages::UserEditPastePage<G> as Component<G>>::Props),
    Error(u16, String),
}

impl<G: Html> Page<G> {
    fn from_context(ctx: Context) -> Self {
        let page = try_block! {
            Ok::<_, Error>(match ctx.route() {
                Ok(Route::Index) => Self::Index,
                Ok(Route::Paste(arg)) =>
                    Self::Paste(pages::PastePage::<G>::from_context(arg.clone(), ctx)?),
                Ok(Route::User(arg)) =>
                    Self::User(pages::UserPage::<G>::from_context(arg.clone(), ctx)?),
                Ok(Route::UserPaste(user, id)) =>
                    Self::UserPaste(pages::UserPastePage::<G>::from_context((user.clone(), id.clone()), ctx)?),
                Ok(Route::UserEditPaste(user, id)) =>
                    Self::UserEditPaste(pages::UserEditPastePage::<G>::from_context((user.clone(), id.clone()), ctx)?),
                Ok(Route::NotFound) => Self::not_found(),
                Err(err) => Self::resolve_err(err),
            })
        };

        let page = Self::resolve(page);

        if let Self::Error(status_code, _) = page {
            ResponseContext::set_status_code(status_code);
        }

        if let Ok(meta) = page.meta() {
            // TODO: should we log here if something goes wrong?
            ResponseContext::set_meta(meta);
        }

        page
    }

    fn from_hydration(route: &Route) -> Self {
        let hk = sycamore::utils::hydrate::get_current_id().unwrap();

        // Router component including the data-router attribute
        let element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .body()
            .unwrap()
            .query_selector(&format!("[data-hk=\"{}.{}\"]", hk.0, hk.1))
            .unwrap()
            .unwrap();

        // Recover a page that was stored during SSR.
        // This usually happens when we were supposed to render a page, but the page threw an error
        // and it was in place redirected to an error page.
        let stored_state = element.get_attribute("data-state");
        let stored_page = element
            .get_attribute("data-route")
            .as_deref()
            .and_then(|route| Self::restore(route, stored_state));

        if let Some(page) = stored_page {
            return page;
        }

        let page = try_block! {
            Ok::<_, Error>(match route {
                Route::Index => Self::Index,
                Route::Paste(arg) => Self::Paste(pages::PastePage::<G>::from_hydration(arg.clone(), element)?),
                Route::User(arg) => Self::User(pages::UserPage::<G>::from_hydration(arg.clone(), element)?),
                Route::UserPaste(user, id) => Self::UserPaste(
                    pages::UserPastePage::<G>::from_hydration((user.clone(), id.clone()), element)?
                ),
                Route::UserEditPaste(user, id) => Self::UserEditPaste(
                    pages::UserEditPastePage::<G>::from_hydration((user.clone(), id.clone()), element)?
                ),
                Route::NotFound => Self::not_found(),
            })
        };

        Self::resolve(page)
    }

    async fn from_dynamic(route: &Route) -> Self {
        use crate::try_block_async;

        let page = try_block_async! {
            Ok::<_, Error>(match route {
                Route::Index => Self::Index,
                Route::Paste(arg) => {
                    Self::Paste(pages::PastePage::<G>::from_dynamic(arg.clone()).await?)
                },
                Route::User(arg) => {
                    Self::User(pages::UserPage::<G>::from_dynamic(arg.clone()).await?)
                },
                Route::UserPaste(user, id) => {
                    Self::UserPaste(pages::UserPastePage::<G>::from_dynamic((user.clone(), id.clone())).await?)
                },
                Route::UserEditPaste(user, id) => {
                    Self::UserEditPaste(pages::UserEditPastePage::<G>::from_dynamic((user.clone(), id.clone())).await?)
                },
                Route::NotFound => Self::not_found(),
            })
        };

        let page = Self::resolve(page);

        if let Ok(meta) = page.meta() {
            // TODO: maybe update other metadata
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .head()
                .unwrap()
                .query_selector("title")
                .unwrap()
                .unwrap()
                .set_text_content(Some(&meta.title));
        }

        page
    }

    fn resolve(r: Result<Self>) -> Self {
        r.unwrap_or_else(|e| Self::resolve_err(&e))
    }

    fn resolve_err(err: &Error) -> Self {
        tracing::warn!("encountered error: {:?}", err);
        // TODO: error context on these errors,
        // e.g. not found page displaying the resource type
        match err {
            Error::NotFound(_, _) => Self::Error(404, "Not Found".to_owned()),
            // TODO: rethink this, if this happens because of a pastebin.com build this is fine and
            // a 400 status code, if this happens on an uploaded paste, this is a problem.
            Error::PobError(_) => Self::Error(400, "Invalid Build Code".to_owned()),
            _ => Self::Error(500, "Unknown Error".to_owned()),
        }
    }

    fn meta(&self) -> Result<Meta> {
        match self {
            Self::Index => Ok(Meta::index()),
            Self::Paste(ref props) => pages::PastePage::<G>::meta(props),
            Self::User(ref props) => pages::UserPage::<G>::meta(props),
            Self::UserPaste(ref props) => pages::UserPastePage::<G>::meta(props),
            Self::UserEditPaste(ref props) => pages::UserEditPastePage::<G>::meta(props),
            Self::Error(_, message) => Ok(Meta::error(message)),
        }
    }

    /// Used to serialize meta pages which can not be inferred from a route.
    ///
    /// Meta pages are pages like error pages which can happen under any route.
    /// These pages need to remember on hydration and need to be restored
    /// as such again.
    ///
    /// Returns a pair of meta identifier and its state.
    fn store(&self) -> Option<(&'static str, Option<String>)> {
        // Sync with `Self::restore`.
        let Self::Error(status_code, message) = self else { return None };

        let state = serialize_for_attribute(&(status_code, message));
        Some(("error", Some(state)))
    }

    /// Deserializes a meta page from its identifier and state.
    fn restore(previous: &str, state: Option<String>) -> Option<Self> {
        // Sync with `Self::store`.
        if previous != "error" {
            return None;
        }

        let (code, message) = deserialize_from_attribute(&state.expect("route state"));
        Some(Self::Error(code, message))
    }

    fn not_found() -> Self {
        Self::Error(404, "Not Found".to_owned())
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
        Page::User(props) => view! {
            pages::UserPage(props)
        },
        Page::UserPaste(props) => view! {
            pages::UserPastePage(props)
        },
        Page::UserEditPaste(props) => view! {
            pages::UserEditPastePage(props)
        },
        Page::Error(status_code, message) => view! {
            // This needs to be in a component to not interfere with hydration.
            // A new hydration level is introduced per component, this
            // makes sure elements defined here don't mess with hydration
            // levels of other branches.
            DisplayError((status_code, message))
        },
    }
}

#[component(DisplayError<G>)]
pub fn display_error((status_code, message): (u16, String)) -> View<G> {
    view! {
        span(class="pr-2") { (status_code) }
        span() { (message) }
    }
}
