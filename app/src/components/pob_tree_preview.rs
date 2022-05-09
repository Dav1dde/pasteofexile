use crate::components::PobColoredText;
use crate::{Prefetch, ResponseContext};
use pob::{PathOfBuilding, SerdePathOfBuilding, TreeSpec};
use std::{any::TypeId, cell::Cell, rc::Rc};
use sycamore::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlElement, MouseEvent, TouchEvent};

struct Tree {
    name: String,
    image_url: String,
}

#[component(PobTreePreview<G>)]
pub fn pob_tree_preview(pob: Rc<SerdePathOfBuilding>) -> View<G> {
    let trees = pob
        .tree_specs()
        .into_iter()
        .map(|spec| {
            let name = spec.title.unwrap_or("<Default>").to_owned();
            let image_url = get_tree_url(&spec).unwrap();
            Tree { name, image_url }
        })
        .collect::<Vec<_>>();

    if trees.is_empty() {
        return view! {};
    }

    if TypeId::of::<G>() == TypeId::of::<SsrNode>() {
        for tree in &trees {
            ResponseContext::preload(Prefetch::Image(tree.image_url.clone()));
        }
    }

    let value = trees.get(0).unwrap().image_url.clone();
    let style = format!("background-image: url({value})");
    let value = Signal::new(value);

    let select = trees.into_iter()
        .enumerate()
        .map(|(i, t)| {
            let selected = i == 0;
            view! { option(value=t.image_url, selected=selected) { span { PobColoredText(t.name) } } }
        })
        .collect::<Vec<_>>();
    let select = View::new_fragment(select);

    let node_ref = NodeRef::new();

    let current_offset = Rc::new(Cell::new((0, 0)));
    let move_start = Rc::new(Cell::new(None));
    let last_pinch = Rc::new(Cell::new(None));

    let on_move_start = cloned!(move_start => move |event| {
        move_start.set(get_event_pos(&event));
    });
    let on_move_move = cloned!(move_start, current_offset, last_pinch, node_ref => move |event| {
        let (start_x, start_y) = match move_start.get() {
            Some(v) => v,
            None => return,
        };

        if let Some(pinch) = get_pinch(&event) {
            if let Some(lp) = last_pinch.get() {
                let element = crate::utils::from_ref::<_, HtmlElement>(&node_ref);
                let style = web_sys::window().unwrap().get_computed_style(&element).unwrap().unwrap();

                let zoom = style
                    .get_property_value("background-size")
                    .unwrap()
                    .replace('%', "");

                let zoom_delta = (pinch - lp) / 2.0;
                let zoom = zoom.parse::<f32>().unwrap();
                let zoom: f32 = zoom + zoom_delta;
                let zoom = zoom.max(100.0).min(300.0);

                let _ = element.style().set_property("background-size", &format!("{zoom}%"));
            }

            last_pinch.set(Some(pinch));
            event.prevent_default();
            return;
        } else if last_pinch.get().is_some() {
            // we pinched ignore everythign after
            return;
        }

        let (x, y) = match get_event_pos(&event) {
            Some(v) => v,
            None => return,
        };

        let (current_offset_x, current_offset_y) = current_offset.get();

        // TODO: instead of remembering the start, remember the last event
        let diff_x = current_offset_x + x - start_x;
        let diff_y = current_offset_y + y - start_y;

        let offset = format!("calc(50% + {diff_x}px) calc(50% + {diff_y}px)");
        let _ = crate::utils::from_ref::<_, HtmlElement>(&node_ref)
            .style()
            .set_property("background-position", &offset);

        event.prevent_default();
    });
    let on_move_end = cloned!(move_start, last_pinch, current_offset => move |event| {
        let (start_x, start_y) = match move_start.get() {
            Some(v) => v,
            None => return,
        };

        move_start.set(None);
        let last_pinch = last_pinch.replace(None);

        // event turned into pinch -> dont update anything else because it becomes weird
        if last_pinch.is_some() {
            return;
        }

        let (x, y) = match get_event_pos(&event) {
            Some(v) => v,
            None => return,
        };

        let (current_offset_x, current_offset_y) = current_offset.get();

        let diff_x = current_offset_x + x - start_x;
        let diff_y = current_offset_y + y - start_y;

        current_offset.set((diff_x, diff_y));
    });

    let on_input = cloned!(value, node_ref => move |_| {
        let property = format!("url({})", value.get());
        let _ = crate::utils::from_ref::<_, HtmlElement>(&node_ref)
            .style()
            .set_property("background-image", &property);
    });

    view! {
        select(class="sm:ml-3 mt-1 mb-2", bind:value=value, on:input=on_input) { (select) }
        div(class="h-[370px] md:h-[500px] cursor-move md:resize-y md:overflow-auto",
            on:mousedown=on_move_start.clone(),
            on:mousemove=on_move_move.clone(),
            on:mouseup=on_move_end.clone(),
            on:mouseleave=on_move_end.clone(),
            on:touchstart=on_move_start,
            on:touchmove=on_move_move,
            on:touchend=on_move_end.clone(),
            on:touchcancel=on_move_end,
        ) {
            div(ref=node_ref,
                class="h-full w-full bg-center bg-no-repeat bg-[length:180%] md:bg-[length:80%]",
                style=style) {
            }
        }
    }
}

fn get_tree_url(spec: &TreeSpec) -> Option<String> {
    spec.url
        .and_then(|url| url.rsplit_once('/'))
        .and_then(|(_, data)| spec.version.map(|ver| (data, ver)))
        .map(|(data, ver)| format!("https://tree.pobb.in/{}/{}", ver.replace('_', "."), data))
}

fn get_event_pos(event: &Event) -> Option<(i32, i32)> {
    let mouse = event
        .dyn_ref::<MouseEvent>()
        .map(|event| (event.client_x(), event.client_y()));

    let touch = event
        .dyn_ref::<TouchEvent>()
        .filter(|event| event.changed_touches().length() == 1)
        .and_then(|event| event.changed_touches().get(0))
        .map(|touch| (touch.client_x(), touch.client_y()));

    mouse.or(touch)
}

fn get_pinch(event: &Event) -> Option<f32> {
    event
        .dyn_ref::<TouchEvent>()
        .filter(|event| event.touches().length() == 2)
        .map(|event| {
            let a = event.touches().get(0).unwrap();
            let b = event.touches().get(1).unwrap();

            (a.client_x() as f32 - b.client_x() as f32)
                .hypot(a.client_y() as f32 - b.client_y() as f32)
        })
}
