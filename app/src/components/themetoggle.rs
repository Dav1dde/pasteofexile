use crate::memo_cond as css;
use sycamore::prelude::*;

static SVG_SUN: &str = r#"
<svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
</svg>
"#;

static SVG_MOON: &str = r#"
<svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
</svg>
"#;

#[component(ThemeToggle<G>)]
pub fn theme_toggle() -> View<G> {
    let active = Signal::new(true);

    #[cfg(not(feature = "ssr"))]
    create_effect(cloned!(active => move || {
        let class_list = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .document_element()
            .unwrap()

            .class_list();

        if *active.get() {
            class_list.add_1("dark").unwrap();
        } else {
            class_list.remove_1("dark").unwrap();
        }
    }));

    let toggle = cloned!(active => move |_| active.set(!*active.get()));

    let bg = css!(
        active,
        "bg-sky-500 w-14 h-7 flex items-center rounded-full mx-3 px-1",
        "bg-slate-300 w-14 h-7 flex items-center rounded-full mx-3 px-1"
    );
    let btn = css!(
        active,
        "bg-white w-5 h-5 rounded-full shadow-md transform translate-x-7",
        "bg-white w-5 h-5 rounded-full shadow-md transform"
    );
    let sun_class = css!(active, "", "text-sky-500");
    let moon_class = css!(active, "text-sky-500", "");

    view! {
        div(class="flex") {
            span(class=*sun_class.get(), dangerously_set_inner_html=SVG_SUN)
            div(class=*bg.get(), on:click=toggle) {
                div(class=*btn.get())
            }
            span(class=*moon_class.get(), dangerously_set_inner_html=SVG_MOON)
        }
    }
}
