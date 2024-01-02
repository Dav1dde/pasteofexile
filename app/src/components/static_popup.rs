use sycamore::prelude::*;
use wasm_bindgen::JsCast;

use crate::components::Popup;

#[derive(Prop)]
pub struct StaticPopupProps<'a, G: Html> {
    children: Children<'a, G>,
    content: View<G>,
}

#[component]
pub fn StaticPopup<'a, G: Html>(cx: Scope<'a>, props: StaticPopupProps<'a, G>) -> View<G> {
    let node_ref = create_node_ref(cx);
    let attach = create_signal(cx, None);

    let mouseover = move |event: web_sys::Event| {
        let target = event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::Element>().ok());
        attach.set(target);
    };
    let mouseout = |_: web_sys::Event| attach.set(None);

    let children = props.children.call(cx);
    view! {
        cx,
        div(
            ref=node_ref,
            on:mouseover=mouseover,
            on:mouseout=mouseout,
            class="inline-block",
        ) { (children) }
        Popup(attach=attach) { (props.content) }
    }
}
