use crate::{
    components::{PasteToolbox, PasteToolboxProps, ViewPaste, ViewPasteProps},
    effect,
    future::LocalBoxFuture,
    meta, pob,
    router::RoutedComponent,
    svg,
    utils::find_text,
    Meta, Result,
};
use ::pob::{PathOfBuildingExt, SerdePathOfBuilding};
use shared::model::{PasteId, UserPasteId};
use std::rc::Rc;
use sycamore::prelude::*;

pub struct Data {
    id: UserPasteId,
    title: Option<String>,
    content: String,
    pob: Rc<SerdePathOfBuilding>,
}

impl<G: Html> RoutedComponent<G> for UserPastePage<G> {
    type RouteArg = (String, String);

    fn from_context((user, id): Self::RouteArg, ctx: crate::Context) -> Result<Data> {
        let paste = ctx.get_paste().unwrap();
        Ok(Data {
            id: UserPasteId { user, id },
            title: paste.metadata().map(|m| m.title.to_owned()),
            content: paste.content().to_owned(),
            pob: paste.path_of_building()?,
        })
    }

    fn from_hydration((user, id): Self::RouteArg, element: web_sys::Element) -> Result<Data> {
        let content = find_text(&element, "[data-marker-content]").unwrap_or_default();
        let title = find_text(&element, "[data-marker-title]");

        let pob = Rc::new(SerdePathOfBuilding::from_export(&content)?);
        Ok(Data {
            id: UserPasteId { user, id },
            title,
            content,
            pob,
        })
    }

    fn from_dynamic<'a>((user, id): Self::RouteArg) -> LocalBoxFuture<'a, Result<Data>> {
        Box::pin(async move {
            // TODO: get rid of these clones
            let paste = crate::api::get_paste(&PasteId::new_user(user.clone(), id.clone())).await?;
            let pob = Rc::new(SerdePathOfBuilding::from_export(&paste.content)?);
            Ok(Data {
                id: UserPasteId { user, id },
                title: paste.metadata.map(|x| x.title),
                content: paste.content,
                pob,
            })
        })
    }

    fn meta(arg: &Data) -> Result<Meta> {
        let pob: &SerdePathOfBuilding = &*arg.pob;

        let config = pob::TitleConfig { no_title: true };
        let mut title = pob::title_with_config(pob, &config).into();
        if let Some(version) = pob.max_tree_version() {
            title = format!("{title} [{version}] by {}", arg.id.user).into();
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

#[component(UserPastePage<G>)]
pub fn user_paste_page(
    Data {
        id,
        title,
        content,
        pob,
    }: Data,
) -> View<G> {
    let deleted = Signal::new(false);

    let back_to_user = id.to_user_url();
    let navigate_after_delete = back_to_user.clone();

    let toolbox = PasteToolboxProps {
        id: id.clone(),
        on_delete: deleted.clone(),
    };
    let name = id.user.clone();
    let props = ViewPasteProps {
        id: id.into(),
        title,
        content,
        pob,
    };

    effect!(deleted, {
        if *deleted.get() {
            sycamore_router::navigate(&navigate_after_delete);
        }
    });

    view! {
        div(class="flex justify-between") {
            a(href=back_to_user, class="flex items-center mb-12 text-sky-400") {
                span(dangerously_set_inner_html=svg::BACK, class="h-[16px] mr-2")
                    span() { (name) } "'s builds"
            }
            PasteToolbox(toolbox)
        }
        ViewPaste(props)
    }
}
