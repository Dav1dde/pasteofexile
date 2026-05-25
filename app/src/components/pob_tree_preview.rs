use itertools::Itertools;
use pob::{PathOfBuilding, Socket, TreeSpec};
use shared::{model::data, GameVersion};
use sycamore::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlElement};

use crate::{
    build::Build,
    components::{PobColoredSelect, PobItem, Popup, StaticPopup, TreeNode},
    consts,
    tree::SvgTree,
    utils::{hooks::scoped_event_passive, IteratorExt},
};

#[derive(Debug)]
struct Tree<'build> {
    tree_url: String,
    svg_url: &'static str,
    spec: TreeSpec<'build>,
    nodes: &'build data::Nodes,
    overrides: Vec<Override<'build>>,
}

impl Tree<'_> {
    fn socket(&self, id: u32) -> Option<&Socket> {
        self.spec.sockets.iter().find(|socket| socket.node_id == id)
    }

    fn mastery(&self, id: u32) -> Option<&str> {
        for mastery in &self.nodes.masteries {
            for stat in &mastery.stats {
                if stat.id == id {
                    return Some(&stat.text);
                }
            }
        }

        None
    }
}

#[derive(Debug)]
struct Override<'build> {
    count: usize,
    name: &'build str,
    effect: &'build str,
    node_id: u32,
}

#[component]
pub fn PobTreePreview<'a, G: Html>(cx: Scope<'a>, build: &'a Build) -> View<G> {
    let gv = build.game_version();

    let current_tree = create_memo(cx, move || {
        build.active_tree().and_then(|t| {
            let tree_url = get_tree_url(&t.spec)?;
            let svg_url = SvgTree::url(gv, &t.spec);
            let overrides = extract_overrides(&t.spec.overrides);

            Some(Tree {
                tree_url,
                svg_url,
                spec: t.spec,
                nodes: t.nodes,
                overrides,
            })
        })
    });

    let tree_loaded = create_signal(cx, false);
    let node_ref = create_node_ref(cx);

    let current_svg = create_signal(cx, "");
    create_effect(cx, || {
        let new_svg = (*current_tree.get())
            .as_ref()
            .map(|t| t.svg_url)
            .unwrap_or("");

        // Debounce the svg and reset loading state when it changed.
        if new_svg != *current_svg.get() {
            tree_loaded.set(false);
            current_svg.set(new_svg);
        }
    });

    let events = std::sync::Once::new();
    let attach = create_signal(cx, None);
    let popup = create_signal(cx, view! { cx, });
    let on_mouseover_tree = move |event: web_sys::Event| {
        let target: HtmlElement = event.target().unwrap().unchecked_into();

        let Some(id) = target.id().strip_prefix('n').and_then(|s| s.parse().ok()) else {
            attach.set(None);
            return;
        };

        let current_tree = current_tree.get();
        let Some(current_tree) = current_tree.as_ref() else {
            return;
        };

        let item = current_tree
            .socket(id)
            .and_then(|socket| build.item_by_id(socket.item_id))
            .and_then(|item| pob::Item::parse(item).ok());

        let content = if let Some(item) = item {
            view! { cx, PobItem(game_version=gv, item=item) }
        } else {
            let dataset = target.dataset();

            let kind = dataset.get("kind");
            let name = dataset.get("name").unwrap_or_default();

            let stats = if let Some(mastery) = current_tree.mastery(id) {
                vec![mastery.to_owned()]
            } else {
                dataset
                    .get("stats")
                    .map(|s| s.split(";;").map(Into::into).collect())
                    .unwrap_or_default()
            };

            view! { cx, TreeNode(kind=kind, name=name, stats=stats) }
        };

        popup.set(content);
        attach.set(Some(target.unchecked_into()));
    };

    create_effect(cx, move || {
        let current_tree = current_tree.get();
        let Some(current_tree) = current_tree.as_ref() else {
            return;
        };

        if !*tree_loaded.get() {
            return;
        }

        let s = SvgTree::from_ref(node_ref).unwrap();
        s.load(&current_tree.spec);

        events.call_once(|| {
            scoped_event_passive(cx, s.element(), "mouseover", on_mouseover_tree);
        });
    });

    let select = render_select(cx, build);

    let nodes = create_memo(cx, move || {
        render_nodes(cx, gv, (*current_tree.get()).as_ref())
    });
    let tree_level = create_memo(cx, move || {
        if gv.is_poe2() {
            return View::empty();
        }
        let current_tree = current_tree.get();
        let Some(current_tree) = current_tree.as_ref() else {
            return View::empty();
        };
        let (nodes, level) = resolve_level(build, &current_tree.spec);
        let tree_url = current_tree.tree_url.clone();
        let desc = format!("Level {level} ({nodes} passives)");
        view! { cx,
            a(href=tree_url, rel="external", target="_blank",
            class="text-sky-500 dark:text-sky-400 hover:underline") {
                (desc)
            }
        }
    });

    let on_mouseover_side = |event: Event| {
        let target: HtmlElement = event.target().unwrap().unchecked_into();
        let Some(node_id) = target.dataset().get("nodeId") else {
            return;
        };

        if let Some(tree) = SvgTree::from_ref(node_ref) {
            tree.highlight(node_id.split(','));
        }
    };
    let on_mouseout_side = |event: Event| {
        let target: HtmlElement = event.target().unwrap().unchecked_into();
        if target.dataset().get("nodeId").is_none() {
            return;
        };

        if let Some(tree) = SvgTree::from_ref(node_ref) {
            tree.clear_highlight();
        }
    };

    view! { cx,
        Popup(attach=attach, parent=Some(node_ref)) { (&*popup.get()) }
        div(class="flex flex-wrap align-center") {
            div(class="h-9 max-w-full") { (select) }
            div(class="flex-1 text-right sm:mr-3 whitespace-nowrap") { (&*tree_level.get()) }
        }
        div(class="grid grid-cols-10 gap-3") {
            div(class="col-span-10 lg:col-span-7 h-[450px] md:h-[800px] cursor-move md:overflow-auto mt-2") {
                object(
                    ref=node_ref,
                    data=current_svg.get(),
                    class="h-full w-full bg-center bg-no-repeat touch-pan
                    transition-[background-image] duration-1000 will-change-[background-image]",
                    type="image/svg+xml",
                    title="Skilltree Preview",
                    on:load=|_: Event| { tree_loaded.set(true); },
                    on:mouseout=|_: Event| { attach.set(None) }
                ) {}
            }

            div(class="col-span-10 lg:col-span-3 flex flex-col gap-3 h-full") {
                div(
                    class="flex flex-col gap-3 md:gap-6 h-full w-full overflow-y-auto cursor-default",
                    on:mouseover=on_mouseover_side,
                    on:mouseout=on_mouseout_side,
                ) {
                    (*nodes.get())
                }
            }
        }
    }
}

fn resolve_level(build: &Build, tree: &TreeSpec) -> (usize, usize) {
    let allocated = tree.nodes.len();

    // TODO: needs auto-generated node information for ascendancies
    if allocated == 0 {
        return (0, 0);
    }

    // character start node
    let allocated = allocated - 1;

    let resolve_asc = || {
        match allocated {
            0..=38 => 0,
            39..=69 => 3, // 2 points + ascendancy start node
            70..=90 => 5,
            91..=98 => 7,
            _ => 9,
        }
    };

    // Points count towards allocated but aren't available skill tree points.
    let asc = tree.ascendancy_id.map(|_| resolve_asc()).unwrap_or(0);
    let asc2 = tree
        .alternate_ascendancy_id
        .map(|_| resolve_asc())
        .unwrap_or(0);

    let bandits = match build.bandit() {
        Some(_) => 0,
        None => match allocated {
            0..=21 => 0,
            _ => 2,
        },
    };

    let quests = match allocated - asc - bandits {
        0..=11 => 0,
        12..=23 => 2,
        24..=34 => 3,
        35..=44 => 5,
        45..=49 => 6,
        50..=57 => 8,
        58..=64 => 11,
        65..=73 => 14,
        74..=80 => 17,
        81..=85 => 19,
        _ => 22,
    };

    (
        allocated - asc - asc2,
        1 + allocated - asc - asc2 - bandits - quests,
    )
}

fn extract_overrides<'a>(overrides: &[pob::Override<'a>]) -> Vec<Override<'a>> {
    overrides
        .iter()
        .sorted_unstable_by_key(|k| (k.name, k.effect))
        .dedup_by_with_count(|a, b| (a.name, a.effect) == (b.name, b.effect))
        .map(|(count, o)| Override {
            count,
            name: o.name,
            effect: o.effect,
            node_id: o.node_id,
        })
        .collect()
}

fn render_nodes<G: GenericNode + Html>(
    cx: Scope,
    gv: GameVersion,
    tree: Option<&Tree<'_>>,
) -> View<G> {
    let Some(tree) = tree else {
        return View::empty();
    };

    let nodes = tree.nodes;

    if gv.is_poe2() {
        return view! { cx,
            div(class="text-stone-200 hidden lg:block text-center") {
                "No Keystones and Mastery data for PoE 2 yet"
            }
        };
    }

    if nodes.is_empty() {
        return view! { cx,
            div(class="text-stone-200 hidden lg:block text-center") {
                "No Keystones and Masteries"
            }
        };
    }

    let overrides = tree
        .overrides
        .iter()
        .map(|o| render_override(cx, o))
        .collect_view();

    let keystones = nodes
        .keystones
        .iter()
        .map(|node| render_keystone(cx, gv, node))
        .collect_view();

    let masteries = nodes
        .masteries
        .iter()
        .map(|node| render_mastery(cx, gv, node))
        .collect_view();

    view! { cx,
        div(class="grid grid-cols-fit-mastery gap-2 lg:gap-1 empty:hidden") { (overrides) }
        div(class="grid grid-cols-fit-keystone gap-2 lg:gap-1 empty:hidden") { (keystones) }
        div(class="grid grid-cols-fit-mastery gap-2 lg:gap-1 empty:hidden") { (masteries) }
    }
}

fn render_override<G: GenericNode + Html>(cx: Scope, r#override: &Override) -> View<G> {
    let name = r#override.name.to_owned();
    let effect = r#override.effect.to_owned();
    let count = if r#override.count > 1 {
        format!("(x{})", r#override.count)
    } else {
        String::new()
    };
    let node_id = r#override.node_id;

    view! { cx,
        div(class="bg-slate-900 rounded-xl px-4 py-3", data-node-id=node_id) {
            div(class="mb-2 text-stone-200 text-sm md:text-base pointer-events-none") {
                span() { (name) }
                span(class="text-xs ml-1") { (count) }
            }
            div(class="flex flex-col gap-2 pb-1 whitespace-pre-line pointer-events-none
                text-xs md:text-sm text-slate-400") {
                (effect)
            }
        }
    }
}

fn render_keystone<G: GenericNode + Html>(
    cx: Scope,
    gv: GameVersion,
    node: &data::Node,
) -> View<G> {
    let name = node.name.to_owned();
    let alt = name.clone();
    let stats = node.stats.iter().map(|s| &s.text).join("\n");

    let src = node
        .icon
        .as_deref()
        .map(|s| crate::assets::item_image_url(gv, s))
        .unwrap_or_default();

    let node_ids = node_ids(&node.stats);

    let stats = view! { cx,
        div(class="bg-black/[0.8] font-['FontinSmallCaps'] py-2 px-4 text-sm whitespace-pre-line") {
            (stats)
        }
    };

    view! { cx,
        div(class="bg-slate-900 rounded-xl px-4 py-3") {
            StaticPopup(content=stats) {
                div(class="text-stone-200 text-sm md:text-base flex items-center gap-2", data-node-id=node_ids) {
                    img(class="pointer-events-none rounded-xl w-7 h-7",
                        src=src, alt=alt, onerror=consts::IMG_ONERROR_HIDDEN, loading="lazy") {}
                    span(class="pointer-events-none") { (name) }
                }
            }
        }
    }
}

fn render_mastery<G: GenericNode + Html>(cx: Scope, gv: GameVersion, node: &data::Node) -> View<G> {
    let name = node.name.to_owned();
    let alt = name.clone();
    let stats = node
        .stats
        .iter()
        .map(|stat| {
            let stat = stat.clone();
            view! { cx, li(class="leading-tight", data-node-id=stat.id) { (stat.text) } }
        })
        .collect_view();

    let src = node
        .icon
        .as_deref()
        .map(|s| crate::assets::item_image_url(gv, s))
        .unwrap_or_default();

    let node_ids = node_ids(&node.stats);

    view! { cx,
        div(class="bg-slate-900 rounded-xl px-4 py-3") {
            div(class="mb-2 text-stone-200 text-sm md:text-base flex items-center gap-2", data-node-id=node_ids) {
                img(class="pointer-events-none rounded-xl w-7 h-7",
                    src=src, alt=alt, onerror=consts::IMG_ONERROR_HIDDEN, loading="lazy") {}
                span(class="pointer-events-none") { (name) }
            }
            ul(class="flex flex-col gap-2 pb-1 whitespace-pre-line text-xs md:text-sm text-slate-400") { (stats) }
        }
    }
}

fn render_select<'a, G: GenericNode + Html>(cx: Scope<'a>, build: &'a Build) -> View<G> {
    let trees = build.pob().tree_specs();

    if trees.len() <= 1 {
        return view! { cx, };
    }

    let selected = create_selector(cx, || build.active_tree_id());
    let on_change = move |id| {
        if let Some(id) = id {
            build.set_active_tree(id);
        }
    };

    view! { cx,
        PobColoredSelect(options=trees, selected=selected, label="Select tree", on_change=on_change)
    }
}

fn get_tree_url(spec: &TreeSpec) -> Option<String> {
    spec.url
        .filter(|url| {
            url.starts_with("https://pathofexile.com")
                || url.starts_with("https://www.pathofexile.com")
        })
        .map(|url| url.to_owned())
}

fn node_ids(node: &[data::NodeStat]) -> String {
    node.iter().map(|n| n.id).unique().join(",")
}
