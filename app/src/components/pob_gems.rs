use itertools::Itertools;
use pob::{PathOfBuilding, Skill};
use shared::{model::data, Color};
use sycamore::prelude::*;

use crate::{
    build::Build,
    components::{PobColoredSelect, PobColoredText},
    pob::formatting::strip_colors,
    svg,
    utils::IteratorExt,
};

#[component]
pub fn PobGems<'a, G: Html>(cx: Scope<'a>, build: &'a Build) -> View<G> {
    let mut skill_sets = build.skill_sets();

    if skill_sets.is_empty() {
        return view! { cx, div() { "No Skill Gems" } };
    } else if skill_sets.len() == 1 {
        let skills = render_skills(cx, skill_sets.remove(0).skills, build.data());
        return view! { cx,
            div(class="columns-2xs gap-5 sm:ml-3 mt-5 leading-[1.35rem]") {
                (skills)
            }
        };
    }

    let content = create_signal(cx, view! { cx, });

    let options = skill_sets
        .iter()
        .map(|ss| {
            ss.title
                .map(|s| s.to_owned())
                .unwrap_or_else(|| ss.id.to_string())
        })
        .collect();
    let selected = skill_sets.iter().position(|ss| ss.is_selected);
    let on_change = move |index| {
        let Some(index) = index else { return };
        if let Some(ss) = build.skill_sets().into_iter().nth(index) {
            content.set(render_skills::<G>(cx, ss.skills, build.data()));
        }
    };

    if let Some(ss) = skill_sets.into_iter().find(|ss| ss.is_selected) {
        content.set(render_skills(cx, ss.skills, build.data()));
    }

    view! { cx,
        PobColoredSelect(options=options, selected=selected, on_change=on_change)

        div(class="columns-2xs gap-5 sm:ml-3 leading-[1.35rem]") {
            div() { (&*content.get()) }
        }
    }
}

fn render_skills<'a, G: GenericNode + Html>(
    cx: Scope<'a>,
    skills: Vec<Skill<'a>>,
    data: &'a data::Data,
) -> View<G> {
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
                    let title = create_ref(cx, strip_colors(&label));
                    view! { cx, div(class="truncate", title=title) { PobColoredText(text=&label, links=false) } }
                })
                .collect_view();

            let class = "break-inside-avoid leading-4 mt-5 first:mt-[0.5rem]";
            skills.push(view! { cx, div(class=class) { (labels) } });
        } else {
            // a bunch of skills with gems
            skills.extend(
                group
                    .filter(has_active_gem)
                    .map(|skill| render_skill(cx, skill, data)),
            );
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

fn render_skill<'a, G: Html>(cx: Scope<'a>, skill: Skill<'a>, data: &'a data::Data) -> View<G> {
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

            let data = gem.gem_id.and_then(|gem_id| data.gems.get(gem_id));

            // This could be empty for skills from uniques (see also `pob/src/gems.rs`),
            // but PoB has a workaround so this shouldn't be empty.
            // Rather add more uniques to the existing workaround then adding another here.
            //
            // Fallback to skill_id, works for `Purity` and maybe other things ...
            // better than just having it silently disappear.
            let name = Some(gem.name)
                .filter(|name| !name.is_empty())
                .or(gem.skill_id)
                .unwrap_or("<unknown>")
                .to_owned();

            let quality = match gem.quality_id {
                Some("Alternate1") => "Anomalous ",
                Some("Alternate2") => "Divergent ",
                Some("Alternate3") => "Phantasmal ",
                _ => "",
            };

            let mut color = "";
            let mut bold = false;
            let mut gem_position = "";

            match (gem.is_selected, gem.is_active) {
                (true, _) => {
                    bold = true;
                    color = "text-amber-50";
                }
                (_, true) => color = "text-stone-100",
                (false, false) => {
                    if is_only {
                    } else if is_first {
                        gem_position = "gem-first";
                    } else if is_last {
                        gem_position = "gem-last";
                    } else {
                        gem_position = "gem-middle";
                    }
                }
            };

            let color = match data.map(|data| data.color) {
                Some(Color::Red) => "text-rose-500",
                Some(Color::Green) => "text-lime-400",
                Some(Color::Blue) => "text-blue-400",
                Some(Color::White) => "text-slate-50",
                None => color,
            };

            let class = [
                "truncate",
                if bold { "font-bold" } else { "" },
                color,
                gem_position,
            ]
            .join(" ");

            let name = format!("{quality}{name}");
            let title = format!("{name} ({}/{})", gem.level, gem.quality);
            view! { cx, div(class=class, title=title) { (name) } }
        })
        .collect_vec();

    if gems.is_empty() {
        return view! { cx, div() {} };
    }
    let gems = View::new_fragment(gems);

    let svg = match skill.slot {
        Some("Weapon 1") => svg::ICON_WEAPON,
        Some("Weapon 2") => svg::ICON_WEAPON,
        Some("Weapon 1 Swap") => svg::ICON_WEAPON,
        Some("Weapon 2 Swap") => svg::ICON_WEAPON,
        Some("Bow") => svg::ICON_BOW,
        Some("Quiver") => svg::ICON_QUIVER,
        Some("Shield") => svg::ICON_SHIELD,
        Some("Shield Swap") => svg::ICON_SHIELD,
        Some("Helmet") => svg::ICON_HELMET,
        Some("Body Armour") => svg::ICON_BODY_ARMOUR,
        Some("Gloves") => svg::ICON_GLOVES,
        Some("Boots") => svg::ICON_BOOTS,
        Some("Amulet") => svg::ICON_AMULET,
        Some("Ring 1") => svg::ICON_RING,
        Some("Ring 2") => svg::ICON_RING,
        Some("Belt") => svg::ICON_BELT,
        _ => "",
    };

    let slot = skill.slot.unwrap_or("");

    view! { cx,
        div(class="break-inside-avoid mt-5 first:mt-0 bg-slate-900 px-5 py-2.5 rounded-xl") {
            div(dangerously_set_inner_html=svg, data-slot=slot, class="float-right w-6") {}
            (gems)
        }
    }
}
