use shared::PasteId;
use sycamore::prelude::*;
use wasm_bindgen::JsCast;

use crate::{
    consts::IMG_ONERROR_INVISIBLE,
    storage::{PasteList, Storage, StoredPaste},
    svg,
    utils::{
        document, hooks::scoped_event_passive, is_at_least_medium_breakpoint, memo_cond,
        on_click_anchor, pretty_date_ts, view_if, IteratorExt,
    },
};

#[component]
pub fn PasteHistory<G: Html>(cx: Scope) -> View<G> {
    let open = create_signal(cx, false);
    let open_toggle = || open.set(!*open.get());

    let open_style = memo_cond!(cx, open, "transform: translateX(0)", "");

    let close_on_mobile = move |_| {
        // TODO: this does not work on navigation to user page on mobile
        // because we stop propagation on the anchor tag to not double navigate.
        // Shitty to fix ...
        if *open.get() && !is_at_least_medium_breakpoint() {
            open_toggle();
        }
    };

    let content = view_if!(cx, open, {
        div(class="sticky flex justify-center text-slate-100 border-b border-sky-400 p-2") {
            button(
                on:click=move |_| open_toggle(),
                title="Close",
                class="absolute top-[10px] left-2 w-[16px] text-sky-400 hover:text-sky-200",
                dangerously_set_inner_html=svg::CHEVRON_RIGHT,
            ) {}

            div(class="flex gap-1 items-center") {
                div(class="w-[16px]", dangerously_set_inner_html=svg::HISTORY) {}
                "History"
            }
        }

        div(on:click=close_on_mobile, class="h-full overflow-y-auto pt-3") {
            PasteHistoryElements()
        }
    });

    if G::IS_BROWSER {
        scoped_event_passive(cx, document(), "keyup", move |ev: web_sys::Event| {
            let ev = ev.unchecked_into::<web_sys::KeyboardEvent>();
            if *open.get() && ev.key_code() == 27 {
                open_toggle();
            }
        });

        // Hide the document scroll bar if the overlay is full screen.
        // When closing reset to default.
        create_effect(cx, move || {
            let should_hide = *open.get() && !is_at_least_medium_breakpoint();
            let _ = document::<web_sys::HtmlDocument>()
                .body()
                .unwrap()
                .style()
                .set_property("overflow-y", if should_hide { "hidden" } else { "auto" });
        });
    }

    view! { cx,
        button(
            on:click=move |_| open_toggle(),
            title="History",
            class="w-[16px] text-sky-400 hover:text-sky-200",
            dangerously_set_inner_html=svg::HISTORY,
        ) {}
        div(style=open_style.get(), class="modal-right flex flex-col") {
            (*content.get())
        }
    }
}

#[component]
fn PasteHistoryElements<G: Html>(cx: Scope) -> View<G> {
    let storage = use_context::<Storage>(cx);

    view! { cx,
        PageHistoryList(storage.visited())
    }
}

#[component]
fn PageHistoryList<'a, G: Html>(cx: Scope<'a>, list: PasteList<'a>) -> View<G> {
    let items = list
        .get_all()
        .into_iter()
        .map(|item| render_history_item(cx, item))
        .collect_view();

    view! { cx,
        ul(class="flex flex-col gap-2 mb-5") {
            (items)
        }
    }
}

fn render_history_item<G: Html>(cx: Scope, item: StoredPaste) -> View<G> {
    let href = item.paste.id.to_url();

    let color = crate::meta::get_color(item.paste.ascendancy_or_class);
    let image = crate::assets::ascendancy_image(item.paste.ascendancy_or_class);

    let time = pretty_date_ts(item.stored);
    let version = item.paste.version.unwrap_or_default();

    let by = if let PasteId::UserPaste(id) = item.paste.id {
        let href = id.to_user_url();
        view! { cx,
            span(class="ml-2") {
                span() { "[by " }
                a(class="text-sky-400 hover:text-sky-200 hover:underline",  href=href, on:click=on_click_anchor) {
                    (id.user)
                }
                span() { "]" }
            }
        }
    } else {
        View::empty()
    };

    view! { cx,
        li(class="flex items-center gap-2 hover:bg-[color:var(--bg-col)]",
        style=format!("--col: {color}; --bg-col: {color}66")) {
            img(src=image,
                class="asc-image rounded-r-full",
                alt="Ascendancy Thumbnail",
                onerror=IMG_ONERROR_INVISIBLE) {}

            a(href=href, class="flex-1 flex flex-col gap-1") {
                div(class="text-amber-50") { (item.paste.title) sup(class="ml-1") { (version) } }
                div(class="text-xs") {
                    span { (time) }
                    (by)
                }
            }
        }
    }
}
