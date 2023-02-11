use itertools::Itertools;
use pob::PathOfBuilding;
use sycamore::prelude::*;
use wasm_bindgen::JsCast;

use super::{PobColoredSelect, PobItem, PobItemSet, Popup};
use crate::build::Build;

#[component]
pub fn PobGearPreview<'a, G: Html>(cx: Scope<'a>, build: &'a Build) -> View<G> {
    let attach = create_signal(cx, None);
    let current_item = create_signal(cx, None);

    let popup = create_memo(cx, move || {
        if let Some(item) = &*current_item.get() {
            view! { cx, PobItem(*item) }
        } else {
            view! { cx, }
        }
    });

    let item_sets = create_ref(cx, build.item_sets());

    let item_set = item_sets.iter().find_or_first(|set| set.is_selected);
    let item_set = create_signal(cx, item_set);

    let options = item_sets
        .iter()
        .map(|item_set| {
            item_set
                .title
                .map(|s| s.to_owned())
                .unwrap_or_else(|| item_set.id.to_string())
        })
        .collect();

    let selected = item_sets.iter().position(|set| set.is_selected);
    let on_change = move |index| {
        let Some(index) = index else { return };
        item_set.set(item_sets.get(index));
    };

    let items = create_memo(cx, move || {
        let item_set = item_set.get();
        view! { cx,
            PobItemSet(
                build_=build,
                item_set=*item_set,
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
        Popup(attach=attach) { (&*popup.get()) }
        div(class=select_classes) {
            PobColoredSelect(options=options, selected=selected, on_change=on_change)
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
