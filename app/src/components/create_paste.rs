use crate::{memo, memo_cond, model::UserPasteId, session::SessionValue, svg::SPINNER};
use pob::SerdePathOfBuilding;
use sycamore::{context::use_context, prelude::*};

pub enum CreatePasteProps {
    None,
    Update { id: UserPasteId, content: String },
}

impl CreatePasteProps {
    fn content(&self) -> Option<String> {
        match self {
            // TODO: should be able to get rid of that clone
            Self::Update { content, .. } => Some(content.clone()),
            _ => None,
        }
    }

    fn is_update(&self) -> bool {
        matches!(self, Self::Update { .. })
    }

    #[cfg(not(feature = "ssr"))]
    fn paste_id(&self) -> Option<&UserPasteId> {
        match self {
            Self::Update { id, .. } => Some(id),
            _ => None,
        }
    }
}

impl Default for CreatePasteProps {
    fn default() -> Self {
        Self::None
    }
}

#[component(CreatePaste<G>)]
pub fn create_paste(props: CreatePasteProps) -> View<G> {
    let is_update = props.is_update();

    let value = Signal::new(props.content().unwrap_or_default());
    let loading = Signal::new(false);
    let error = Signal::new(String::new());
    let as_user = Signal::new(is_update);
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
            .unwrap_or_else(|| if is_update {
                "Update your Build".to_owned()
            } else {
                "Share your Build".to_owned()
            })
    );

    #[cfg(not(feature = "ssr"))]
    let paste_id = props.paste_id().cloned();
    #[cfg(not(feature = "ssr"))]
    let btn_submit = cloned!((loading, value, error, as_user, title, custom_title, paste_id) => move |_| {
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
        let future = cloned!((loading, error, paste_id) => async move {
            let title = if custom_title.is_empty() { &*title } else { &*custom_title };
            let id = paste_id.map(|e| e.clone().into());
            // TODO: include PasteId here, for updates
            let params = api::CreatePaste {
                as_user,
                content: &*value,
                title,
                id: id.as_ref(),
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
    let btn_content = memo_cond!(
        loading,
        SPINNER,
        if is_update { "Update" } else { "Create" }
    );

    let on_input = cloned!(error => move |_| error.set("".to_owned()));

    let as_user2 = as_user.clone();
    let value2 = value.clone();
    view! {
        div(class="flex flex-col gap-y-3") {
            h1(class="dark:text-slate-100 text-slate-900") { (title.get()) }
            textarea(
                bind:value=value,
                on:input=on_input,
                spellcheck=false,
                class="dark:bg-slate-500 bg-slate-200 block w-full mt-1 py-2 px-3 rounded-sm shadow-sm focus:outline-none dark:text-slate-300 text-slate-700 resize-none text-sm break-all",
                style="min-height: 60vh"
            ) {
                (value2.get())
            }
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
                    (if session.get().is_logged_in() && !is_update {
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
