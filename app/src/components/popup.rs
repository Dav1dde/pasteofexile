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
        let viewport_width = window.inner_width().unwrap().as_f64().unwrap();
        let scroll_x = window.scroll_x().unwrap_or(0.0);
        let scroll_y = window.scroll_y().unwrap_or(0.0);

        // make visible to be able to query a width and height
        // at a position where the popup can get its full width and height
        let _ = style.set_property("left", "-1000px");
        let _ = style.set_property("top", "0");
        let _ = style.set_property("max-width", &format!("{viewport_width}px"));
        let _ = style.set_property("display", "block");

        let el_rect = element.get_bounding_client_rect();
        let p_rect = popup.get_bounding_client_rect();

        // TODO: dynamic attach points depending on most space available
        // and content width/height
        let el_attach = (el_rect.x() + el_rect.width() / 2.0, el_rect.y()); // middle top

        // TODO: dont perfectly center if it would go out of bounds (left or right)
        let mut p_root = (
            el_attach.0 - (p_rect.width() / 2.0) + scroll_x,
            (el_attach.1 - p_rect.height() + scroll_y).max(scroll_y),
        );

        // correct the right overflow to the left
        let p_x_end = p_root.0 + p_rect.width();
        if p_x_end > viewport_width {
            // -20px because of a weird bug where the browser makes and element
            // smaller than it has width for, only happens on a certain amulet?
            p_root = (p_root.0 - (p_x_end - viewport_width) - 20.0, p_root.1);
        }

        let _ = style.set_property("left", &format!("{}px", p_root.0.max(0.0)));
        let _ = style.set_property("top", &format!("{}px", p_root.1));
    });

    view! { cx,
        div(class="absolute z-30 pointer-events-none hidden", ref=node_ref) {
            (children)
        }
    }
}
