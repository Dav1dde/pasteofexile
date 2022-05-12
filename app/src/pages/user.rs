use crate::{future::LocalBoxFuture, router::RoutedComponent, Meta, Result};
use sycamore::prelude::*;

pub struct Data {
    name: String,
}

impl<G: Html> RoutedComponent<G> for UserPage<G> {
    type RouteArg = String;

    fn from_context(name: Self::RouteArg, _ctx: crate::Context) -> Result<Data> {
        Ok(Data { name })
    }

    fn from_hydration(name: Self::RouteArg, _element: web_sys::Element) -> Result<Data> {
        Ok(Data { name })
    }

    fn from_dynamic<'a>(name: Self::RouteArg) -> LocalBoxFuture<'a, Result<Data>> {
        Box::pin(async move { Ok(Data { name }) })
    }

    fn meta(Data { name }: &Data) -> Result<Meta> {
        let title = format!("Test {name}").into();
        let description = "description".into();
        let image = "".into();
        let color = "";

        Ok(Meta {
            title,
            description,
            image,
            color,
        })
    }
}

#[component(UserPage<G>)]
pub fn user_page(Data { name }: Data) -> View<G> {
    view! {
        div() {
            (name)
        }
    }
}
