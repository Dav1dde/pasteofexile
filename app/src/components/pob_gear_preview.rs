use pob::PathOfBuilding;
use sycamore::prelude::*;

use crate::{build::Build, consts::IMG_ONERROR_EMPTY};

#[component]
pub fn PobGearPreview<'a, G: Html>(cx: Scope<'a>, build: &'a Build) -> View<G> {
    let item_set = build
        .pob()
        .item_sets()
        .into_iter()
        .next()
        .unwrap_or_default();

    let slots = slots(&item_set.gear)
        .into_iter()
        .map(|(name, item)| render_items(cx, name, item))
        .collect();
    let slots = View::new_fragment(slots);

    let flasks = flasks(&item_set.gear)
        .into_iter()
        .map(|(name, item)| render_items(cx, name, item))
        .collect();
    let flasks = View::new_fragment(flasks);

    view! { cx,
        div(class="flex justify-center") {
            div(class="inventory flex-initial w-full lg:w-[65%] justify-center bg-slate-900 rounded-xl px-5 py-7") {
                (slots)
                div(class="flasks") {
                    (flasks)
                }
            }
        }
    }
}

fn render_items<G: Html>(cx: Scope, name: &'static str, base: Option<&str>) -> View<G> {
    let src = base
        .and_then(|item| pob::Item::parse(item).ok())
        .and_then(item_image_url)
        .unwrap_or_default();

    let class = format!("item {name}");
    view! { cx,
        img(src=src, class=class, onerror=IMG_ONERROR_EMPTY) {}
    }
}

fn item_image_url(item: pob::Item<'_>) -> Option<String> {
    let name = percent_encoding::utf8_percent_encode(item.base, percent_encoding::NON_ALPHANUMERIC);
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
