use std::num::NonZeroU8;

use shared::{Id, User, UserPasteId};
use sycamore::prelude::*;

use crate::{
    components::{CreatePaste, CreatePasteProps},
    future::LocalBoxFuture,
    router::RoutedComponent,
    utils::{find_attribute, find_text},
    Meta, Result,
};

pub struct UserEditPastePage {
    id: UserPasteId,
    title: Option<String>,
    content: String,
    rank: Option<NonZeroU8>,
}

impl RoutedComponent for UserEditPastePage {
    type RouteArg = (User, Id);

    fn from_context((user, id): Self::RouteArg, ctx: crate::Context) -> Result<Self> {
        let paste = ctx.into_paste().unwrap();
        Ok(Self {
            id: UserPasteId { user, id },
            content: paste.content,
            rank: paste.metadata.as_ref().and_then(|m| m.rank),
            title: paste.metadata.map(|m| m.title),
        })
    }

    fn from_hydration((user, id): Self::RouteArg, element: web_sys::Element) -> Result<Self> {
        let content = find_text(&element, "[data-marker-content]").unwrap_or_default();
        let title = find_text(&element, "[data-marker-title]");
        let rank = find_attribute(&element, "data-rank");

        Ok(Self {
            id: UserPasteId { user, id },
            content,
            title,
            rank,
        })
    }

    fn from_dynamic<'a>((user, id): Self::RouteArg) -> LocalBoxFuture<'a, Result<Self>> {
        let id = UserPasteId { user, id }.into();
        Box::pin(async move {
            let paste = crate::api::get_paste(&id).await?;
            Ok(Self {
                id: id.unwrap_user(),
                content: paste.content,
                rank: paste.metadata.as_ref().and_then(|m| m.rank),
                title: paste.metadata.map(|x| x.title),
            })
        })
    }

    fn meta(&self) -> Result<Meta> {
        Ok(Meta {
            title: "Edit Build".into(),
            description: "".into(),
            ..Default::default()
        })
    }

    fn render<G: Html>(self, cx: Scope) -> View<G> {
        let Self {
            id,
            content,
            title,
            rank,
        } = self;
        let props = CreatePasteProps::Update {
            id,
            content,
            title,
            rank,
        };
        view! { cx,
            CreatePaste(props)
        }
    }
}
