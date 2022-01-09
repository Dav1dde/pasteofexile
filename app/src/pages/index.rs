#[cfg(not(feature = "ssr"))]
use reqwasm::http::Request;
use sycamore::prelude::*;
#[cfg(not(feature = "ssr"))]
use wasm_bindgen_futures::spawn_local;

#[component(IndexPage<G>)]
pub fn index_page() -> View<G> {
    let value = Signal::new(String::new());

    let submit_disabled = create_memo(cloned!((value) => move || (*value.get()).len() == 0));

    #[cfg(not(feature = "ssr"))]
    let submit = cloned!((value) => move |_| {
        let value = value.clone();
        spawn_local(async move {
            let resp = Request::post("/api/v1/paste/")
                .body(&*value.get())
                .send()
                .await
                .unwrap();

            log::info!("{:?}", resp);
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
