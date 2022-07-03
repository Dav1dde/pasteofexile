use crate::{
    components::{ViewPaste, ViewPasteProps},
    future::LocalBoxFuture,
    meta,
    model::PasteId,
    pob,
    router::RoutedComponent,
    Meta, Result,
};
use ::pob::{PathOfBuildingExt, SerdePathOfBuilding};
use std::rc::Rc;
use sycamore::prelude::*;

pub struct Data {
    id: String,
    content: String,
    pob: Rc<SerdePathOfBuilding>,
}

impl<G: Html> RoutedComponent<G> for PastePage<G> {
    type RouteArg = String;

    fn from_context(id: Self::RouteArg, ctx: crate::Context) -> Result<Data> {
        let paste = ctx.get_paste().unwrap();
        Ok(Data {
            id,
            content: paste.content().to_owned(),
            pob: paste.path_of_building()?,
        })
    }

    fn from_hydration(id: Self::RouteArg, element: web_sys::Element) -> Result<Data> {
        let content = element
            .query_selector("textarea")
            .unwrap()
            .unwrap()
            .inner_html();

        let pob = Rc::new(SerdePathOfBuilding::from_export(&content)?);
        Ok(Data { id, content, pob })
    }

    fn from_dynamic<'a>(id: Self::RouteArg) -> LocalBoxFuture<'a, Result<Data>> {
        Box::pin(async move {
            // TODO: get rid of this clone, there needs to be a better way to pass this around
            let content = crate::api::get_paste(&crate::model::PasteId::id(id.clone())).await?;
            let pob = Rc::new(SerdePathOfBuilding::from_export(&content)?);
            Ok(Data { id, content, pob })
        })
    }

    fn meta(arg: &Data) -> Result<Meta> {
        let pob: &SerdePathOfBuilding = &*arg.pob;

        let config = pob::TitleConfig { no_title: true };
        let mut title = pob::title_with_config(pob, &config).into();
        if let Some(version) = pob.max_tree_version() {
            title = format!("{} [{}]", title, version).into();
        }

        let description = meta::get_paste_summary(pob).join("\n").into();

        let image = crate::assets::ascendancy_image(pob.ascendancy_or_class_name())
            .unwrap_or("")
            .into();
        let color = meta::get_color(pob.ascendancy_or_class_name());

        Ok(Meta {
            title,
            description,
            image,
            color,
        })
    }
}

#[component(PastePage<G>)]
pub fn paste_page(Data { id, content, pob }: Data) -> View<G> {
    let props = ViewPasteProps {
        id: PasteId::id(id),
        content,
        pob,
    };
    view! {
        ViewPaste(props)
    }
}
