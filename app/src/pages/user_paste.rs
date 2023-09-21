use std::{borrow::Cow, convert::TryInto};

use ::pob::PathOfBuildingExt;
use shared::{Id, User, UserPasteId};
use sycamore::prelude::*;

use crate::{
    build::Build,
    components::{PasteToolbox, ViewPaste, ViewPasteProps},
    future::LocalBoxFuture,
    meta, pob,
    router::RoutedComponent,
    svg,
    utils::{deserialize_attribute, find_attribute, find_text, serialize_for_attribute},
    Meta, Result,
};

pub struct UserPastePage {
    id: UserPasteId,
    title: Option<String>,
    last_modified: u64,
    build: Build,
}

impl RoutedComponent for UserPastePage {
    type RouteArg = (User, Id);

    fn from_context((user, id): Self::RouteArg, ctx: crate::Context) -> Result<Self> {
        let mut paste = ctx.into_paste().unwrap();
        let title = paste.metadata.take().map(|m| m.title);

        Ok(Self {
            id: UserPasteId { user, id },
            title,
            last_modified: paste.last_modified,
            build: paste.try_into()?,
        })
    }

    fn from_hydration((user, id): Self::RouteArg, element: web_sys::Element) -> Result<Self> {
        let content = find_text(&element, "[data-marker-content]").unwrap_or_default();
        let title = find_text(&element, "[data-marker-title]");
        let last_modified = find_attribute(&element, "data-last-modified").unwrap_or_default();
        let data = deserialize_attribute(&element, "data-data").unwrap_or_default();

        let build = Build::new(content, data)?;
        Ok(Self {
            id: UserPasteId { user, id },
            title,
            last_modified,
            build,
        })
    }

    fn from_dynamic<'a>((user, id): Self::RouteArg) -> LocalBoxFuture<'a, Result<Self>> {
        let id = UserPasteId { user, id }.into();
        Box::pin(async move {
            let mut paste = crate::api::get_paste(&id).await?;
            let title = paste.metadata.take().map(|x| x.title);

            Ok(Self {
                id: id.unwrap_user(),
                title,
                last_modified: paste.last_modified,
                build: paste.try_into()?,
            })
        })
    }

    fn meta(&self) -> Result<Meta> {
        let pob = self.build.pob();
        let config = pob::TitleConfig { no_level: true };

        let title: Cow<str> = self
            .title
            .as_ref()
            .map(|x| x.into())
            .unwrap_or_else(|| pob::title_with_config(pob, &config).into());
        let title = match pob.max_tree_version() {
            Some(version) => format!("{title} [{version}] by {}", self.id.user),
            None => format!("{title} by {}", self.id.user),
        }
        .into();

        let description = meta::get_paste_summary(pob).join("\n").into();

        let image = crate::assets::ascendancy_image(pob.ascendancy_or_class()).into();
        let color = meta::get_color(pob.ascendancy_or_class());

        let oembed = format!("/oembed.json?user={}", self.id.user).into();

        Ok(Meta {
            title,
            description,
            image,
            color,
            oembed,
        })
    }

    fn render<G: Html>(self, cx: Scope) -> View<G> {
        view! { cx, UserPastePageComponent(self) }
    }
}

#[component]
fn UserPastePageComponent<G: Html>(
    cx: Scope,
    UserPastePage {
        id,
        title,
        last_modified,
        build,
    }: UserPastePage,
) -> View<G> {
    let build = create_ref(cx, build);
    let back_to_user = create_ref(cx, id.to_user_url());
    let deleted = create_signal(cx, false);

    let data = serialize_for_attribute::<G>(build.data());

    let name = id.user.clone();
    let props = ViewPasteProps {
        id: id.clone().into(),
        title,
        last_modified,
        build,
    };

    create_effect(cx, || {
        if *deleted.get() {
            sycamore_router::navigate(back_to_user);
        }
    });

    view! { cx,
        div(data-nodes=data) {}
        div(class="flex justify-between") {
            a(href=back_to_user, class="flex items-center mb-4 text-sky-400") {
                span(dangerously_set_inner_html=svg::BACK, class="h-[16px] mr-2")
                    span() { (name) } "'s builds"
            }
            PasteToolbox(id=id, on_delete=deleted)
        }
        ViewPaste(props)
    }
}
