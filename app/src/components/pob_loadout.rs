use pob::PathOfBuilding;
use sycamore::prelude::*;

use super::PobColoredSelect;
use crate::{build::Build, components::pob_colored_select::SelectItem};

#[component]
pub fn PobLoadout<'a, G: Html>(cx: Scope<'a>, build: &'a Build) -> View<G> {
    let mut loadouts = vec![Loadout {
        name: "Custom".to_owned(),
        loadout: None,
    }];

    loadouts.extend(build.pob().loadouts().into_iter().map(|loadout| Loadout {
        name: get_name(build, loadout),
        loadout: Some(loadout),
    }));

    let on_change = |id: Option<Option<pob::Loadout>>| {
        if let Some(loadout) = id.and_then(|t| t) {
            build.set_active_tree(loadout.tree);
            build.set_current_item_set(loadout.item_set);
            build.set_current_skill_set(loadout.skill_set);
        }
    };

    let raw_loadouts = build.pob().loadouts();
    let selected = create_selector(cx, move || {
        let tree = build.active_tree_id();
        let item_set = build.current_item_set_id();
        let skill_set = build.current_skill_set_id();

        Some(match (tree, item_set, skill_set) {
            (Some(tree), Some(item_set), Some(skill_set)) => Some(pob::Loadout {
                tree,
                item_set,
                skill_set,
            })
            .filter(|loadout| raw_loadouts.contains(loadout)),
            _ => None,
        })
    });

    view! { cx,
        PobColoredSelect(options=loadouts, selected=selected, label="Select loadout", on_change=on_change)
    }
}

fn get_name(build: &Build, loadout: pob::Loadout) -> String {
    if let Some(title) = build.tree_spec_by_id(loadout.tree).and_then(|t| t.title) {
        return title.to_owned();
    }

    "Default".to_owned()
}

pub struct Loadout {
    name: String,
    /// The actual selection.
    ///
    /// Might be `None` for the `Custom` loadout.
    loadout: Option<pob::Loadout>,
}

impl SelectItem for Loadout {
    type Id = Option<pob::Loadout>;

    fn id(&self) -> Self::Id {
        self.loadout
    }

    fn render(&self) -> String {
        self.name.clone()
    }
}
