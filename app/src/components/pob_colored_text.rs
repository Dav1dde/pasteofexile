use sycamore::prelude::*;

use crate::pob::formatting::{Color, ColoredText};

#[component(PobColoredText<G>)]
pub fn pob_tree_table(text: String) -> View<G> {
    let t = ColoredText::new(&text).map(render_fragment).collect();

    View::new_fragment(t)
}

fn render_fragment<G: GenericNode>((color, text): (Color, &str)) -> View<G> {
    let text = text.to_owned();

    match color {
        Color::Hex(hex) => {
            let style = format!("color: #{hex}");
            view! { span(style=style) { (text) } }
        }
        Color::Named(name) => {
            let class = name_to_class(name);
            view! { span(class=class) { (text) } }
        }
        Color::None => view! { span { (text) } },
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
