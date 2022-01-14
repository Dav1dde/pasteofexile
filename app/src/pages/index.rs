use pob::{PathOfBuilding, SerdePathOfBuilding};
use sycamore::prelude::*;

macro_rules! memo {
    ($signal:ident, $x:expr) => {
        create_memo(cloned!($signal => move || {
            $x
        }))
    };
}

#[component(IndexPage<G>)]
pub fn index_page() -> View<G> {
    let value = Signal::new(String::new());

    let pob = memo!(value, SerdePathOfBuilding::from_export(&*value.get()));
    let submit_disabled = memo!(pob, pob.get().is_err());
    let title = memo!(
        pob,
        (*pob.get())
            .as_ref()
            .ok()
            .map(|pob| format!("Level {} {}", pob.level(), pob.ascendancy_name()))
            .unwrap_or_else(|| "Share your Build".to_owned())
    );

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
        div(class="flex flex-col gap-y-3") {
            h1(class="dark:text-slate-100 text-slate-900") { (title.get()) }
            textarea(
                bind:value=value,
                spellcheck=false,
                class="dark:bg-slate-500 bg-slate-200 block w-full mt-1 py-2 px-3 rounded-sm shadow-sm focus:outline-none dark:text-slate-300 text-slate-700 resize-none",
                style="min-height: 60vh"
            )
            div(class="text-right") {
                button(
                    on:click=submit,
                    disabled=*submit_disabled.get(),
                    class="bg-sky-500 hover:bg-sky-700 hover:cursor-pointer px-8 py-2 text-sm rounded-lg font-semibold text-white disabled:opacity-50 disabled:cursor-not-allowed"
                    ) {
                    "Create"
                }
            }
        }
    }
}
