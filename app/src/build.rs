use std::convert::TryFrom;

use pob::{PathOfBuilding, SerdePathOfBuilding, TreeSpec};
use shared::model::data;
use sycamore::reactive::{create_rc_signal, RcSignal};

pub struct Build {
    // required because access through method does break sycamore
    pub content: String,
    pob: SerdePathOfBuilding,
    data: data::Data,

    active_tree: RcSignal<usize>,
    active_item_set: RcSignal<usize>,
    active_skill_set: RcSignal<usize>,
    loadouts: Vec<pob::Loadout>,
}

impl Build {
    pub fn pob(&self) -> &impl PathOfBuilding {
        &self.pob
    }

    pub fn data(&self) -> &data::Data {
        &self.data
    }

    pub fn active_tree(&self) -> &RcSignal<usize> {
        &self.active_tree
    }

    pub fn active_item_set(&self) -> &RcSignal<usize> {
        &self.active_item_set
    }

    pub fn active_skill_set(&self) -> &RcSignal<usize> {
        &self.active_skill_set
    }

    pub fn loadouts(&self) -> &[pob::Loadout] {
        &self.loadouts
    }

    pub fn selected_loadout_index(&self) -> Option<usize> {
        let tree = *self.active_tree.get();
        let item_set = *self.active_item_set.get();
        let skill_set = *self.active_skill_set.get();

        self.loadouts.iter().position(|loadout| {
            loadout.tree_index == tree
                && loadout.item_set_index == item_set
                && loadout.skill_set_index == skill_set
        })
    }

    pub fn select_loadout(&self, index: usize) {
        let Some(loadout) = self.loadouts.get(index) else {
            return;
        };

        self.active_tree.set(loadout.tree_index);
        self.active_item_set.set(loadout.item_set_index);
        self.active_skill_set.set(loadout.skill_set_index);
    }
}

impl Build {
    // TODO: this needs a rewrite, accepting additional data from /json is awkward
    pub fn new(content: String, data: data::Data) -> crate::Result<Self> {
        let pob = SerdePathOfBuilding::from_export(&content)?;

        let active_tree = pob
            .tree_specs()
            .iter()
            .position(|spec| spec.active)
            .unwrap_or(0);
        let active_item_set = pob
            .item_sets()
            .iter()
            .position(|set| set.is_selected)
            .unwrap_or(0);
        let active_skill_set = pob
            .skill_sets()
            .iter()
            .position(|set| set.is_selected)
            .unwrap_or(0);
        let loadouts = pob.loadouts();

        Ok(Self {
            content,
            pob,
            data,
            active_tree: create_rc_signal(active_tree),
            active_item_set: create_rc_signal(active_item_set),
            active_skill_set: create_rc_signal(active_skill_set),
            loadouts,
        })
    }

    pub fn trees(&self) -> impl Iterator<Item = (&data::Nodes, TreeSpec<'_>)> {
        static DEFAULT_NODES: data::Nodes = data::Nodes {
            keystones: Vec::new(),
            masteries: Vec::new(),
        };

        let nodes = self
            .data
            .nodes
            .iter()
            .chain(std::iter::repeat(&DEFAULT_NODES));
        let specs = self.pob.tree_specs().into_iter();

        std::iter::zip(nodes, specs)
    }
}

impl std::ops::Deref for Build {
    type Target = SerdePathOfBuilding;

    fn deref(&self) -> &Self::Target {
        &self.pob
    }
}

impl TryFrom<crate::context::Paste> for Build {
    type Error = crate::Error;

    fn try_from(paste: crate::context::Paste) -> Result<Self, Self::Error> {
        Self::new(paste.content, paste.data)
    }
}

impl TryFrom<shared::model::Paste> for Build {
    type Error = crate::Error;

    fn try_from(paste: shared::model::Paste) -> Result<Self, Self::Error> {
        Self::new(paste.content, paste.data)
    }
}
