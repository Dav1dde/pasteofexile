use ::pob::{PathOfBuilding, PathOfBuildingExt};
use shared::PasteId;
use sycamore::{futures::spawn_local_scoped, prelude::*};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlTextAreaElement;

use super::PobGearPreview;
use crate::{
    build::Build,
    components::{PobColoredText, PobGems, PobTreePreview},
    consts::IMG_ONERROR_HIDDEN,
    pob::{self, Element},
    storage::Storage,
    utils::{async_callback, document, from_ref, view_cond, IteratorExt},
};

pub struct ViewPasteProps<'a> {
    pub id: PasteId,
    pub title: Option<String>,
    pub last_modified: u64,
    pub build: &'a Build,
}

#[allow(dead_code)]
#[derive(PartialEq, Eq)]
enum CopyState {
    Ready,
    Progress,
    Done,
    Failed,
}

impl CopyState {
    fn name(&self) -> &'static str {
        match self {
            Self::Ready => "Copy",
            Self::Progress => "Copy",
            Self::Done => "Copied",
            Self::Failed => "Failed",
        }
    }
}

#[component]
pub fn ViewPaste<'a, G: Html>(
    cx: Scope<'a>,
    ViewPasteProps {
        id,
        title,
        last_modified,
        build,
    }: ViewPasteProps<'a>,
) -> View<G> {
    let title = title.unwrap_or_else(|| pob::title(build.pob()));

    push_paste_to_history::<G>(cx, &id, &title, last_modified, build);

    let version = build.max_tree_version().unwrap_or_default();
    let since = crate::utils::pretty_date_ts(last_modified);
    let date = js_sys::Date::new(&JsValue::from_f64(last_modified as f64)).to_string();

    let open_in_pob_url = id.to_pob_open_url();

    let notes = view_cond!(cx, !build.notes().is_empty(), {
        div(class="flex-auto") {
            h2(class="text-lg dark:text-slate-100 text-slate-900 mb-2 mt-24 border-b border-solid") { "Notes" }
            pre(class="text-xs break-words whitespace-pre-wrap font-mono sm:ml-3") {
                PobColoredText(text=build.notes(), links=true)
            }
        }
    });
    let tree_preview = view_cond!(cx, has_displayable_tree(build.pob()), {
        div(class="basis-full") {
            h2(class="text-lg dark:text-slate-100 text-slate-900 mb-2 mt-12 border-b border-solid") { "Tree Preview" }
            PobTreePreview(build)
        }
    });

    let select_all = |event: web_sys::Event| {
        let s: HtmlTextAreaElement = event.target().unwrap().unchecked_into();
        let _ = s.focus();
        s.select();
    };

    let content_ref = create_node_ref(cx);
    let copy_state = create_signal(cx, CopyState::Ready);

    let copy_to_clipboard = async_callback!(
        cx,
        {
            copy_state.set(CopyState::Progress);

            from_ref::<web_sys::HtmlTextAreaElement>(content_ref).select();

            let document: web_sys::HtmlDocument = document();
            let state = if document.exec_command("copy").is_ok() {
                CopyState::Done
            } else {
                CopyState::Failed
            };

            let _ = document
                .get_selection()
                .unwrap()
                .unwrap()
                .remove_all_ranges();

            copy_state.set(state);

            gloo_timers::future::TimeoutFuture::new(1_000).await;
            copy_state.set(CopyState::Ready);
        },
        *copy_state.get() == CopyState::Ready
    );

    let btn_copy_name = create_memo(cx, || copy_state.get().name());
    let btn_copy_disabled = create_memo(cx, || *copy_state.get() != CopyState::Ready);

    let core_stats = pob::summary::core_stats(build.pob());
    let defense = pob::summary::defense(build.pob());
    let offense = pob::summary::offense(build.pob());
    let config = pob::summary::config(build.pob());

    let summary = [core_stats, defense, offense, config]
        .into_iter()
        .map(|stat| render(cx, stat))
        .map(|stat| view! { cx, div(class="flex-row gap-x-5") { (stat) } })
        .collect_view();

    let src =
        crate::assets::ascendancy_image(build.pob().ascendancy_or_class_name()).unwrap_or_default();

    view! { cx,
        div(class="text-right text-sm text-slate-500", title=date, data-last-modified=last_modified) { (since) }
        div(class="flex flex-col md:flex-row gap-y-5 md:gap-x-3 mb-24") {
            div(class="flex-auto flex flex-col gap-y-2 -mt-[3px]") {
                h1(class="flex items-center text-xl mb-1 dark:text-slate-100 text-slate-900") {
                    img(src=src,
                        class="asc-image rounded-full mr-3 -ml-2",
                        alt="Ascendancy Thumbnail",
                        onerror=IMG_ONERROR_HIDDEN) {}
                    span(class="pt-[3px]", data-marker-title="") { (title) }
                    sup(class="ml-1") {
                        span { (version) }
                    }
                }
                (summary)
            }
            div(class="flex flex-col flex-initial gap-y-3 md:w-96") {
                textarea(
                    ref=content_ref,
                    on:click=select_all,
                    class="flex-auto resize-none text-sm break-all outline-none max-h-40
                        min-h-[5rem] dark:bg-slate-600 bg-slate-200 dark:text-slate-400 text-slate-700
                        rounded-sm shadow-sm pl-1 overflow-x-hidden",
                    data-marker-content="",
                    aria-label="Path of Building buildcode",
                    readonly=true
                ) {
                    (build.content)
                }
                div(class="text-right") {
                    button(
                        on:click=copy_to_clipboard,
                        disabled=*btn_copy_disabled.get(),
                        title="Copy to Clipboard",
                        class="hover:underline hover:cursor-pointer px-6 py-2 text-sm disabled:cursor-not-allowed inline-flex"
                    ) { (btn_copy_name.get()) }
                    a(
                        href=open_in_pob_url,
                        title="Open build in Path of Building, requires up to date PoB",
                        class="btn btn-primary"
                    ) { "Open" }
                }
            }
        }
        div(class="flex flex-wrap gap-x-10 gap-y-16") {
            div(class="flex-auto w-60") {
                h2(class="text-lg dark:text-slate-100 text-slate-900 mb-2 border-b border-solid") { "Gear" }
                PobGearPreview(build)
            }
            div(class="flex-auto w-full lg:w-auto") {
                h2(class="text-lg dark:text-slate-100 text-slate-900 mb-2 border-b border-solid") { "Gems" }
                PobGems(build)
            }
        }
        (tree_preview)
        (notes)
        div(class="h-[150px]") {}
    }
}

fn render<G: Html>(cx: Scope, elements: Vec<Element>) -> View<G> {
    elements
        .into_iter()
        .filter_map(|e| e.render_to_view(cx))
        .collect_view()
}

fn has_displayable_tree(pob: &impl PathOfBuilding) -> bool {
    let specs = pob.tree_specs();

    specs.len() > 1
        || specs
            .first()
            .map(|spec| spec.nodes.len() > 1)
            .unwrap_or(false)
}

fn push_paste_to_history<G: Html>(
    cx: Scope,
    id: &PasteId,
    title: &str,
    last_modified: u64,
    build: &Build,
) {
    if G::IS_BROWSER {
        let storage = use_context::<Storage>(cx);

        let s = shared::model::PasteSummary {
            id: id.clone(),
            title: title.to_owned(),
            ascendancy_or_class: build.ascendancy_or_class_name().to_owned(),
            version: build.max_tree_version(),
            main_skill_name: build.main_skill_name().map(|s| s.to_owned()),
            last_modified,
            rank: None,
        };

        spawn_local_scoped(cx, async move {
            gloo_timers::future::sleep(std::time::Duration::from_millis(500)).await;
            storage.visited().add(s);
        });
    }
}
