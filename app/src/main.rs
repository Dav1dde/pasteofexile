use app::App;
use sycamore::prelude::*;

pub fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    let root = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .query_selector("#app")
        .unwrap()
        .unwrap();

    sycamore::hydrate_to(|| view! { App(None) }, &root);
}
