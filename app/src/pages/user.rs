use crate::{future::LocalBoxFuture, model::PasteSummary, router::RoutedComponent, Meta, Result};
use sycamore::prelude::*;

pub struct Data {
    name: String,
    pastes: Vec<PasteSummary>,
}

impl<G: Html> RoutedComponent<G> for UserPage<G> {
    type RouteArg = String;

    fn from_context(name: Self::RouteArg, ctx: crate::Context) -> Result<Data> {
        Ok(Data {
            name,
            pastes: ctx.get_user().unwrap().to_vec(),
        })
    }

    fn from_hydration(name: Self::RouteArg, _element: web_sys::Element) -> Result<Data> {
        Ok(Data {
            name,
            pastes: Vec::new(),
        })
    }

    fn from_dynamic<'a>(name: Self::RouteArg) -> LocalBoxFuture<'a, Result<Data>> {
        Box::pin(async move {
            Ok(Data {
                name,
                pastes: Vec::new(),
            })
        })
    }

    fn meta(Data { name, .. }: &Data) -> Result<Meta> {
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
pub fn user_page(Data { pastes, .. }: Data) -> View<G> {
    let p = pastes
        .into_iter()
        .map(|summary| {
            let url = summary.to_url();
            view! {
                div() {
                    a(href=url) { (summary.title) }
                }
            }
        })
        .collect();
    let p = View::new_fragment(p);

    view! {
        (p)
    }
}
