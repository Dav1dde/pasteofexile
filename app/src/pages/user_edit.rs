use crate::{
    components::{CreatePaste, CreatePasteProps},
    future::LocalBoxFuture,
    router::RoutedComponent,
    utils::find_text,
    Meta, Result,
};
use shared::{model::UserPasteId, User};
use sycamore::prelude::*;

pub struct Data {
    id: UserPasteId,
    title: Option<String>,
    content: String,
}

impl<G: Html> RoutedComponent<G> for UserEditPastePage<G> {
    type RouteArg = (User, String);

    fn from_context((user, id): Self::RouteArg, ctx: crate::Context) -> Result<Data> {
        let paste = ctx.into_paste().unwrap();
        Ok(Data {
            id: UserPasteId { user, id },
            title: paste.metadata.map(|m| m.title),
            content: paste.content,
        })
    }

    fn from_hydration((user, id): Self::RouteArg, element: web_sys::Element) -> Result<Data> {
        let content = find_text(&element, "[data-marker-content]").unwrap_or_default();
        let title = find_text(&element, "[data-marker-title]");

        Ok(Data {
            id: UserPasteId { user, id },
            content,
            title,
        })
    }

    fn from_dynamic<'a>((user, id): Self::RouteArg) -> LocalBoxFuture<'a, Result<Data>> {
        let id = shared::model::PasteId::new_user(user, id);
        Box::pin(async move {
            let paste = crate::api::get_paste(&id).await?;
            Ok(Data {
                id: id.unwrap_user(),
                content: paste.content,
                title: paste.metadata.map(|x| x.title),
            })
        })
    }

    fn meta(_arg: &Data) -> Result<Meta> {
        Ok(Meta {
            title: "Edit Build".into(),
            description: "".into(),
            ..Default::default()
        })
    }
}

#[component(UserEditPastePage<G>)]
pub fn user_edit_paste_page(Data { id, content, title }: Data) -> View<G> {
    let props = CreatePasteProps::Update { id, content, title };
    view! {
        CreatePaste(props)
    }
}
