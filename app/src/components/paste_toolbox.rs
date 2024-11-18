use shared::UserPasteId;
use sycamore::prelude::*;

use crate::{
    session::SessionValue,
    svg,
    utils::{async_callback, memo_cond},
};

#[derive(Prop)]
pub struct PasteToolboxProps<'a> {
    pub id: UserPasteId,
    pub on_delete: &'a Signal<bool>,
}

#[component]
pub fn PasteToolbox<'a, G: Html>(
    cx: Scope<'a>,
    PasteToolboxProps { id, on_delete }: PasteToolboxProps<'a>,
) -> View<G> {
    let session = sycamore::reactive::use_context::<SessionValue>(cx);
    let id = create_ref(cx, id);

    let is_current_user = create_memo(cx, || {
        let session = session.get();
        Some(id.user.as_str()) == session.user().map(|u| u.name.as_str())
    });

    // TODO wtf is this
    let on_delete_cb = async_callback!(
        cx,
        {
            match crate::api::delete_paste(id).await {
                Err(err) => tracing::error!("deletion failed: {:?}", err),
                Ok(_) => on_delete.set(true),
            }
        },
        {
            // TODO: show paste identifier/title
            let message = "Are you sure you want to delete this build?".to_owned();
            web_sys::window()
                .unwrap()
                .confirm_with_message(&message)
                .unwrap_or_default()
        }
    );

    let controls = memo_cond!(
        cx,
        is_current_user,
        {
            let edit_href = id.to_paste_edit_url().into_cow();
            let on_edit = move |ev: web_sys::Event| {
                sycamore_router::navigate(&edit_href);
                ev.stop_propagation();
                ev.prevent_default();
            };

            view! { cx,
                div(class="flex justify-end gap-2 h-4") {
                    // Can't just use a link here, the parent might stop propagation
                    // Also, this is not a link that can be just browsed to, it needs
                    // to be clicked and opened client side.
                    a(on:click=on_edit,
                      href="javascript:void(0)",
                      class="w-4 cursor-pointer",
                      title="Edit",
                      dangerously_set_inner_html=svg::PEN) {}
                    span(on:click=on_delete_cb,
                         class="w-4 text-red-600 cursor-pointer",
                         title="Delete",
                         dangerously_set_inner_html=svg::TRASH) {}
                }
            }
        },
        view! { cx, }
    );

    view! { cx,
        div() { (&*controls.get()) }
    }
}
