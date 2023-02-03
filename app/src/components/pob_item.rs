use itertools::Itertools;
use sycamore::prelude::*;

use crate::utils::view_cond;

#[component]
pub fn PobItem<'a, G: Html>(cx: Scope<'a>, item: pob::Item<'a>) -> View<G> {
    let render_mod = |m: pob::Mod<'a>| {
        let line: String = m.line.to_owned();

        let style = if m.crafted {
            "color: #b4b4ff"
        } else if m.fractured {
            "color: #a29162"
        } else {
            "color: #88f"
        };

        view! { cx, li(style=style) { (line) } }
    };

    let influence1 = item.influence1.map_or_else(View::empty, move |influence| view! { cx,
        div(class="absolute left-[2px] top-0 bottom-0 w-[26px]", style=influence_style(influence)) {}
    });
    let influence2 = item.influence2.map_or_else(View::empty, move |influence| view! { cx,
        div(class="absolute right-[2px] top-0 bottom-0 w-[26px]", style=influence_style(influence)) {}
    });

    let enchants = item.enchants().map(render_mod).collect_vec();
    let implicits = item.implicits().map(render_mod).collect_vec();
    let explicits = item.explicits().map(render_mod).collect_vec();

    let mut unmet = Vec::new();
    if item.split {
        unmet.push(view! { cx, li(style="color: #88f") { "Split" } });
    }
    if item.mirrored {
        unmet.push(view! { cx, li(style="color: #88f") { "Mirrored" } });
    }
    if item.corrupted {
        unmet.push(view! { cx, li(style="color: #d20000") { "Corrupted" } });
    }

    let name = item.name.unwrap_or_default().to_owned();
    let base = item.base.to_owned();

    let magic_or_normal = matches!(item.rarity, pob::Rarity::Normal | pob::Rarity::Magic);
    let base = view_cond!(cx, !magic_or_normal, { div() { (base) } });

    let header_style = header_style(item.rarity);
    let data_rarity = rarity_str(item.rarity);

    view! { cx,
        div(class="bg-black/[0.8] text-center pob-item", data-rarity=data_rarity) {
            div(class="px-7 py-2 bg-contain relative", style=header_style) {
                (influence1)
                div { (name) }
                (base)
                (influence2)
            }
            div(class="p-2 pt-1") {
                Mods(enchants)
                Mods(implicits)
                Mods(explicits)
                Mods(unmet)
            }
        }
    }
}

#[component]
pub fn Mods<G: Html>(cx: Scope, mods: Vec<View<G>>) -> View<G> {
    if mods.is_empty() {
        return view! { cx, };
    }

    let content = View::new_fragment(mods);
    view! { cx, ul { (content) } }
}

fn rarity_str(rarity: pob::Rarity) -> &'static str {
    match rarity {
        pob::Rarity::Normal => "White",
        pob::Rarity::Magic => "Magic",
        pob::Rarity::Rare => "Rare",
        pob::Rarity::Unique => "Unique",
    }
}

fn header_style(rarity: pob::Rarity) -> String {
    let name = rarity_str(rarity);
    let color = match rarity {
        pob::Rarity::Normal => "#c8c8c8",
        pob::Rarity::Magic => "#88f",
        pob::Rarity::Rare => "#ff7",
        pob::Rarity::Unique => "#af6025",
    };

    const BASE: &str = "https://assets.pobb.in/1/Art/2DArt/UIImages/InGame/ItemsHeader";

    format!(
        "color: {color}; background: \
        url({BASE}{name}Left.webp) top left / contain no-repeat, \
        url({BASE}{name}Right.webp) top right / contain no-repeat, \
        url({BASE}{name}Middle.webp) top left / contain repeat-x"
    )
}

fn influence_style(influence: pob::Influence) -> &'static str {
    macro_rules! inf {
        ($name:expr) => {
            concat!(
                "background: url(https://assets.pobb.in/1/Art/2DArt/UIImages/InGame/",
                $name,
                "ItemSymbol.webp) center / contain no-repeat"
            )
        };
    }

    match influence {
        pob::Influence::Shaper => inf!("Shaper"),
        pob::Influence::Elder => inf!("Elder"),
        pob::Influence::Crusader => inf!("Crusader"),
        pob::Influence::Hunter => inf!("Basilisk"),
        pob::Influence::Redeemer => inf!("Eyrie"),
        pob::Influence::Warlord => inf!("Judicator"),
        pob::Influence::SearingExarch => inf!("CleansingFire"),
        pob::Influence::EaterOfWorlds => inf!("Tangled"),
        pob::Influence::Synthesis => inf!("Synthesised"),
        pob::Influence::Fracture => inf!("Fractured"),
    }
}
