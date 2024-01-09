use sycamore::prelude::*;

use crate::utils::IteratorExt;

#[derive(Prop)]
pub struct Props {
    pub name: String,
    pub stats: Vec<String>,
}

#[component]
pub fn TreeNode<G: Html>(cx: Scope<'_>, props: Props) -> View<G> {
    let stats = props
        .stats
        .into_iter()
        .map(|s| view! { cx, div() { (s) } })
        .collect_view();

    view! { cx,
        div(class="bg-black/[0.8] text-center pob-item font-['FontinSmallCaps']", data-rarity="Rare") {
            div(class="px-7 py-1 bg-contain relative text-[1.1875rem] leading-6", style=header_style()) {
                div { (props.name) }
            }
            div(class="p-2 pt-1 text-[#88f]") {
                (stats)
            }
        }
    }
}

fn header_style() -> String {
    let name = "Rare";
    let color = "#ff7";

    const BASE: &str = "https://assets.pobb.in/1/Art/2DArt/UIImages/InGame/ItemsHeader";

    format!(
        "color: {color}; background: \
        url({BASE}{name}Left.webp) top left / contain no-repeat, \
        url({BASE}{name}Right.webp) top right / contain no-repeat, \
        url({BASE}{name}Middle.webp) top left / contain repeat-x"
    )
}
