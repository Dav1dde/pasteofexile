use crate::{async_callback, memo, memo_cond, session::SessionValue, svg};
use shared::model::UserPasteId;
use sycamore::{context::use_context, prelude::*};

pub struct PasteToolboxProps {
    pub id: UserPasteId,
    pub on_delete: Signal<bool>,
}

#[component(PasteToolbox<G>)]
pub fn paste_toolbox(
    PasteToolboxProps {
        id,
        on_delete: _on_delete,
    }: PasteToolboxProps,
) -> View<G> {
    let session = use_context::<SessionValue>();

    let is_current_user = memo!(session, id, {
        let session = session.get();
        Some(id.user.as_str()) == session.user().map(|u| u.name.as_str())
    });

    let on_delete = async_callback!(
        id,
        _on_delete,
        {
            match crate::api::delete_paste(&id.into()).await {
                Err(err) => log::error!("deletion failed: {:?}", err),
                Ok(_) => _on_delete.set(true),
            }
        },
        {
            // TODO: show paste identifier/title
            let message = format!("Are you sure you want to delete this build?");
            web_sys::window()
                .unwrap()
                .confirm_with_message(&message)
                .unwrap_or_default()
        }
    );

    let edit_href = id.to_paste_edit_url();

    let controls = memo_cond!(is_current_user, "flex justify-end gap-2 h-4", "hidden");
    view! {
        div(class=*controls.get()) {
            a(href=edit_href, class="cursor-pointer", title="Edit", dangerously_set_inner_html=svg::PEN) {}
            span(on:click=on_delete,
                 class="text-red-600 cursor-pointer",
                 title="Delete",
                 dangerously_set_inner_html=svg::TRASH) {}
        }
    }
}
