use sycamore::prelude::*;

use crate::{memo, memo_cond, svg};

#[component(ImportPastebin<G>)]
pub fn import_pastebin() -> View<G> {
    let value = Signal::new(String::new());
    let loading = Signal::new(false);

    let pastebin_id = memo!(value, {
        value
            .get()
            .strip_prefix("https://pastebin.com/")
            .filter(|candidate| is_pastebin_id(candidate))
            .map(|id| id.to_owned())
    });

    let btn_disabled = memo!(pastebin_id, loading, {
        *loading.get() || pastebin_id.get().is_none()
    });

    let submit = cloned!(pastebin_id, loading => move |_| {
        if let Some(id) = pastebin_id.get().as_ref() {
            loading.set(true);
            sycamore_router::navigate(id);
        }
    });

    let btn_content = memo_cond!(loading, svg::SPINNER, "Import");

    view! {
        div(class="flex flex-col gap-y-1") {
            div(class="dark:text-slate-200 text-slate-800") { "Import from pastebin.com" }
            form(class="flex flex-wrap items-center justify-end gap-3") {
                input(class="input flex-1 basis-[14rem]", bind:value=value) {}
                button(
                    class="btn btn-primary min-w-[100px]",
                    type="submit",
                    disabled=*btn_disabled.get(),
                    on:click=submit,
                    dangerously_set_inner_html=&btn_content.get()
                ) {}
            }
        }
    }
}

fn is_pastebin_id(candidate: &str) -> bool {
    candidate.len() == 8
        && candidate
            .bytes()
            .all(|c| matches!(c, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9'))
}
