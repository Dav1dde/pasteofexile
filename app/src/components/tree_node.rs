use sycamore::prelude::*;

use crate::utils::IteratorExt;

#[derive(Prop)]
pub struct Props {
    pub name: String,
    pub stats: Vec<String>,
    pub kind: Option<String>,
}

#[component]
pub fn TreeNode<G: Html>(cx: Scope<'_>, props: Props) -> View<G> {
    let stats = props
        .stats
        .into_iter()
        .map(|s| view! { cx, div() { (s) } })
        .collect_view();

    let header_style = header_style(props.kind);

    view! { cx,
        div(class="bg-black/[0.8] text-center pob-item font-['FontinSmallCaps']", data-rarity="Rare") {
            div(class="px-9 py-2.5 bg-contain relative text-[1.1875rem] leading-6", style=header_style) {
                div { (props.name) }
            }
            div(class="p-2 pt-1 text-[#88f] whitespace-pre-line leading-5") {
                (stats)
            }
        }
    }
}

fn header_style(kind: Option<String>) -> String {
    const BASE: &str = "https://assets.pobb.in/1/Art/2DArt/UIImages/InGame/";

    let name = match kind.as_deref() {
        Some("Mastery") => "PassiveMastery/Mastery",
        Some(name) => name,
        None => "Normal",
    };

    format!(
        "color: #efe492; background: \
        url({BASE}{name}PassiveHeaderLeft.webp) top left / contain no-repeat, \
        url({BASE}{name}PassiveHeaderRight.webp) top right / contain no-repeat, \
        url({BASE}{name}PassiveHeaderMiddle.webp) top left / contain repeat-x"
    )
}
