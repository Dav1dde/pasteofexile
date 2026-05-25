use pob::{ItemSet, ItemSetId, SkillSet, SkillSetId, TreeSpec, TreeSpecId};
use sycamore::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlSelectElement;

use super::pob_colored_text::{color_to_style, Style};
use crate::{consts::SELECT_ONCHANGE_COLOR_FROM_OPTION, pob::formatting::only_first_color, utils};

pub trait SelectItem {
    type Id: Copy + PartialEq + 'static;

    fn id(&self) -> Self::Id;
    fn render(&self) -> String;
}

#[derive(Prop)]
pub struct PobColoredSelectProps<'a, T: SelectItem, F> {
    pub options: Vec<T>,
    pub selected: &'a ReadSignal<Option<T::Id>>,
    pub label: &'static str,
    pub on_change: F,
}

#[component]
pub fn PobColoredSelect<'a, G: Html, T, F>(
    cx: Scope<'a>,
    props: PobColoredSelectProps<'a, T, F>,
) -> View<G>
where
    T: SelectItem + 'a,
    F: Fn(Option<T::Id>) + 'a,
{
    let raw_options = create_ref(cx, props.options);

    let mut start_style = Style::None;
    let mut options = Vec::new();
    for (i, item) in raw_options.iter().enumerate() {
        let content = item.render();
        let (color, content) = only_first_color(&content);

        let selected = Some(item.id()) == *props.selected.get();

        if selected {
            start_style = color_to_style(color);
        }

        let v = match color_to_style(color) {
            Style::Class(class) => {
                view! { cx, option(selected=selected, value=i, class=class) { (content) } }
            }
            Style::Style(style) => {
                view! { cx, option(selected=selected, value=i, style=style) { (content) } }
            }
            Style::None => {
                // Use "default" color here to make sure
                // the option doesnt inherit the color from the select.
                view! { cx, option(selected=selected, value=i, class="text-slate-300") { (content) } }
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

        let index = element.selected_index().try_into().ok();
        let id = index
            .and_then(|index: usize| raw_options.get(index))
            .map(|item| item.id());
        (props.on_change)(id);
    };

    let (class, style) = match start_style {
        Style::Class(class) => (class, String::new()),
        Style::Style(style) => ("", style),
        Style::None => ("", String::new()),
    };
    let class = format!("sm:ml-3 mt-1 mb-2 px-1 max-w-full {class}");

    let select = create_node_ref(cx);

    create_effect(cx, move || {
        let selected = *props.selected.get();
        let idx = raw_options
            .iter()
            .position(|item| Some(item.id()) == selected);

        if let Some(idx) = idx {
            if let Some(select) = utils::try_from_ref::<web_sys::HtmlSelectElement>(select) {
                select.set_value(&idx.to_string());
            }
        }
    });

    view! { cx,
        select(
            ref=select,
            class=class,
            style=style,
            aria-label=props.label,
            on:input=on_input,
            onchange=SELECT_ONCHANGE_COLOR_FROM_OPTION,
            autocomplete="off",
        ) { (options) }
    }
}

impl SelectItem for SkillSet<'_> {
    type Id = SkillSetId;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn render(&self) -> String {
        self.title
            .map(|s| s.to_owned())
            .unwrap_or_else(|| self.id.0.to_string())
    }
}

impl SelectItem for ItemSet<'_> {
    type Id = ItemSetId;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn render(&self) -> String {
        self.title
            .map(|s| s.to_owned())
            .unwrap_or_else(|| self.id.0.to_string())
    }
}

impl SelectItem for TreeSpec<'_> {
    type Id = TreeSpecId;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn render(&self) -> String {
        self.title.unwrap_or("<Default>").to_owned()
    }
}
