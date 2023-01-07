use std::convert::TryInto;

use ::pob::PathOfBuildingExt;
use shared::model::PasteId;
use sycamore::prelude::*;

use crate::{
    build::Build,
    components::{ViewPaste, ViewPasteProps},
    future::LocalBoxFuture,
    meta, pob,
    router::RoutedComponent,
    utils::{deserialize_attribute, find_attribute, find_text, serialize_for_attribute},
    Meta, Result,
};

pub struct Data {
    id: String,
    title: Option<String>,
    last_modified: u64,
    build: Build,
}

impl<G: Html> RoutedComponent<G> for PastePage<G> {
    type RouteArg = String;

    fn from_context(id: Self::RouteArg, ctx: crate::Context) -> Result<Data> {
        let mut paste = ctx.into_paste().unwrap();
        let title = paste.metadata.take().and_then(|m| m.title);

        Ok(Data {
            id,
            title,
            last_modified: paste.last_modified,
            build: paste.try_into()?,
        })
    }

    fn from_hydration(id: Self::RouteArg, element: web_sys::Element) -> Result<Data> {
        let content = find_text(&element, "[data-marker-content]").unwrap_or_default();
        let title = find_text(&element, "[data-marker-title]");
        let last_modified = find_attribute(&element, "data-last-modified").unwrap_or_default();
        let nodes = deserialize_attribute(&element, "data-nodes").unwrap_or_default();

        let build = Build::new(content, nodes)?;
        Ok(Data {
            id,
            title,
            last_modified,
            build,
        })
    }

    fn from_dynamic<'a>(id: Self::RouteArg) -> LocalBoxFuture<'a, Result<Data>> {
        Box::pin(async move {
            // TODO: get rid of this clone, there needs to be a better way to pass this around
            let mut paste = crate::api::get_paste(&PasteId::new_id(id.clone())).await?;
            let title = paste.metadata.take().and_then(|x| x.title);

            Ok(Data {
                id,
                title,
                last_modified: paste.last_modified,
                build: paste.try_into()?,
            })
        })
    }

    fn meta(arg: &Data) -> Result<Meta> {
        let pob = arg.build.pob();

        let config = pob::TitleConfig { no_level: true };
        let mut title = arg
            .title
            .as_ref()
            .map(|x| x.to_owned())
            .unwrap_or_else(|| pob::title_with_config(pob, &config))
            .into();
        if let Some(version) = pob.max_tree_version() {
            title = format!("{title} [{version}]").into();
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
            ..Default::default()
        })
    }
}

#[component(PastePage<G>)]
pub fn paste_page(
    Data {
        id,
        title,
        last_modified,
        build,
    }: Data,
) -> View<G> {
    let data_nodes = serialize_for_attribute(build.nodes());
    let props = ViewPasteProps {
        id: PasteId::new_id(id),
        title,
        last_modified,
        build,
    };
    view! {
        div(data-nodes=data_nodes) {}
        ViewPaste(props)
    }
}
