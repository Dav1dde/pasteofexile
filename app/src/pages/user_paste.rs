use crate::{
    build::Build,
    components::{PasteToolbox, PasteToolboxProps, ViewPaste, ViewPasteProps},
    effect,
    future::LocalBoxFuture,
    meta, pob,
    router::RoutedComponent,
    svg,
    utils::{deserialize_attribute, find_attribute, find_text, serialize_for_attribute},
    Meta, Result,
};
use ::pob::PathOfBuildingExt;
use shared::{
    model::{PasteId, UserPasteId},
    User,
};
use std::{borrow::Cow, convert::TryInto};
use sycamore::prelude::*;

pub struct Data {
    id: UserPasteId,
    title: Option<String>,
    last_modified: u64,
    build: Build,
}

impl<G: Html> RoutedComponent<G> for UserPastePage<G> {
    type RouteArg = (User, String);

    fn from_context((user, id): Self::RouteArg, ctx: crate::Context) -> Result<Data> {
        let mut paste = ctx.into_paste().unwrap();
        let title = paste.metadata.take().map(|m| m.title);

        Ok(Data {
            id: UserPasteId { user, id },
            title,
            last_modified: paste.last_modified,
            build: paste.try_into()?,
        })
    }

    fn from_hydration((user, id): Self::RouteArg, element: web_sys::Element) -> Result<Data> {
        let content = find_text(&element, "[data-marker-content]").unwrap_or_default();
        let title = find_text(&element, "[data-marker-title]");
        let last_modified = find_attribute(&element, "data-last-modified").unwrap_or_default();
        let nodes = deserialize_attribute(&element, "data-nodes").unwrap_or_default();

        let build = Build::new(content, nodes)?;
        Ok(Data {
            id: UserPasteId { user, id },
            title,
            last_modified,
            build,
        })
    }

    fn from_dynamic<'a>((user, id): Self::RouteArg) -> LocalBoxFuture<'a, Result<Data>> {
        Box::pin(async move {
            // TODO: get rid of these clones
            let mut paste =
                crate::api::get_paste(&PasteId::new_user(user.clone(), id.clone())).await?;
            let title = paste.metadata.take().map(|x| x.title);

            Ok(Data {
                id: UserPasteId { user, id },
                title,
                last_modified: paste.last_modified,
                build: paste.try_into()?,
            })
        })
    }

    fn meta(arg: &Data) -> Result<Meta> {
        let pob = arg.build.pob();
        let config = pob::TitleConfig { no_level: true };

        let title: Cow<str> = arg
            .title
            .as_ref()
            .map(|x| x.into())
            .unwrap_or_else(|| pob::title_with_config(pob, &config).into());
        let title = match pob.max_tree_version() {
            Some(version) => format!("{title} [{version}] by {}", arg.id.user),
            None => format!("{title} by {}", arg.id.user),
        }
        .into();

        let description = meta::get_paste_summary(pob).join("\n").into();

        let image = crate::assets::ascendancy_image(pob.ascendancy_or_class_name())
            .unwrap_or("")
            .into();
        let color = meta::get_color(pob.ascendancy_or_class_name());

        let oembed = format!("/oembed.json?user={}", arg.id.user).into();

        Ok(Meta {
            title,
            description,
            image,
            color,
            oembed,
        })
    }
}

#[component(UserPastePage<G>)]
pub fn user_paste_page(
    Data {
        id,
        title,
        last_modified,
        build,
    }: Data,
) -> View<G> {
    let deleted = Signal::new(false);

    let back_to_user = id.to_user_url();
    let navigate_after_delete = back_to_user.clone();

    let data_nodes = serialize_for_attribute(build.nodes());

    let toolbox = PasteToolboxProps {
        id: id.clone(),
        on_delete: deleted.clone(),
    };
    let name = id.user.clone();
    let props = ViewPasteProps {
        id: id.into(),
        title,
        last_modified,
        build,
    };

    effect!(deleted, {
        if *deleted.get() {
            sycamore_router::navigate(&navigate_after_delete);
        }
    });

    view! {
        div(data-nodes=data_nodes) {}
        div(class="flex justify-between") {
            a(href=back_to_user, class="flex items-center mb-4 text-sky-400") {
                span(dangerously_set_inner_html=svg::BACK, class="h-[16px] mr-2")
                    span() { (name) } "'s builds"
            }
            PasteToolbox(toolbox)
        }
        ViewPaste(props)
    }
}
