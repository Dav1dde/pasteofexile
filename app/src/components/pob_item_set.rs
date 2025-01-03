use itertools::Itertools;
use pob::PathOfBuilding;
use shared::GameVersion;
use sycamore::prelude::*;

use crate::{
    build::Build,
    consts::IMG_ONERROR_EMPTY,
    utils::{click_has_ctrl, open_wiki_page, IteratorExt},
};

#[component(inline_props)]
pub fn PobItemSet<'a, 'b, G: Html>(
    cx: Scope<'a>,
    build_: &'a Build,
    item_set: Option<&'a pob::ItemSet<'a>>,
    current_item: &'a Signal<Option<pob::Item<'a>>>,
) -> View<G> {
    let build = build_;
    let gv = build.pob().game_version();

    let gear = item_set
        .map(|set| &set.gear)
        .unwrap_or_else(|| create_ref(cx, pob::Gear::default()));

    let slots = slots(gear)
        .into_iter()
        .map(move |(name, item)| render_item_str(cx, gv, name, item, current_item))
        .collect_view();

    let flasks = flasks(gv, gear)
        .into_iter()
        .map(move |(name, item)| render_item_str(cx, gv, name, item, current_item))
        .collect_view();

    let sockets = create_memo(cx, move || {
        let index = build.active_tree().get();
        // TODO: maybe this should be displayed next to the tree preview?
        let mut tree_sockets = if let Some(tree) = build.tree_specs().get(*index) {
            tree.sockets
                .iter()
                // There are items included which are socketed in non activated sockets
                .filter(|socket| tree.nodes.contains(&socket.node_id))
                .filter_map(|socket| build.item_by_id(socket.item_id))
                .filter_map(|item| pob::Item::parse(item).ok())
                .collect_vec()
        } else {
            Vec::new()
        };
        tree_sockets.sort_unstable_by_key(|item| {
            // Cluster Jewels -> Unique -> Base Name -> Name
            (
                !item.is_cluster_jewel(),
                !item.rarity.is_unique(),
                item.base,
                item.name,
            )
        });

        gear.sockets
            .clone()
            .into_iter()
            .filter_map(|item| pob::Item::parse(item).ok())
            .chain(tree_sockets)
            .map(move |item| render_item(cx, gv, "socket", Some(item), current_item))
            .collect_view()
    });

    view! { cx,
        (slots)
            div(class="flasks") {
                (flasks)
            }
        div(class="col-span-full") {}
        (&*sockets.get())
    }
}

fn render_item_str<'a, G: Html>(
    cx: Scope<'a>,
    gv: GameVersion,
    name: &'static str,
    item: Option<&'a str>,
    current_item: &'a Signal<Option<pob::Item<'a>>>,
) -> View<G> {
    render_item(
        cx,
        gv,
        name,
        item.and_then(|item| pob::Item::parse(item).ok()),
        current_item,
    )
}

fn render_item<'a, G: Html>(
    cx: Scope<'a>,
    gv: GameVersion,
    name: &'static str,
    item: Option<pob::Item<'a>>,
    current_item: &'a Signal<Option<pob::Item<'a>>>,
) -> View<G> {
    let class = format!("item {name}");

    let Some(image_name) = item.map(|item| item_image_name(&item)) else {
        // hide offhand instead of having an empty area, most of the time
        // it's probably empty because the main hand is a two hander
        if name == "weapon2" {
            return View::empty();
        }
        return view! { cx, div(class=class) {} };
    };

    let src = crate::assets::item_image_url(gv, image_name);

    let mouseover = move |_: web_sys::Event| current_item.set(item);
    let open_wiki = move |event: web_sys::Event| {
        if let Some(item) = item {
            if item.rarity.is_unique() && click_has_ctrl(event) {
                if let Some(name) = item.name {
                    open_wiki_page(name);
                }
            }
        }
    };

    view! { cx,
        img(src=src, class=class, alt=image_name,
            onerror=IMG_ONERROR_EMPTY, loading="lazy",
            on:mouseover=mouseover, on:click=open_wiki) {}
    }
}

fn item_image_name<'a>(item: &pob::Item<'a>) -> &'a str {
    if item.rarity.is_unique() {
        if let Some(name) = item.fixed_item_name() {
            return name;
        }
    }

    if item.base.starts_with("Two-Toned") {
        // Bases have at least 126 base value, you can't craft that much.
        match (item.armour >= 126, item.evasion >= 126) {
            (true, true) => "TwoTonedArEv",
            (false, true) => "TwoTonedEvEs",
            _ => "TwoTonedArEs",
        }
    } else if item.base.starts_with("Two-Stone") {
        let mut fire = false;
        let mut cold = false;
        for implicit in item.implicits() {
            fire = fire || implicit.line.contains("Fire");
            cold = cold || implicit.line.contains("Cold");
            if fire || cold {
                break;
            }
        }

        match (fire, cold) {
            (true, true) => "TwoStoneFC",
            (false, true) => "TwoStoneCL",
            _ => "TwoStoneFL",
        }
    } else {
        item.base
    }
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

fn flasks<'a>(gv: GameVersion, gear: &pob::Gear<'a>) -> [(&'static str, Option<&'a str>); 5] {
    match gv {
        GameVersion::One => [
            ("flask1", gear.flask1),
            ("flask2", gear.flask2),
            ("flask3", gear.flask3),
            ("flask4", gear.flask4),
            ("flask5", gear.flask5),
        ],
        GameVersion::Two => [
            ("flask1", gear.flask1),
            ("flask2", gear.charm1),
            ("flask3", gear.charm2),
            ("flask4", gear.charm3),
            ("flask5", gear.flask2),
        ],
    }
}
