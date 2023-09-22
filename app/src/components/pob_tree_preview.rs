use std::cell::RefCell;

use itertools::Itertools;
use pob::TreeSpec;
use shared::model::data;
use sycamore::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlElement, PointerEvent};

use crate::{
    build::Build, components::PobColoredSelect, consts, utils::IteratorExt, Prefetch,
    ResponseContext,
};

#[derive(Debug)]
struct Tree<'build> {
    name: String,
    image_url: String,
    tree_url: String,
    active: bool,
    nodes: &'build data::Nodes,
    overrides: Vec<Override<'build>>,
    allocated: usize,
}

#[derive(Debug)]
struct Override<'build> {
    count: usize,
    name: &'build str,
    effect: &'build str,
}

#[component]
pub fn PobTreePreview<'a, G: Html>(cx: Scope<'a>, build: &'a Build) -> View<G> {
    let trees = build
        .trees()
        .filter_map(|(nodes, spec)| {
            let url = get_tree_url(&spec)?;
            Some(Tree {
                name: spec.title.unwrap_or("<Default>").to_owned(),
                image_url: get_tree_image_url(&spec, &url)?,
                tree_url: url,
                active: spec.active,
                nodes,
                overrides: extract_overrides(spec.overrides),
                allocated: spec.nodes.len(),
            })
        })
        .collect::<Vec<_>>();

    if trees.is_empty() {
        return view! { cx, };
    }

    if !G::IS_BROWSER {
        for tree in trees.iter() {
            if tree.active {
                ResponseContext::preload(Prefetch::Image(tree.image_url.clone()));
            }
        }
    }

    let trees = create_ref(cx, trees);
    let current_tree = create_signal(cx, trees.iter().find_or_first(|t| t.active).unwrap());

    // TODO: this updates the currently active tree, but it doesn't read from it
    // the select would need to be updated as well if the tree changes, kinda tricky...
    let select = render_select(cx, trees, move |index, tree| {
        current_tree.set(tree);
        build.active_tree().set(index);
    });

    let node_ref = create_node_ref(cx);
    let state = create_ref(cx, RefCell::new(TouchState::new(node_ref.clone())));
    let on_move_start = |event: Event| {
        let event = event.unchecked_into::<PointerEvent>();
        state.borrow_mut().add_pointer(&event);
    };
    let on_move_move = |event: Event| {
        let event = event.unchecked_into::<PointerEvent>();
        let mut state = state.borrow_mut();

        let pointer_state = match state.get_pointer(&event) {
            Some(pointer_state) => pointer_state,
            None => return,
        };

        if state.size() == 1 {
            let dx = event.client_x() - pointer_state.x;
            let dy = event.client_y() - pointer_state.y;
            state.move_canvas(dx, dy);
            state.apply();
        } else if state.size() == 2 {
            let other = state
                .pointers()
                .find(|p| p.id != event.pointer_id())
                .unwrap();

            let distance = f32::hypot(
                other.x as f32 - event.client_x() as f32,
                other.y as f32 - event.client_y() as f32,
            );

            state.zoom_pinch(distance);
            state.apply();
        }

        state.update_pointer(&event);
    };
    let on_move_end = |event: Event| {
        let event = event.unchecked_into::<PointerEvent>();
        state.borrow_mut().remove_pointer(&event);
    };

    let nodes = create_memo(cx, move || render_nodes(cx, &current_tree.get()));
    let tree_background = create_memo(cx, || {
        format!("background-image: url({})", current_tree.get().image_url)
    });
    let tree_level = create_memo(cx, move || {
        let current_tree = current_tree.get();
        let (nodes, level) = resolve_level(current_tree.allocated);
        let desc = format!("Level {level} ({nodes} passives)");
        view! { cx,
            a(href=current_tree.tree_url, rel="external", target="_blank",
            class="text-sky-500 dark:text-sky-400 hover:underline") {
                (desc)
            }
        }
    });

    view! { cx,
        div(class="flex flex-wrap align-center") {
            div(class="h-9 max-w-full") { (select) }
            div(class="flex-1 text-right sm:mr-3 whitespace-nowrap") { (&*tree_level.get()) }
        }
        div(class="grid grid-cols-10 gap-3") {
            div(class="col-span-10 lg:col-span-7 h-[450px] md:h-[800px] cursor-move md:overflow-auto mt-2",
                on:pointerdown=on_move_start,
                on:pointermove=on_move_move,
                on:pointerup=on_move_end,
                on:pointerleave=on_move_end,
                on:pointercancel=on_move_end,
            ) {
                div(ref=node_ref,
                    class="h-full w-full bg-center bg-no-repeat touch-pan
                    transition-[background-image] duration-1000 will-change-[background-image]",
                    style=tree_background.get()) {
                }
            }

            div(class="col-span-10 lg:col-span-3 flex flex-col gap-3 h-full relative") {
                div(class="flex flex-col gap-3 md:gap-6 h-full w-full lg:absolute overflow-y-auto") {
                    (*nodes.get())
                }
            }
        }
    }
}

fn resolve_level(allocated: usize) -> (usize, usize) {
    // TODO: needs auto-generated node information for ascendancies
    if allocated == 0 {
        return (0, 0);
    }

    // character start node
    let allocated = allocated - 1;

    // points count towards allocated but aren't available skill tree points
    let asc = match allocated {
        0..=38 => 0,
        39..=69 => 3, // 2 points + ascendancy start node
        70..=90 => 5,
        91..=98 => 7,
        _ => 9,
    };

    // TODO: check for bandits
    let bandits = match allocated {
        0..=21 => 0,
        _ => 2,
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

    (allocated - asc, 1 + allocated - asc - bandits - quests)
}

fn extract_overrides(mut overrides: Vec<pob::Override<'_>>) -> Vec<Override<'_>> {
    overrides.sort_unstable_by_key(|k| (k.name, k.effect));

    overrides
        .into_iter()
        .dedup_by_with_count(|a, b| (a.name, a.effect) == (b.name, b.effect))
        .map(|(count, o)| Override {
            count,
            name: o.name,
            effect: o.effect,
        })
        .collect()
}

fn render_nodes<G: GenericNode + Html>(cx: Scope, tree: &Tree<'_>) -> View<G> {
    let nodes = tree.nodes;

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
        .map(|node| render_keystone(cx, node))
        .collect_view();

    let masteries = nodes
        .masteries
        .iter()
        .map(|node| render_mastery(cx, node))
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

    view! { cx,
        div(class="bg-slate-900 rounded-xl px-4 py-3") {
            div(class="mb-2 text-stone-200 text-sm md:text-base") {
                span() { (name) }
                span(class="text-xs ml-1") { (count) }
            }
            ul(class="flex flex-col gap-2 pb-1 whitespace-pre-line text-xs md:text-sm text-slate-400") { (effect) }
        }
    }
}

fn render_keystone<G: GenericNode + Html>(cx: Scope, node: &data::Node) -> View<G> {
    let name = node.name.to_owned();
    let alt = name.clone();
    let stats = node.stats.iter().join("\n");

    let src = node
        .icon
        .as_deref()
        .map(crate::assets::item_image_url)
        .unwrap_or_default();

    view! { cx,
        div(class="bg-slate-900 rounded-xl px-4 py-3", title=stats) {
            div(class="text-stone-200 text-sm md:text-base flex items-center gap-2") {
                img(class="rounded-xl w-7", src=src, alt=alt, onerror=consts::IMG_ONERROR_HIDDEN, loading="lazy") {}
                span() { (name) }
            }
        }
    }
}

fn render_mastery<G: GenericNode + Html>(cx: Scope, node: &data::Node) -> View<G> {
    let name = node.name.to_owned();
    let alt = name.clone();
    let stats = node
        .stats
        .iter()
        .map(|stat| {
            let stat = stat.clone();
            view! { cx, li(class="leading-tight") { (stat) } }
        })
        .collect_view();

    let src = node
        .icon
        .as_deref()
        .map(crate::assets::item_image_url)
        .unwrap_or_default();

    view! { cx,
        div(class="bg-slate-900 rounded-xl px-4 py-3") {
            div(class="mb-2 text-stone-200 text-sm md:text-base flex items-center gap-2") {
                img(class="rounded-xl w-7", src=src, alt=alt, onerror=consts::IMG_ONERROR_HIDDEN, loading="lazy") {}
                span() { (name) }
            }
            ul(class="flex flex-col gap-2 pb-1 whitespace-pre-line text-xs md:text-sm text-slate-400") { (stats) }
        }
    }
}

fn render_select<'a, G: GenericNode + Html, F>(
    cx: Scope<'a>,
    trees: &'a Vec<Tree>,
    on_change: F,
) -> View<G>
where
    F: Fn(usize, &'a Tree) + 'a,
{
    if trees.len() <= 1 {
        return view! { cx, };
    }

    let options = trees.iter().map(|t| t.name.clone()).collect();
    let selected = trees.iter().position(|t| t.active);
    let on_change = move |index| {
        if let Some(index) = index {
            on_change(index, &trees[index])
        }
    };

    view! { cx,
        PobColoredSelect(options=options, selected=selected, on_change=on_change)
    }
}

fn get_tree_image_url(spec: &TreeSpec, url: &str) -> Option<String> {
    url.rsplit_once('/')
        .map(|(_, data)| data)
        .zip(spec.version)
        .map(|(data, ver)| format!("https://tree.pobb.in/{ver}/{data}"))
}

fn get_tree_url(spec: &TreeSpec) -> Option<String> {
    spec.url
        .filter(|url| {
            url.starts_with("https://pathofexile.com")
                || url.starts_with("https://www.pathofexile.com")
        })
        .map(|url| url.to_owned())
}

#[derive(Debug)]
struct PointerState {
    id: i32,
    x: i32,
    y: i32,
}

#[derive(Debug)]
struct TouchState<G: GenericNode> {
    node: NodeRef<G>,

    // pointer state
    pointers: Vec<PointerState>,
    distance: Option<f32>,

    // canvas state
    center_x: i32,
    center_y: i32,
    zoom: Option<f32>,
}

impl<G: GenericNode> TouchState<G> {
    fn new(node: NodeRef<G>) -> Self {
        Self {
            node,
            pointers: Vec::with_capacity(3),
            distance: None,
            center_x: 0,
            center_y: 0,
            zoom: None,
        }
    }

    fn size(&self) -> usize {
        self.pointers.len()
    }

    fn pointers(&self) -> impl Iterator<Item = &PointerState> {
        self.pointers.iter()
    }

    fn get_pointer(&self, event: &PointerEvent) -> Option<&PointerState> {
        self.pointers.iter().find(|p| p.id == event.pointer_id())
    }

    fn add_pointer(&mut self, event: &PointerEvent) {
        let pointer = PointerState {
            id: event.pointer_id(),
            x: event.client_x(),
            y: event.client_y(),
        };
        self.pointers.push(pointer);
    }

    fn update_pointer(&mut self, event: &PointerEvent) {
        if let Some(p) = self
            .pointers
            .iter_mut()
            .find(|p| p.id == event.pointer_id())
        {
            p.x = event.client_x();
            p.y = event.client_y();
        }
    }

    fn remove_pointer(&mut self, event: &PointerEvent) {
        self.pointers.retain(|p| p.id != event.pointer_id());
        self.distance = None;
    }

    fn move_canvas(&mut self, dx: i32, dy: i32) {
        self.center_x += dx;
        self.center_y += dy;
    }

    fn zoom_pinch(&mut self, new_distance: f32) {
        if self.zoom.is_none() {
            // Zoomed at least once, now enable drag of the tree
            // instead of scrolling the page (pan-y).
            let _ = crate::utils::from_ref::<HtmlElement>(&self.node)
                .style()
                .set_property("touch-action", "none");
        }

        if let Some(distance) = self.distance {
            let zoom = self.zoom.unwrap_or_else(|| {
                let element = crate::utils::from_ref::<HtmlElement>(&self.node);
                get_background_size(&element)
            });

            let pinch = distance - new_distance;
            let new_zoom = (zoom - pinch / 2.0).clamp(100.0, 300.0);

            self.center_x -= (self.center_x as f32 * (1.0 - new_zoom / zoom)) as i32;
            self.center_y -= (self.center_y as f32 * (1.0 - new_zoom / zoom)) as i32;

            self.zoom = Some(new_zoom);
        }

        self.distance = Some(new_distance);
    }

    fn apply(&self) {
        let element = &crate::utils::from_ref::<HtmlElement>(&self.node);
        let position = format!(
            "calc(50% + {}px) calc(50% + {}px)",
            self.center_x, self.center_y
        );
        let _ = element
            .style()
            .set_property("background-position", &position);
        if let Some(zoom) = self.zoom {
            let _ = element
                .style()
                .set_property("background-size", &format!("{zoom}%"));
        }
    }
}

fn get_background_size(element: &HtmlElement) -> f32 {
    let bg_size = web_sys::window()
        .unwrap()
        .get_computed_style(element)
        .unwrap()
        .unwrap()
        .get_property_value("background-size")
        .ok()
        .unwrap_or_default();

    if bg_size.is_empty() {
        return 100.0;
    }

    bg_size.replace('%', "").parse().unwrap_or(100.0)
}
