use sycamore::prelude::*;

use crate::utils::try_from_ref;

#[derive(Prop)]
pub struct PopupProps<'a, G: Html> {
    children: Children<'a, G>,
    attach: &'a ReadSignal<Option<web_sys::Element>>,
}

#[component]
pub fn Popup<'a, G: Html>(cx: Scope<'a>, props: PopupProps<'a, G>) -> View<G> {
    let children = props.children.call(cx);
    let node_ref = create_node_ref(cx);

    create_effect(cx, || {
        let element = props.attach.get();

        let Some(popup) = try_from_ref::<web_sys::HtmlElement>(node_ref) else { return; };
        let style = popup.style();

        // TODO: the positioning relies that there is no position relative container above
        let Some(element) = element.as_ref() else {
            let _ = style.set_property("display", "none");
            return;
        };

        let window = web_sys::window().unwrap();

        // make visible to be able to query a width and height
        let _ = style.set_property("display", "block");

        let el_rect = element.get_bounding_client_rect();
        let p_rect = popup.get_bounding_client_rect();

        // TODO: dynamic attach points depending on most space available
        // and content width/height
        let el_attach = (el_rect.x() + el_rect.width() / 2.0, el_rect.y()); // middle top

        let p_root = (
            el_attach.0 - (p_rect.width() / 2.0) + window.scroll_x().unwrap_or(0.0),
            el_attach.1 - p_rect.height() + window.scroll_y().unwrap_or(0.0),
        );

        let _ = style.set_property("left", &format!("{}px", p_root.0));
        let _ = style.set_property("top", &format!("{}px", p_root.1));
    });

    view! { cx,
        div(class="absolute z-30 hidden", ref=node_ref) {
            (children)
        }
    }
}
