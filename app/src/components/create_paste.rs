use pob::SerdePathOfBuilding;
use shared::{model::UserPasteId, validation};
use sycamore::{context::use_context, prelude::*};
use wasm_bindgen::JsCast;

use crate::{memo, memo_cond, session::SessionValue, svg::SPINNER};

pub enum CreatePasteProps {
    None,
    Update {
        id: UserPasteId,
        content: String,
        title: Option<String>,
    },
}

impl CreatePasteProps {
    fn content(&self) -> Option<String> {
        match self {
            // TODO: should be able to get rid of that clone (not use bind?)
            Self::Update { content, .. } => Some(content.to_string()),
            _ => None,
        }
    }

    fn title(&self) -> Option<String> {
        match self {
            // TODO: should be able to get rid of that clone
            Self::Update { title, .. } => title.clone(),
            _ => None,
        }
    }

    fn is_update(&self) -> bool {
        matches!(self, Self::Update { .. })
    }

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
    let custom_title = Signal::new(props.title().unwrap_or_default());
    let custom_id = Signal::new(props.paste_id().map(|up| up.id.clone()).unwrap_or_default());

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
                tracing::info!("{}", err);
                error.set("Invalid PoB Code".to_owned());
                None
            }
        }
    });

    let raw_title = memo!(pob, (*pob.get()).as_ref().map(crate::pob::title));
    let title = memo!(
        raw_title,
        (*raw_title.get())
            .as_ref()
            .cloned()
            .unwrap_or_else(|| if is_update {
                "Update your Build".to_owned()
            } else {
                "Share your Build".to_owned()
            })
    );

    #[cfg(not(feature = "ssr"))]
    let paste_id = props.paste_id().cloned();
    #[cfg(not(feature = "ssr"))]
    let btn_submit = cloned!(loading, value, error, as_user, title, custom_title, custom_id, paste_id => move |_| {
        use wasm_bindgen_futures::spawn_local;
        use crate::api;

        if *loading.get() {
            tracing::info!("can't submit, already loading");
            return;
        }

        error.set("".to_owned());

        let value = value.get();
        let as_user = *as_user.get();
        let title = title.get();
        let custom_title = custom_title.get();
        let custom_id = custom_id.get();
        let future = cloned!(loading, error, paste_id => async move {
            let id = paste_id.map(|e| e.clone().into());
            let title = if custom_title.is_empty() { &*title } else { &*custom_title };

            let params = api::CreatePaste {
                id: id.as_ref(),
                as_user,
                title,
                custom_id: (!custom_id.is_empty()).then_some(&*custom_id),
                content: &*value,
            };
            match api::create_paste(params).await {
                Err(err) => {
                    loading.set(false);
                    error.set(err.to_string());
                    tracing::info!("{:?}", err);
                }
                Ok(id) => {
                    sycamore_router::navigate(&id.to_url());
                }
            };
        });

        loading.set(true);
        spawn_local(future);
    });
    #[cfg(feature = "ssr")]
    let btn_submit = |_| {};

    // TODO: allow pasting of PoBs that cannot be properly parsed but appear to be valid
    let btn_submit_disabled = memo!(loading, pob, as_user, custom_title, custom_id, {
        // TODO: show error/validation messages, this memo probably needs to return a `Validation`
        // and the button subscribes to memo with `is_valid()` and `error` merges from the
        // validation as well.
        if *loading.get() || pob.get().is_none() {
            return true;
        }
        if *as_user.get() {
            // Empty means auto generated or default
            let id = custom_id.get().is_empty()
                || validation::user::is_valid_custom_id(&custom_id.get()).is_valid();
            let title = custom_title.get().is_empty()
                || validation::user::is_valid_custom_title(&custom_title.get()).is_valid();
            if !id || !title {
                return true;
            }
        }

        false
    });
    let btn_content = memo_cond!(
        loading,
        SPINNER,
        if is_update { "Update" } else { "Create" }
    );

    let on_input = cloned!(error => move |_| error.set("".to_owned()));

    let on_custom_id = cloned!(custom_id => move |event: web_sys::Event| {
        let event = event.unchecked_into::<web_sys::InputEvent>();
        if event.is_composing() {
            return;
        }

        let target = event
            .target()
            .unwrap()
            .unchecked_into::<web_sys::HtmlInputElement>();
        let value = target
            .value()
            .chars()
            .map(|c| if c == ' ' { '_' } else { c })
            .filter(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_'))
            .collect::<String>();
        target.set_value(&value);
        custom_id.set(value);
    });

    let as_user_content = memo_cond!(
        as_user,
        {
            let title = raw_title.clone();
            let title2 = (*raw_title.get()).clone().unwrap_or_default();
            let custom_id = custom_id.clone();
            let on_custom_id = on_custom_id.clone();
            view! {
                div() { "Title" }
                input(
                    class="input",
                    type="text",
                    maxlength=90,
                    minlength=3,
                    value=title2,
                    aria-label="Title",
                    placeholder=title.get().as_deref().unwrap_or_default(),
                    bind:value=custom_title.clone(),
                ) {}

                div(title="Id of the build, reusing an Id overwrites the previous build") { "Id" }
                input(
                    class="input",
                    type="text",
                    maxlength=90,
                    minlength=3,
                    pattern="[a-zA-Z0-9-_]*",
                    aria-label="Build ID",
                    placeholder="<auto generated>",
                    readonly=is_update,
                    value=custom_id.get(),
                    on:input=on_custom_id) { }
            }
        },
        view! {}
    );

    let cancel = if is_update {
        view! {
            button(
                title="Back",
                tabindex="-1",
                onclick="window.history.go(-1)",
                class="hover:underline hover:cursor-pointer select-none
                text-sm disabled:cursor-not-allowed inline-flex"
            ) { "Cancel" }
        }
    } else {
        view! {}
    };

    let value2 = value.clone();
    view! {
        div(class="flex flex-col gap-y-3") {
            h1(class="dark:text-slate-100 text-slate-900", data-marker-title="") { (title.get()) }
            textarea(
                bind:value=value,
                on:input=on_input,
                spellcheck=false,
                class="dark:bg-slate-500 bg-slate-200 block w-full mt-1 py-2 px-3
                    rounded-sm shadow-sm focus:outline-none dark:text-slate-300 text-slate-700
                    resize-none text-sm break-all",
                style="height: 50vh; max-height: 650px",
                data-marker-content="",
                aria-label="Path of Building buildcode",
            ) {
                (value2.get())
            }
            div(class="grid grid-cols-[min-content_1fr] gap-3 items-center empty:hidden") {
                (&*as_user_content.get())
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
                (cancel)
                button(
                    on:click=btn_submit,
                    disabled=*btn_submit_disabled.get(),
                    class="btn btn-primary min-w-[100px]",
                    dangerously_set_inner_html=&btn_content.get()
                ) {
                }
            }
        }
    }
}
