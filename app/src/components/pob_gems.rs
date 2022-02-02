use crate::{components::PobColoredText, pob::formatting::strip_colors};
use itertools::Itertools;
use pob::{PathOfBuilding, SerdePathOfBuilding, Skill};
use std::rc::Rc;
use sycamore::prelude::*;

#[component(PobGems<G>)]
pub fn pob_gems(pob: Rc<SerdePathOfBuilding>) -> View<G> {
    let mut skills = Vec::new();

    let iter_skills = pob
        .skills()
        .into_iter()
        .filter(is_enabled)
        .filter(|s| !is_enchant(s));

    for (key, group) in &iter_skills.group_by(|s| s.gems.is_empty()) {
        if key {
            // it's a bunched up group of labels
            let labels = group
                .filter(|s| s.label.is_some())
                .map(|s| s.label.unwrap().to_owned())
                .map(|label| {
                    let title = strip_colors(&label);
                    view! { div(class="truncate", title=title) { PobColoredText(label) } }
                })
                .collect::<Vec<_>>();

            let class = if labels.len() == 1 {
                "break-inside-avoid leading-4 mt-5 first:mt-[0.5rem] underline"
            } else {
                "break-inside-avoid leading-4 mt-5 first:mt-[0.5rem]"
            };

            let labels = View::new_fragment(labels);
            skills.push(view! { div(class=class) { (labels) } });
        } else {
            // a bunch of skills with gems
            skills.extend(group.filter(has_active_gem).map(render_skill));
        }
    }

    let skills = View::new_fragment(skills);
    view! {
        div(class="columns-[13rem] gap-5 sm:ml-3 leading-[1.35rem]") {
            (skills)
        }
    }
}

fn is_enabled(skill: &Skill) -> bool {
    // still show selected skills even if they are disabled
    if skill.is_selected {
        return true;
    }

    // remove disabled gems
    if !skill.is_enabled {
        return false;
    }

    // remove offhand gems
    if let Some(slot) = skill.slot {
        // TODO: do we need to check here which weapon set is active?
        return slot != "Weapon 1 Swap" && slot != "Weapon 2 Swap";
    }

    true
}

fn is_enchant(skill: &Skill) -> bool {
    skill.gems.len() == 1
        && skill.gems[0]
            .skill_id
            .map(|id| id.starts_with("Enchant"))
            .unwrap_or(false)
}

fn has_active_gem(skill: &Skill) -> bool {
    skill.gems.iter().any(|g| g.is_active)
}

fn render_skill<G: GenericNode>(skill: Skill) -> View<G> {
    let gems = skill
        .gems
        .into_iter()
        .map(|gem| {
            let name = gem.name.to_owned();
            let class = match (gem.is_selected, gem.is_active) {
                (true, _) => "truncate font-bold dark:text-amber-50 text-slate-800",
                (_, true) => "truncate dark:text-stone-100 text-slate-800",
                (false, false) => "truncate before:content-['+_']",
            };

            let title = format!("{} ({}/{})", name, gem.level, gem.quality);
            view! { div(class=class, title=title) { (name) } }
        })
        .collect::<Vec<View<G>>>();
    let gems = View::new_fragment(gems);

    let class = "break-inside-avoid mt-5 first:mt-0";

    view! {
        div(class=class) {
            (gems)
        }
    }
}
