use itertools::Itertools;
use pob::{PathOfBuilding, Skill};
use sycamore::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlSelectElement};

use crate::{build::Build, components::PobColoredText, pob::formatting::strip_colors};

#[component(PobGems<G>)]
pub fn pob_gems(build: Build) -> View<G> {
    let mut skill_sets = build.skill_sets();

    if skill_sets.is_empty() {
        return view! { div() { } };
    } else if skill_sets.len() == 1 {
        let skills = render_skills(skill_sets.remove(0).skills);
        return view! {
            div(class="columns-[13rem] gap-5 sm:ml-3 leading-[1.35rem]") {
                (skills)
            }
        };
    }

    let content = Signal::new(view! {});

    let mut select = Vec::new();
    for ss in skill_sets.iter() {
        let id = ss.id;
        let selected = ss.is_selected;
        let title = ss
            .title
            .map(|s| s.to_owned())
            .unwrap_or_else(|| id.to_string());
        let view = view! { option(value=id, selected=selected) { span { (title) } } };
        select.push(view)
    }
    let select = View::new_fragment(select);

    let on_input = cloned!((build, content) => move |event: Event| {
        let id = event.target().unwrap().unchecked_into::<HtmlSelectElement>()
            .value().parse::<u16>().unwrap_or(u16::MAX);

        if let Some(ss) = build.skill_sets().into_iter().find(|ss| ss.id == id) {
            content.set(render_skills(ss.skills));
        }
    });

    if let Some(ss) = skill_sets.into_iter().find(|ss| ss.is_selected) {
        content.set(render_skills(ss.skills));
    }

    view! {
        select(class="sm:ml-3 mt-1 mb-2 px-1", on:input=on_input) { (select) }
        div(class="columns-[13rem] gap-5 sm:ml-3 leading-[1.35rem]") {
            div() {
            (&*content.get())
            }
        }
    }
}

fn render_skills<G: GenericNode + Html>(skills: Vec<Skill>) -> View<G> {
    let iter_skills = skills
        .into_iter()
        .filter(is_enabled)
        .filter(|s| !is_enchant(s));

    let mut skills = Vec::new();
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

    View::new_fragment(skills)
}

fn is_enabled(skill: &Skill) -> bool {
    // still show selected skills even if they are disabled
    if skill.is_selected {
        return true;
    }

    // Keep disabled gems, people have multiple setups
    // for trade, ssf, etc. and some of these are disabled
    // if !skill.is_enabled { return false; }

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
        .filter(|gem| gem.is_enabled)
        .with_position()
        .map(|gem| {
            let is_only = matches!(gem, itertools::Position::Only(_));
            let is_first = matches!(gem, itertools::Position::First(_));
            let is_last = matches!(gem, itertools::Position::Last(_));
            let gem = gem.into_inner();

            let name = gem.name.to_owned();
            let class = match (gem.is_selected, gem.is_active) {
                (true, _) => "truncate font-bold dark:text-amber-50 text-slate-800",
                (_, true) => "truncate dark:text-stone-100 text-slate-800",
                (false, false) => {
                    if is_only {
                        "truncate"
                    } else if is_first {
                        "truncate gem-first"
                    } else if is_last {
                        "truncate gem-last"
                    } else {
                        "truncate gem-middle"
                    }
                }
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
