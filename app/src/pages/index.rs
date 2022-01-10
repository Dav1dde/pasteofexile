use sycamore::prelude::*;

#[component(IndexPage<G>)]
pub fn index_page() -> View<G> {
    let value = Signal::new(String::new());

    let submit_disabled = create_memo(cloned!((value) => move || (*value.get()).is_empty()));

    #[cfg(not(feature = "ssr"))]
    let submit = cloned!((value) => move |_| {
        use reqwasm::http::Request;
        use wasm_bindgen_futures::spawn_local;

        let value = value.clone();
        spawn_local(async move {
            let _resp = Request::post("/api/v1/paste/")
                .body(&*value.get())
                .send()
                .await
                .unwrap();

            // TODO: redirect to paste
        })
    });
    #[cfg(feature = "ssr")]
    let submit = |_| {};

    view! {
        div {
            h1 { "Index" }
            textarea(bind:value=value)
                button(on:click=submit, disabled=*submit_disabled.get()) {
                    "submit"
                }
        }
    }
}
