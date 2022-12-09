use std::{any::TypeId, cell::RefCell, rc::Rc};

use itertools::Itertools;
use pob::TreeSpec;
use shared::model::{Node, Nodes};
use sycamore::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlElement, PointerEvent};

use crate::build::Build;
use crate::components::{PobColoredSelect, PobColoredSelectProps};
use crate::{memo, Prefetch, ResponseContext};

#[derive(Debug)]
struct Tree {
    name: String,
    image_url: String,
    active: bool,
    nodes: Rc<Nodes>,
}

#[component(PobTreePreview<G>)]
pub fn pob_tree_preview(build: Build) -> View<G> {
    let trees = build
        .trees()
        .map(|(nodes, spec)| Tree {
            name: spec.title.unwrap_or("<Default>").to_owned(),
            image_url: get_tree_url(&spec).unwrap(),
            active: spec.active,
            nodes: Rc::new(nodes.clone()),
        })
        .collect::<Vec<_>>();

    if trees.is_empty() {
        return view! {};
    }

    if TypeId::of::<G>() == TypeId::of::<SsrNode>() {
        for tree in trees.iter() {
            if tree.active {
                ResponseContext::preload(Prefetch::Image(tree.image_url.clone()));
            }
        }
    }

    let current_tree = trees.iter().find_or_first(|t| t.active).unwrap();

    let value = current_tree.image_url.clone();
    let style = format!("background-image: url({value})");

    let node_ref = NodeRef::new();
    let nodes = Signal::new(Rc::clone(&current_tree.nodes));

    let select = render_select(
        trees,
        cloned!(node_ref, nodes => move |tree| {
            let property = format!("url({})", tree.image_url);
            let _ = crate::utils::from_ref::<_, HtmlElement>(&node_ref)
                .style()
                .set_property("background-image", &property);
            nodes.set(Rc::clone(&tree.nodes));
        }),
    );

    let state = Rc::new(RefCell::new(TouchState::new(node_ref.clone())));
    let on_move_start = cloned!(state => move |event: Event| {
        let event = event.unchecked_into::<PointerEvent>();
        state.borrow_mut().add_pointer(&event);
    });
    let on_move_move = cloned!(state => move |event: Event| {
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
            let other = state.pointers().find(|p| p.id != event.pointer_id()).unwrap();

            let distance = f32::hypot(
                other.x as f32 - event.client_x() as f32,
                other.y as f32 - event.client_y() as f32
            );

            state.zoom_pinch(distance);
            state.apply();
        }

        state.update_pointer(&event);
    });
    let on_move_end = cloned!(state => move |event: Event| {
        let event = event.unchecked_into::<PointerEvent>();
        state.borrow_mut().remove_pointer(&event);
    });

    let nodes = memo!(nodes, render_nodes(&nodes.get()));

    view! {
        (select)
        div(class="grid grid-cols-10 gap-3") {
            div(class="col-span-10 lg:col-span-7 h-[450px] md:h-[800px] cursor-move md:overflow-auto mt-2",
                on:pointerdown=on_move_start,
                on:pointermove=on_move_move,
                on:pointerup=on_move_end.clone(),
                on:pointerleave=on_move_end.clone(),
                on:pointercancel=on_move_end,
            ) {
                div(ref=node_ref,
                    class="h-full w-full bg-center bg-no-repeat touch-none
                    transition-[background-image] duration-1000 will-change-[background-image]",
                    style=style) {
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

pub fn render_nodes<G: GenericNode + Html>(nodes: &Nodes) -> View<G> {
    if nodes.is_empty() {
        return view! {
            div(class="text-stone-200 hidden lg:block text-center") {
                "No Keystones and Masteries"
            }
        };
    }

    let keystones = nodes
        .keystones
        .iter()
        .map(|node| render_keystone(node))
        .collect();
    let keystones = View::new_fragment(keystones);

    let masteries = nodes
        .masteries
        .iter()
        .map(|node| render_mastery(node))
        .collect();
    let masteries = View::new_fragment(masteries);

    view! {
        div(class="grid grid-cols-fit-keystone gap-2 lg:gap-1") { (keystones) }
        div(class="grid grid-cols-fit-mastery gap-2 lg:gap-1") { (masteries) }
    }
}

fn render_keystone<G: GenericNode + Html>(node: &Node) -> View<G> {
    let name = node.name.to_owned();
    let stats = node.stats.iter().join("\n");

    view! {
        div(class="bg-slate-900 rounded-xl px-4 py-3", title=stats) {
            div(class="text-stone-200 text-sm md:text-base") { (name) }
        }
    }
}

fn render_mastery<G: GenericNode + Html>(node: &Node) -> View<G> {
    let name = node.name.to_owned();
    let stats = node
        .stats
        .iter()
        .map(|stat| {
            let stat = stat.clone();
            view! { li(class="leading-tight") { (stat) } }
        })
        .collect();
    let stats = View::new_fragment(stats);

    view! {
        div(class="bg-slate-900 rounded-xl px-4 py-3") {
            div(class="mb-2 text-stone-200 text-sm md:text-base") { (name) }
            ul(class="flex flex-col gap-2 pb-1 whitespace-pre-line text-xs md:text-sm text-slate-400") { (stats) }
        }
    }
}

fn render_select<G: GenericNode + Html, F>(trees: Vec<Tree>, on_change: F) -> View<G>
where
    F: Fn(&Tree) + 'static,
{
    if trees.len() <= 1 {
        return view! {};
    }

    let options = trees.iter().map(|t| t.name.clone()).collect();
    let selected = trees.iter().position(|t| t.active);

    view! {
        PobColoredSelect(PobColoredSelectProps {
            options,
            selected,
            on_change: move |index| if let Some(index) = index { on_change(&trees[index]) },
        })
    }
}

fn get_tree_url(spec: &TreeSpec) -> Option<String> {
    spec.url
        .and_then(|url| url.rsplit_once('/'))
        .map(|(_, data)| data)
        .zip(spec.version)
        .map(|(data, ver)| format!("https://tree.pobb.in/{ver}/{data}"))
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
        if let Some(distance) = self.distance {
            let zoom = self.zoom.unwrap_or_else(|| {
                let element = &crate::utils::from_ref::<_, HtmlElement>(&self.node);
                get_background_size(element)
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
        let element = &crate::utils::from_ref::<_, HtmlElement>(&self.node);
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
        .unwrap();

    if bg_size.is_empty() {
        return 100.0;
    }

    bg_size.replace('%', "").parse().unwrap()
}
