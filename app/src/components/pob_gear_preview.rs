use pob::PathOfBuilding;
use sycamore::prelude::*;
use wasm_bindgen::JsCast;

use super::{PobColoredSelect, PobItem, PobItemSet, Popup};
use crate::build::Build;

#[component]
pub fn PobGearPreview<'a, G: Html>(cx: Scope<'a>, build: &'a Build) -> View<G> {
    let gv = build.game_version();
    let attach = create_signal(cx, None);
    let current_item = create_signal(cx, None);

    let popup = create_memo(cx, move || {
        if let Some(item) = &*current_item.get() {
            view! { cx, PobItem(game_version=gv, item=*item) }
        } else {
            view! { cx, }
        }
    });

    let item_sets = build.item_sets();
    let on_change = move |id| {
        if let Some(id) = id {
            build.set_current_item_set(id);
        }
    };

    let items = create_memo(cx, move || {
        let item_set = build.current_item_set().map(|s| create_ref(cx, s));
        view! { cx,
            PobItemSet(
                build_=build,
                item_set=item_set,
                current_item=current_item,
            )
        }
    });

    let mouseover = |event: web_sys::Event| {
        let a = event
            .target()
            .filter(|target| target.is_instance_of::<web_sys::HtmlImageElement>())
            .map(|target| target.unchecked_into::<web_sys::Element>());

        attach.set(a);
    };
    let mouseout = |_: web_sys::Event| attach.set(None);

    let select_classes = if item_sets.len() >= 2 {
        "-mb-5"
    } else {
        "hidden"
    };

    view! { cx,
        Popup(attach=attach, parent=None) { (&*popup.get()) }
        div(class=select_classes) {
            PobColoredSelect(options=item_sets, selected=build.current_item_set_id(), label="Select gear set", on_change=on_change)
        }
        div(class="flex flex-col justify-center mt-5 sm:px-3",
            on:mouseover=mouseover,
            on:mouseout=mouseout,
        ) {
            div(
                class="inventory flex-initial w-full justify-center rounded-xl"
            ) {
                (&*items.get())
            }
        }
    }
}
