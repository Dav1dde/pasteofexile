use crate::{memo, memo_cond};
use pob::{PathOfBuilding, SerdePathOfBuilding};
use sycamore::prelude::*;

const SPINNER: &str = r#"
<svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
</svg>
Creating ...
"#;

#[component(IndexPage<G>)]
pub fn index_page() -> View<G> {
    let value = Signal::new(String::new());
    let loading = Signal::new(false);

    let pob = memo!(value, {
        let value = &*value.get();
        if value.trim().is_empty() {
            return None;
        }
        SerdePathOfBuilding::from_export(value)
            .map_err(|e| log::info!("{}", e))
            .ok()
    });

    let title = memo!(
        pob,
        (*pob.get())
            .as_ref()
            .map(crate::pob::title)
            .unwrap_or_else(|| "Share your Build".to_owned())
    );

    #[cfg(not(feature = "ssr"))]
    let btn_submit = cloned!((loading, value) => move |_| {
        use wasm_bindgen_futures::spawn_local;

        if *loading.get() {
            log::info!("can't submit, already loading");
            return;
        }

        let value = value.get();
        let future = cloned!(loading => async move {
            match crate::api::create_paste(value).await {
                Err(err) => {
                    loading.set(false);
                    log::info!("{:?}", err);
                }
                Ok(response) => sycamore_router::navigate(&response.id),
            };
        });

        loading.set(true);
        spawn_local(future);
    });
    #[cfg(feature = "ssr")]
    let btn_submit = |_| {};

    // TODO: allow pasting of PoBs that cannot be properly parsed but appear to be valid
    let btn_submit_disabled = memo!(loading, pob, *loading.get() || pob.get().is_none());
    let btn_content = memo_cond!(loading, SPINNER, "Create");

    view! {
        div(class="flex flex-col gap-y-3") {
            h1(class="dark:text-slate-100 text-slate-900") { (title.get()) }
            textarea(
                bind:value=value,
                spellcheck=false,
                class="dark:bg-slate-500 bg-slate-200 block w-full mt-1 py-2 px-3 rounded-sm shadow-sm focus:outline-none dark:text-slate-300 text-slate-700 resize-none text-sm break-all",
                style="min-height: 60vh"
            )
            div(class="text-right") {
                button(
                    on:click=btn_submit,
                    disabled=*btn_submit_disabled.get(),
                    class="bg-sky-500 hover:bg-sky-700 hover:cursor-pointer px-6 py-2 text-sm rounded-lg font-semibold text-white disabled:opacity-50 disabled:cursor-not-allowed inline-flex",
                    dangerously_set_inner_html=*btn_content.get()
                ) {
                }
            }
        }
    }
}
