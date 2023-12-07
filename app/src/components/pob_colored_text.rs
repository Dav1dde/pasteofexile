use sycamore::prelude::*;

use crate::{
    pob::formatting::{Color, ColoredText},
    utils::{
        links::{Link, Links},
        IteratorExt,
    },
};

#[derive(Prop)]
pub struct Props<'a> {
    text: &'a str,
    links: bool,
}

#[component]
pub fn PobColoredText<G: Html>(cx: Scope, props: Props<'_>) -> View<G> {
    // TODO: sycamore - bug in scamore that scope lifetime cant be tied to text (&str)
    ColoredText::new(props.text)
        .map(|cs| render_fragment(cx, cs, props.links))
        .collect_view()
}

#[allow(clippy::enum_variant_names)]
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

fn render_fragment<G: Html>(cx: Scope, (color, text): (Color, &str), links: bool) -> View<G> {
    let text = render_text(cx, text, links);
    match color_to_style(color) {
        Style::Class(class) => view! { cx, span(class=class) { (text) } },
        Style::Style(style) => view! { cx, span(style=style) { (text) } },
        Style::None => view! { cx, span { (text) } },
    }
}

fn render_text<G: Html>(cx: Scope, text: &str, links: bool) -> View<G> {
    if !links {
        let text = text.to_owned();
        return view! { cx, (text) };
    }

    Links::new(text)
        .map(|part| match part {
            Link::Text(text) => {
                let text = text.to_owned();
                view! { cx, span { (text) } }
            }
            Link::Link(link) => {
                let link = link.to_owned();
                let text = link.clone();
                view! { cx, a(href=link, class="underline", target="_blank") { (text) } }
            }
        })
        .collect_view()
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
