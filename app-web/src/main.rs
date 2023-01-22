use app::App;

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

    sycamore::hydrate_to(|cx| sycamore::view! { cx, App(None) }, &root);
}
