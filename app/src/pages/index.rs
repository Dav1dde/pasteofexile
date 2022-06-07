use crate::{memo, memo_cond, session::SessionValue};
use pob::SerdePathOfBuilding;
use sycamore::{context::use_context, prelude::*};

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
    let error = Signal::new(String::new());
    let as_user = Signal::new(false);
    let custom_title = Signal::new(String::new());

    let session = use_context::<SessionValue>();

    let pob = memo!(value, error, {
        let _ = error.get(); // register signal

        let value = &*value.get();
        if value.trim().is_empty() {
            return None;
        }

        match SerdePathOfBuilding::from_export(value) {
            Ok(pob) => Some(pob),
            Err(err) => {
                log::info!("{}", err);
                error.set("Invalid PoB Code".to_owned());
                None
            }
        }
    });

    let title = memo!(
        pob,
        (*pob.get())
            .as_ref()
            .map(crate::pob::title)
            .unwrap_or_else(|| "Share your Build".to_owned())
    );

    #[cfg(not(feature = "ssr"))]
    let btn_submit = cloned!((loading, value, error, as_user, title, custom_title) => move |_| {
        use wasm_bindgen_futures::spawn_local;
        use crate::api;

        if *loading.get() {
            log::info!("can't submit, already loading");
            return;
        }

        error.set("".to_owned());

        let value = value.get();
        let as_user = *as_user.get();
        let title = title.get();
        let custom_title = custom_title.get();
        let future = cloned!((loading, error) => async move {
            let title = if custom_title.is_empty() { &*title } else { &*custom_title };
            let params = api::CreatePaste {
                as_user,
                content: &*value,
                title,
            };
            match api::create_paste(params).await {
                Err(err) => {
                    loading.set(false);
                    error.set(err.to_string());
                    log::info!("{:?}", err);
                }
                Ok(response) => {
                    if let Some(user) = response.user {
                        sycamore_router::navigate(&format!("/u/{}/{}", user, response.id))
                    } else {
                        sycamore_router::navigate(&response.id)
                    }
                }
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

    let on_input = cloned!(error => move |_| error.set("".to_owned()));

    let as_user2 = as_user.clone();
    view! {
        div(class="flex flex-col gap-y-3") {
            h1(class="dark:text-slate-100 text-slate-900") { (title.get()) }
            textarea(
                bind:value=value,
                on:input=on_input,
                spellcheck=false,
                class="dark:bg-slate-500 bg-slate-200 block w-full mt-1 py-2 px-3 rounded-sm shadow-sm focus:outline-none dark:text-slate-300 text-slate-700 resize-none text-sm break-all",
                style="min-height: 60vh"
            )
            div(class="grid grid-cols-[min-content_1fr] gap-3 items-center empty:hidden") {
                (if *as_user2.get() {
                    view! {
                        div(class="") { "Title" }
                        input(
                            class="dark:bg-slate-500 bg-slate-200 w-full px-2 py-1 rounded-sm outline-none",
                            maxlength=90,
                            minlength=3,
                            bind:value=custom_title.clone()) {}
                    }
                } else { view! {} }
                )
            }
            div(class="flex items-center gap-x-5") {
                div(class="flex-auto flex items-center text-red-500") { (*error.get()) }
                div() { // need the div for hydration to not break
                    (if session.get().is_logged_in() {
                        view! {
                            label() {
                                input(type="checkbox", class="mx-2", bind:checked=as_user.clone()) {}
                                "Share on Profile"
                            }
                        }
                    } else {
                        view! {}
                    })
                }
                button(
                    on:click=btn_submit,
                    disabled=*btn_submit_disabled.get(),
                    class="bg-sky-500 hover:bg-sky-700 hover:cursor-pointer px-6 py-2 text-sm rounded-lg font-semibold text-white disabled:opacity-50 disabled:cursor-not-allowed flex",
                    dangerously_set_inner_html=*btn_content.get()
                ) {
                }
            }
        }
    }
}
