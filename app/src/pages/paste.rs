use std::convert::TryInto;

use ::pob::PathOfBuildingExt;
use shared::{Id, PasteId};
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

pub struct PastePage {
    id: Id,
    title: Option<String>,
    last_modified: u64,
    build: Build,
}

impl RoutedComponent for PastePage {
    type RouteArg = Id;

    fn from_context(id: Self::RouteArg, ctx: crate::Context) -> Result<Self> {
        let mut paste = ctx.into_paste().unwrap();
        let title = paste.metadata.take().map(|m| m.title);

        Ok(Self {
            id,
            title,
            last_modified: paste.last_modified,
            build: paste.try_into()?,
        })
    }

    fn from_hydration(id: Self::RouteArg, element: web_sys::Element) -> Result<Self> {
        let content = find_text(&element, "[data-marker-content]").unwrap_or_default();
        let title = find_text(&element, "[data-marker-title]");
        let last_modified = find_attribute(&element, "data-last-modified").unwrap_or_default();
        let nodes = deserialize_attribute(&element, "data-nodes").unwrap_or_default();

        let build = Build::new(content, nodes)?;
        Ok(Self {
            id,
            title,
            last_modified,
            build,
        })
    }

    fn from_dynamic<'a>(id: Self::RouteArg) -> LocalBoxFuture<'a, Result<Self>> {
        let id = id.into();
        Box::pin(async move {
            let mut paste = crate::api::get_paste(&id).await?;
            let title = paste.metadata.take().map(|x| x.title);

            Ok(Self {
                id: id.unwrap_paste(),
                title,
                last_modified: paste.last_modified,
                build: paste.try_into()?,
            })
        })
    }

    fn meta(&self) -> Result<Meta> {
        let pob = self.build.pob();

        let config = pob::TitleConfig { no_level: true };
        let mut title = self
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

    fn render<G: Html>(self, cx: Scope) -> View<G> {
        view! { cx, PastePageComponent(self) }
    }
}

#[component]
fn PastePageComponent<G: Html>(
    cx: Scope,
    PastePage {
        id,
        title,
        last_modified,
        build,
    }: PastePage,
) -> View<G> {
    let data_nodes = serialize_for_attribute::<G>(build.nodes());
    let props = ViewPasteProps {
        id: PasteId::Paste(id),
        title,
        last_modified,
        build: create_ref(cx, build),
    };
    view! { cx,
        div(data-nodes=data_nodes) {}
        ViewPaste(props)
    }
}
