use itertools::Itertools;
use pob::{PathOfBuilding, Skill};
use shared::{model::data, Color};
use sycamore::prelude::*;
use wasm_bindgen::JsCast;

use crate::{
    build::Build,
    components::{PobColoredSelect, PobColoredText, Popup},
    consts,
    pob::formatting::strip_colors,
    svg,
    utils::{open_wiki_page, IteratorExt},
};

#[component]
pub fn PobGems<'a, G: Html>(cx: Scope<'a>, build: &'a Build) -> View<G> {
    let skill_sets = build.skill_sets();

    if skill_sets.is_empty() {
        return view! { cx, div() { "No Skill Gems" } };
    }
    let show_select = skill_sets.len() > 1;

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

    let attach = create_signal(cx, None);
    let popup = create_signal(cx, View::default());

    let mouseover = move |event: web_sys::Event| {
        let target = event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::Element>().ok());

        let gem = target.as_ref().and_then(PopupGem::from_element);
        let gem_data = gem.as_ref().and_then(|gem| build.data().gems.get(&gem.id));

        if let (Some(gem), Some(gem_data)) = (gem, gem_data) {
            popup.set(render_popup(cx, gem, gem_data));
            attach.set(target);
        } else {
            attach.set(None);
        }
    };
    let mouseout = |_: web_sys::Event| attach.set(None);

    let select = match show_select {
        true => {
            view! { cx, PobColoredSelect(options=options, selected=selected, label="Select skill set", on_change=on_change) }
        }
        false => View::default(),
    };

    view! { cx,
        (select)

        Popup(attach=attach) { (&*popup.get()) }
        div(class="columns-2xs gap-5 sm:ml-3 leading-[1.35rem]") {
            div(on:mouseover=mouseover, on:mouseout=mouseout) { (&*content.get()) }
        }
    }
}

#[derive(Debug)]
struct PopupGem {
    id: String,
    name: String,
    quality: u8,
    level: u8,
}

impl PopupGem {
    fn from_element(element: &web_sys::Element) -> Option<Self> {
        let id = element
            .get_attribute("data-gem-id")
            .filter(|id| !id.is_empty())?;
        let name = element.text_content().filter(|name| !name.is_empty())?;
        let level = element
            .get_attribute("data-gem-level")
            .and_then(|s| s.parse().ok())?;
        let quality = element
            .get_attribute("data-gem-quality")
            .and_then(|s| s.parse().ok())?;

        Some(Self {
            id,
            name,
            level,
            quality,
        })
    }
}

fn render_popup<'a, G: GenericNode + Html>(
    cx: Scope<'a>,
    gem: PopupGem,
    data: &'a data::Gem,
) -> View<G> {
    let gem_src = crate::assets::item_image_url(&gem.id);

    let vendors = data
        .vendors
        .iter()
        .map(|vendor| {
            view! { cx,
                div(class="whitespace-nowrap") { "Act " (vendor.act) }
                div() { (vendor.npc) }
                div() { (vendor.quest) }
            }
        })
        .collect_view();

    view! { cx,
        div(class="bg-black/[0.8] font-['FontinSmallCaps'] py-2 px-4 flex flex-col gap-3") {
            div(class="flex items-center gap-10") {
                img(src=gem_src,
                    class="h-10 w-10",
                    onerror=consts::IMG_ONERROR_HIDDEN) {}
                div(class="flex-auto text-center min-w-48") {
                    div(class=gem_color(data.color)) { (gem.name) }
                    div(class="text-center mt-0.5", style="color: #7f7f7f") { "Minimum Level: " (data.level) }
                }
                div(class=gem_color(data.color)) { (gem.level) "/" (gem.quality) }
            }

            div(class="grid grid-cols-[min-content_auto_auto] gap-x-6 md:gap-x-10 gap-y-1 md:mt-1 empty:hidden") {
                (vendors)
            }

            div(class="text-right text-xs italic hidden has-mouse:block", style="color: #7f7f7f") {
                "Wiki: Ctrl+Click"
            }
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

            let open_wiki = move |event: web_sys::Event| {
                let has_ctrl = event
                    .dyn_into::<web_sys::MouseEvent>()
                    .map_or(false, |event| event.ctrl_key());

                if !has_ctrl {
                    return;
                }

                if let Some(data) = data {
                    open_wiki_page(&data.name);
                }
            };

            // This could be empty for skills from uniques (see also `pob/src/gems.rs`),
            // but PoB has a workaround so this shouldn't be empty.
            // Rather add more uniques to the existing workaround then adding another here.
            //
            // Fallback to skill_id, works for `Purity` and maybe other things ...
            // better than just having it silently disappear.
            //
            // After having gem data here, we could use `data.name`, but this includes
            // a ` Support` suffix, which does not match PoB, it also is quite long and
            // unnecessary. Maybe fall back in the future if it is necessary.
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

            let color = data.map_or(color, |data| gem_color(data.color));

            let class = [
                "truncate",
                if bold { "font-bold" } else { "" },
                color,
                gem_position,
            ]
            .join(" ");

            let name = format!("{quality}{name}");
            let gem_id = gem.gem_id.unwrap_or("");
            view! { cx,
                div(class=class,
                    on:click=open_wiki,
                    data-gem-id=gem_id,
                    data-gem-level=gem.level,
                    data-gem-quality=gem.quality) { (name) }
            }
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

fn gem_color(color: Color) -> &'static str {
    match color {
        Color::Red => "text-rose-500",
        Color::Green => "text-lime-400",
        Color::Blue => "text-blue-400",
        Color::White => "text-slate-50",
    }
}
