use sycamore::prelude::*;

use crate::{svg, utils::memo_cond};

#[component]
pub fn ImportPastebin<G: Html>(cx: Scope) -> View<G> {
    let value = create_signal(cx, String::new());
    let loading = create_signal(cx, false);

    let pastebin_id = create_memo(cx, || {
        value
            .get()
            .strip_prefix("https://pastebin.com/")
            .filter(|candidate| is_pastebin_id(candidate))
            .map(|id| id.to_owned())
    });

    let btn_disabled = create_memo(cx, || *loading.get() || pastebin_id.get().is_none());

    let submit = |_| {
        if let Some(id) = pastebin_id.get().as_ref() {
            loading.set(true);
            sycamore_router::navigate(id);
        }
    };

    let btn_content = memo_cond!(cx, loading, svg::SPINNER, "Import");

    view! { cx,
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
