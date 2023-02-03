use sycamore::prelude::*;
use pob::PathOfBuilding;

use crate::build::Build;


#[component]
pub fn PobGearPreview<'a, G: Html>(cx: Scope<'a>, build: &'a Build) -> View<G> {
    let item_set = build.pob().item_sets().into_iter().next().unwrap_or_default();

    let slots = slots(&item_set.gear)
        .into_iter()
        .map(|(name, item)| {
            let src = item.and_then(item_image_url).unwrap_or_default();
            let class = format!("item {name}");
            view! { cx,
                img(src=src, class=class) {}
            }
        })
        .collect();
    let slots = View::new_fragment(slots);

    view! { cx,
        div(class="inventory") {
            (slots)
        }
    }
}

fn item_image_url(item: &str) -> Option<String> {
    let name = item.lines().filter(|line| line != &"Rarity: RARE").skip(1).next()?;
    let name = percent_encoding::utf8_percent_encode(name, percent_encoding::NON_ALPHANUMERIC);
    Some(format!("https://assets.pobb.in/1/{name}.webp"))
}

fn slots<'a>(gear: &pob::Gear<'a>) -> [(&'static str, Option<&'a str>); 15] {
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
        ("flask1", gear.flask1),
        ("flask2", gear.flask2),
        ("flask3", gear.flask3),
        ("flask4", gear.flask4),
        ("flask5", gear.flask5),
    ]
}
