use crate::components::PobColoredText;
use itertools::Itertools;
use pob::{PathOfBuilding, SerdePathOfBuilding, Skill};
use std::rc::Rc;
use sycamore::prelude::*;

#[component(PobGems<G>)]
pub fn pob_gems(pob: Rc<SerdePathOfBuilding>) -> View<G> {
    let mut skills = Vec::new();

    // TODO: text escape for pob escape codes
    for (key, group) in &pob.skills().into_iter().group_by(|s| s.gems.is_empty()) {
        if key {
            // it's a bunched up group of labels
            let labels = group
                .filter(|s| s.label.is_some())
                .map(|s| s.label.unwrap().to_owned())
                .map(|label| view! { div(class="truncate") { PobColoredText(label) } })
                .collect();
            let labels = View::new_fragment(labels);
            skills.push(view! {
                div(class="break-inside-avoid leading-4 mt-5 first:mt-[0.5rem]") {
                    (labels)
                }
            });
        } else {
            // a bunch of skills with gems
            skills.extend(group.map(render_skill));
        }
    }

    let skills = View::new_fragment(skills);

    view! {
        div(class="columns-[13rem] gap-5 sm:ml-3") {
            (skills)
        }
    }
}

fn render_skill<G: GenericNode>(skill: Skill) -> View<G> {
    let gems = skill
        .gems
        .into_iter()
        .map(|gem| {
            let name = gem.name.to_owned();
            let class = match (gem.is_selected, gem.is_active) {
                (true, _) => "font-medium dark:text-amber-50 text-slate-800",
                (_, true) => "text-stone-200",
                _ => "",
            };

            view! { div(class=class) { (name) } }
        })
        .collect::<Vec<View<G>>>();
    let gems = View::new_fragment(gems);

    let class = "flex flex-col break-inside-avoid mt-5 first:mt-0";

    view! {
        div(class=class) {
            (gems)
        }
    }
}
