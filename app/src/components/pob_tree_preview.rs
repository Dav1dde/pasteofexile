use crate::components::PobColoredText;
use crate::{Prefetch, ResponseContext};
use itertools::Itertools;
use pob::{PathOfBuilding, SerdePathOfBuilding, TreeSpec};
use std::{any::TypeId, cell::RefCell, rc::Rc};
use sycamore::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlElement, HtmlSelectElement, PointerEvent};

struct Tree {
    name: String,
    image_url: String,
    active: bool,
}

#[component(PobTreePreview<G>)]
pub fn pob_tree_preview(pob: Rc<SerdePathOfBuilding>) -> View<G> {
    let trees = pob
        .tree_specs()
        .into_iter()
        .map(|spec| Tree {
            name: spec.title.unwrap_or("<Default>").to_owned(),
            image_url: get_tree_url(&spec).unwrap(),
            active: spec.active,
        })
        .collect::<Vec<_>>();

    if trees.is_empty() {
        return view! {};
    }

    if TypeId::of::<G>() == TypeId::of::<SsrNode>() {
        for (i, tree) in trees.iter().enumerate() {
            if i == 0 {
                ResponseContext::preload(Prefetch::Image(tree.image_url.clone()));
            } else {
                ResponseContext::prefetch(Prefetch::Image(tree.image_url.clone()));
            }
        }
    }

    let value = trees
        .iter()
        .find_or_first(|t| t.active)
        .unwrap()
        .image_url
        .clone();
    let style = format!("background-image: url({value})");
    let node_ref = NodeRef::new();
    let select = render_select(
        trees,
        cloned!(node_ref => move |value| {
            let property = format!("url({})", value);
            let _ = crate::utils::from_ref::<_, HtmlElement>(&node_ref)
                .style()
                .set_property("background-image", &property);
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

    view! {
        (select)
        div(class="h-[370px] md:h-[700px] cursor-move md:resize-y md:overflow-auto mt-2",
            on:pointerdown=on_move_start,
            on:pointermove=on_move_move,
            on:pointerup=on_move_end.clone(),
            on:pointerleave=on_move_end.clone(),
            on:pointercancel=on_move_end,
        ) {
            div(ref=node_ref,
                class="h-full w-full bg-center bg-no-repeat bg-[length:180%] md:bg-[length:80%] touch-none
                    transition-[background-image] duration-1000 will-change-[background-image]",
                style=style) {
            }
        }
    }
}

fn render_select<G: GenericNode + Html, F>(trees: Vec<Tree>, on_change: F) -> View<G>
where
    F: Fn(String) + 'static,
{
    if trees.len() <= 1 {
        return view! {};
    }

    let select = trees.into_iter()
        .map(|t| {
            view! { option(value=t.image_url, selected=t.active) { span { PobColoredText(t.name) } } }
        })
        .collect::<Vec<_>>();
    let select = View::new_fragment(select);

    let on_input = move |event: web_sys::Event| {
        let event = event.unchecked_into::<web_sys::InputEvent>();
        let element = event
            .target()
            .unwrap()
            .unchecked_into::<HtmlSelectElement>();
        on_change(element.value());
    };

    view! {
        select(class="sm:ml-3 mt-1 px-1", on:input=on_input) { (select) }
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
            let new_zoom = (zoom - pinch / 2.0).max(100.0).min(300.0);

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
                .set_property("background-size", &format!("{}%", zoom));
        }
    }
}

fn get_background_size(element: &HtmlElement) -> f32 {
    web_sys::window()
        .unwrap()
        .get_computed_style(element)
        .unwrap()
        .unwrap()
        .get_property_value("background-size")
        .unwrap()
        .replace('%', "")
        .parse()
        .unwrap()
}
