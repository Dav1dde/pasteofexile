use sycamore::prelude::*;

use crate::pob::formatting::{Color, ColoredText};

#[component]
pub fn PobColoredText<G: Html>(cx: Scope, text: &str) -> View<G> {
    let t = ColoredText::new(text)
        .map(|cs| render_fragment(cx, cs))
        .collect();

    View::new_fragment(t)
}

pub enum Style {
    Class(&'static str),
    Style(String),
    None,
}

pub fn color_to_style(color: Color<'_>) -> Style {
    match color {
        Color::Hex(hex) => Style::Style(format!("color: #{hex}")),
        Color::Named(name) => Style::Class(name_to_class(name)),
        Color::None => Style::None,
    }
}

fn render_fragment<G: GenericNode>(cx: Scope, (color, text): (Color, &str)) -> View<G> {
    let text = text.to_owned();
    match color_to_style(color) {
        Style::Class(class) => view! { cx, span(class=class) { (text) } },
        Style::Style(style) => view! { cx, span(style=style) { (text) } },
        Style::None => view! { cx, span { (text) } },
    }
}

fn name_to_class(name: u8) -> &'static str {
    match name {
        0 => "text-slate-900",
        1 => "text-red-600",
        2 => "text-green-600",
        3 => "text-blue-600",
        4 => "text-yellow-400",
        5 => "text-fuchsia-500",
        6 => "text-cyan-400",
        7 => "", // normal text color
        8 => "text-zinc-400",
        9 => "text-zinc-600",
        _ => "", // never happens
    }
}
