use crate::future::LocalBoxFuture;
use crate::router::RoutedComponent;
use pob::{PathOfBuilding, SerdePathOfBuilding};
use std::rc::Rc;
use sycamore::prelude::*;

pub struct Data {
    content: String,
    pob: Rc<SerdePathOfBuilding>,
}

impl<G: Html> RoutedComponent<G> for PastePage<G> {
    type RouteArg = String;

    fn from_context(ctx: crate::Context) -> anyhow::Result<Data> {
        let paste = ctx.get_paste().unwrap();
        Ok(Data {
            content: paste.content().to_owned(),
            pob: paste.path_of_building()?,
        })
    }

    fn from_hydration(element: web_sys::Element) -> anyhow::Result<Data> {
        let content = element
            .query_selector("textarea")
            .unwrap()
            .unwrap()
            .inner_html();

        let pob = Rc::new(SerdePathOfBuilding::from_export(&content)?);
        Ok(Data { content, pob })
    }

    fn from_dynamic<'a>(id: Self::RouteArg) -> LocalBoxFuture<'a, anyhow::Result<Data>> {
        Box::pin(async move {
            let content = crate::api::get_paste(id).await?;
            let pob = Rc::new(SerdePathOfBuilding::from_export(&content)?);
            Ok(Data { content, pob })
        })
    }
}

#[component(PastePage<G>)]
pub fn paste_page(Data { content, pob }: Data) -> View<G> {
    let title = crate::pob::title(&*pob);

    view! {
        div {
            h1 { (title) }
            textarea(readonly=true) {
                (content)
            }
        }
    }
}
