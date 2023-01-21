use sycamore::prelude::*;

use crate::{
    components::{CreatePaste, CreatePasteProps, ImportPastebin},
    future::LocalBoxFuture,
    router::RoutedComponent,
    Meta, Result,
};

pub struct IndexPage;

impl RoutedComponent for IndexPage {
    type RouteArg = ();

    fn from_context(_args: Self::RouteArg, _ctx: crate::Context) -> Result<Self> {
        Ok(Self)
    }

    fn from_hydration(_args: Self::RouteArg, _element: web_sys::Element) -> Result<Self> {
        Ok(Self)
    }

    fn from_dynamic<'a>(_args: Self::RouteArg) -> LocalBoxFuture<'a, Result<Self>> {
        Box::pin(async { Ok(Self) })
    }

    fn meta(&self) -> Result<crate::Meta> {
        Ok(Meta::index())
    }

    fn render<G: Html>(self, cx: Scope) -> View<G> {
        view! { cx,
            div(class="flex flex-col gap-12") {
                CreatePaste(CreatePasteProps::default())
                ImportPastebin()
            }
        }
    }
}
