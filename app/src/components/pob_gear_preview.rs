use itertools::Itertools;
use pob::PathOfBuilding;
use sycamore::prelude::*;
use wasm_bindgen::JsCast;

use super::{PobItem, Popup};
use crate::{build::Build, consts::IMG_ONERROR_EMPTY, utils::IteratorExt};

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

    let item_set = build
        .pob()
        .item_sets()
        .into_iter()
        .next()
        .unwrap_or_default();

    let slots = slots(&item_set.gear)
        .into_iter()
        .map(move |(name, item)| render_items(cx, name, item, current_item))
        .collect_view();

    let flasks = flasks(&item_set.gear)
        .into_iter()
        .map(move |(name, item)| render_items(cx, name, item, current_item))
        .collect_view();

    let sockets = create_memo(cx, move || {
        let index = build.active_tree().get();
        // TODO: maybe this should be displayed next to the tree preview?
        let mut tree_sockets = if let Some(tree) = build.pob().tree_specs().get(*index) {
            tree.sockets
                .iter()
                // There are items included which are socketed in non activated sockets
                .filter(|socket| tree.nodes.contains(&socket.node_id))
                .filter_map(|socket| build.pob().item_by_id(socket.item_id))
                .collect_vec()
        } else {
            Vec::new()
        };
        // TODO: this sorting should be better... Cluster Jewels -> Uniques -> Abyss -> Others?
        tree_sockets.sort_unstable();

        item_set
            .gear
            .sockets
            .clone()
            .into_iter()
            .chain(tree_sockets)
            .map(move |content| render_items(cx, "socket", Some(content), current_item))
            .collect_view()
    });

    let mouseover = |event: web_sys::Event| {
        let a = event
            .target()
            .filter(|target| target.is_instance_of::<web_sys::HtmlImageElement>())
            .map(|target| target.unchecked_into::<web_sys::Element>());

        attach.set(a);
    };
    let mouseout = |_: web_sys::Event| attach.set(None);

    view! { cx,
        Popup(attach=attach) { (&*popup.get()) }
        div(class="flex flex-col justify-center mt-5 sm:px-3",
                on:mouseover=mouseover,
                on:mouseout=mouseout,
            ) {
            div(
                class="inventory flex-initial w-full justify-center rounded-xl"
            ) {
                (slots)
                div(class="flasks") {
                    (flasks)
                }
                div(class="col-span-full") {}
                (&*sockets.get())
            }
        }
    }
}

fn render_items<'a, G: Html>(
    cx: Scope<'a>,
    name: &'static str,
    base: Option<&'a str>,
    current_item: &'a Signal<Option<pob::Item<'a>>>,
) -> View<G> {
    let item = base.and_then(|item| pob::Item::parse(item).ok());
    let src = item.and_then(item_image_url).unwrap_or_default();
    let alt = item
        .and_then(|item| {
            if item.rarity.is_unique() {
                item.name
            } else {
                Some(item.base)
            }
        })
        .unwrap_or(name)
        .to_owned();

    let mouseover = move |_: web_sys::Event| current_item.set(item);

    let class = format!("item {name}");
    view! { cx,
        img(src=src, class=class, alt=alt, onerror=IMG_ONERROR_EMPTY, loading="lazy", on:mouseover=mouseover) {}
    }
}

fn item_image_url(item: pob::Item<'_>) -> Option<String> {
    let name = if item.rarity.is_unique() {
        item.name.unwrap_or(item.base)
    } else {
        item.base
    };
    let name = percent_encoding::utf8_percent_encode(name, percent_encoding::NON_ALPHANUMERIC);
    Some(format!("https://assets.pobb.in/1/{name}.webp"))
}

fn slots<'a>(gear: &pob::Gear<'a>) -> [(&'static str, Option<&'a str>); 10] {
    [
        ("weapon1", gear.weapon1),
        ("weapon2", gear.weapon2),
        ("helmet", gear.helmet),
        ("body_armour", gear.body_armour),
        ("gloves", gear.gloves),
        ("boots", gear.boots),
        ("amulet", gear.amulet),
        ("ring1", gear.ring1),
        ("ring2", gear.ring2),
        ("belt", gear.belt),
    ]
}

fn flasks<'a>(gear: &pob::Gear<'a>) -> [(&'static str, Option<&'a str>); 5] {
    [
        ("flask1", gear.flask1),
        ("flask2", gear.flask2),
        ("flask3", gear.flask3),
        ("flask4", gear.flask4),
        ("flask5", gear.flask5),
    ]
}
