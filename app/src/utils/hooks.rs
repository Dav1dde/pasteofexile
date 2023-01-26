use sycamore::prelude::*;
use wasm_bindgen::{JsCast, UnwrapThrowExt};

// TODO: 1.67 Regressio https://github.com/rust-lang/rust/issues/107426
// pub fn create_toggle_bool(cx: Scope, initial: bool) -> (&ReadSignal<bool>, impl Fn() + Copy + '_) {
//     let state = create_signal(cx, initial);
//
//     (state, || state.set(!*state.get()))
// }

pub fn scoped_event_passive<'a, F: FnMut(web_sys::Event) + 'a>(
    cx: Scope<'a>,
    node: web_sys::HtmlElement,
    name: &'static str,
    handler: F,
) {
    let boxed: Box<dyn FnMut(web_sys::Event)> = Box::new(handler);
    let handler: Box<dyn FnMut(web_sys::Event) + 'static> = unsafe { std::mem::transmute(boxed) };
    let closure = create_ref(cx, wasm_bindgen::closure::Closure::wrap(handler));

    let mut options = web_sys::AddEventListenerOptions::new();
    options.passive(true);

    node.add_event_listener_with_callback_and_add_event_listener_options(
        name,
        closure.as_ref().unchecked_ref(),
        &options,
    )
    .unwrap_throw();

    on_cleanup(cx, move || {
        node.remove_event_listener_with_callback(name, closure.as_ref().unchecked_ref())
            .unwrap_throw();
    });
}
