use crate::{
    components::{ViewPaste, ViewPasteProps},
    future::LocalBoxFuture,
    meta, pob,
    router::RoutedComponent,
    utils::find_text,
    Meta, Result,
};
use ::pob::{PathOfBuildingExt, SerdePathOfBuilding};
use shared::model::PasteId;
use std::rc::Rc;
use sycamore::prelude::*;

pub struct Data {
    id: String,
    title: Option<String>,
    content: String,
    pob: Rc<SerdePathOfBuilding>,
}

impl<G: Html> RoutedComponent<G> for PastePage<G> {
    type RouteArg = String;

    fn from_context(id: Self::RouteArg, ctx: crate::Context) -> Result<Data> {
        let paste = ctx.get_paste().unwrap();
        Ok(Data {
            id,
            title: paste.metadata().map(|m| m.title.to_owned()),
            content: paste.content().to_owned(),
            pob: paste.path_of_building()?,
        })
    }

    fn from_hydration(id: Self::RouteArg, element: web_sys::Element) -> Result<Data> {
        let content = find_text(&element, "[data-marker-content]").unwrap_or_default();
        let title = find_text(&element, "[data-marker-title]");

        let pob = Rc::new(SerdePathOfBuilding::from_export(&content)?);
        Ok(Data {
            id,
            title,
            content,
            pob,
        })
    }

    fn from_dynamic<'a>(id: Self::RouteArg) -> LocalBoxFuture<'a, Result<Data>> {
        Box::pin(async move {
            // TODO: get rid of this clone, there needs to be a better way to pass this around
            let paste = crate::api::get_paste(&PasteId::new_id(id.clone())).await?;
            let pob = Rc::new(SerdePathOfBuilding::from_export(&paste.content)?);
            let title = paste.metadata.map(|x| x.title);
            Ok(Data {
                id,
                title,
                content: paste.content,
                pob,
            })
        })
    }

    fn meta(arg: &Data) -> Result<Meta> {
        let pob: &SerdePathOfBuilding = &*arg.pob;

        let config = pob::TitleConfig { no_title: true };
        let mut title = arg
            .title
            .as_ref()
            .map(|x| x.to_owned())
            .unwrap_or_else(|| pob::title_with_config(pob, &config))
            .into();
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
pub fn paste_page(
    Data {
        id,
        title,
        content,
        pob,
    }: Data,
) -> View<G> {
    let props = ViewPasteProps {
        id: PasteId::new_id(id),
        title,
        content,
        pob,
    };
    view! {
        ViewPaste(props)
    }
}
