use crate::{
    components::{CreatePaste, CreatePasteProps},
    future::LocalBoxFuture,
    router::RoutedComponent,
    Meta, Result,
};
use sycamore::prelude::*;

pub struct Data {
    id: String,
    user: String,
    content: String,
}

impl<G: Html> RoutedComponent<G> for UserEditPastePage<G> {
    type RouteArg = (String, String);

    fn from_context((user, id): Self::RouteArg, ctx: crate::Context) -> Result<Data> {
        let paste = ctx.get_paste().unwrap();
        Ok(Data {
            id,
            user,
            content: paste.content().to_owned(),
        })
    }

    fn from_hydration((user, id): Self::RouteArg, element: web_sys::Element) -> Result<Data> {
        let content = element
            .query_selector("textarea")
            .unwrap()
            .unwrap()
            .inner_html();

        Ok(Data { id, user, content })
    }

    fn from_dynamic<'a>((user, id): Self::RouteArg) -> LocalBoxFuture<'a, Result<Data>> {
        Box::pin(async move {
            let content = crate::api::get_paste(crate::api::PasteId::UserPaste(&user, &id)).await?;
            Ok(Data { id, user, content })
        })
    }

    fn meta(_arg: &Data) -> Result<Meta> {
        // TODO: fix me
        Ok(Meta {
            title: "test".into(),
            description: "description".into(),
            image: "".into(),
            color: "",
        })
    }
}

#[component(UserEditPastePage<G>)]
pub fn user_edit_paste_page(Data { id, user, content }: Data) -> View<G> {
    let props = CreatePasteProps::Update { id, user, content };
    view! {
        CreatePaste(props)
    }
}
