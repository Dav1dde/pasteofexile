use sycamore::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlSelectElement;

use super::pob_colored_text::{color_to_style, Style};
use crate::{
    consts::SELECT_ONCHANGE_COLOR_FROM_OPTION,
    pob::formatting::{only_first_color, Color},
};

#[derive(Prop)]
pub struct PobColoredSelectProps<F> {
    pub options: Vec<String>, // TODO: this could be Vec<(Color, String)> which would be one less
    // string clone, or with sycamore-0.8 &str
    pub selected: Option<usize>,
    pub label: &'static str,
    pub on_change: F,
}

#[component]
pub fn PobColoredSelect<'a, G: Html, F>(cx: Scope<'a>, props: PobColoredSelectProps<F>) -> View<G>
where
    F: Fn(Option<usize>) + 'a,
{
    let selected_index = props.selected.unwrap_or(0);

    let mut start_color = Color::None;
    let mut options = Vec::new();
    for (i, content) in props.options.iter().enumerate() {
        let (color, content) = only_first_color(content);
        let selected = i == selected_index;

        if selected {
            start_color = color;
        }

        let v = match color_to_style(color) {
            Style::Class(class) => {
                view! { cx, option(selected=selected, class=class) { (content) } }
            }
            Style::Style(style) => {
                view! { cx, option(selected=selected, style=style) { (content) } }
            }
            Style::None => {
                // Use "default" color here to make sure
                // the option doesnt inherit the color from the select.
                view! { cx, option(selected=selected, class="text-slate-300") { (content) } }
            }
        };

        options.push(v);
    }
    let options = View::new_fragment(options);

    let on_input = move |event: web_sys::Event| {
        let event = event.unchecked_into::<web_sys::InputEvent>();
        let element = event
            .target()
            .unwrap()
            .unchecked_into::<HtmlSelectElement>();

        let index = element.selected_index();
        let index = if index < 0 {
            None
        } else {
            Some(index as usize)
        };
        (props.on_change)(index);
    };

    let (class, style) = match color_to_style(start_color) {
        Style::Class(class) => (class, String::new()),
        Style::Style(style) => ("", style),
        Style::None => ("", String::new()),
    };
    let class = format!("sm:ml-3 mt-1 mb-2 px-1 max-w-full {class}");

    view! { cx,
        select(
            class=class,
            style=style,
            aria-label=props.label,
            on:input=on_input,
            onchange=SELECT_ONCHANGE_COLOR_FROM_OPTION
        ) { (options) }
    }
}
