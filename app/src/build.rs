use std::convert::TryFrom;

use pob::{PathOfBuilding, SerdePathOfBuilding, TreeSpec, TreeSpecId};
use shared::model::data;
use sycamore::reactive::{create_rc_signal, RcSignal};

/// A purely read-only view into a [`PathOfBuilding`] build.
#[derive(Debug)]
pub struct Build {
    // required because access through method does break sycamore
    pub content: String,
    pob: SerdePathOfBuilding,
    data: data::Data,

    /// Currently actively displayed loadout.
    ///
    /// Note: this does not represent an actually existing loadout in the build files,
    /// it is simply the combination of tree, items and skills currently actively being displayed.
    ///
    /// This set of data being displayed may happen to match a build defined loadout.
    active_loadout: Loadout,
}

impl Build {
    pub fn pob(&self) -> &impl PathOfBuilding {
        &self.pob
    }

    pub fn data(&self) -> &data::Data {
        &self.data
    }
}

impl Build {
    // TODO: this needs a rewrite, accepting additional data from /json is awkward
    pub fn new(content: String, data: data::Data) -> crate::Result<Self> {
        let pob = SerdePathOfBuilding::from_export(&content)?;

        let active_tree = {
            let specs = pob.tree_specs();
            specs
                .iter()
                .find(|s| s.active)
                .or(specs.first())
                .map(|s| s.id)
        };

        Ok(Self {
            content,
            pob,
            data,
            active_loadout: Loadout {
                tree: create_rc_signal(active_tree),
            },
        })
    }
}

impl Build {
    pub fn set_active_tree(&self, id: TreeSpecId) {
        self.active_loadout.tree.set(Some(id));
    }

    pub fn active_tree_id(&self) -> Option<TreeSpecId> {
        *self.active_loadout.tree.get()
    }

    pub fn active_tree<'a>(&'a self) -> Option<TreeSpecWithNodes<'a>> {
        static DEFAULT_NODES: data::Nodes = data::Nodes {
            keystones: Vec::new(),
            masteries: Vec::new(),
        };

        let id = (*self.active_loadout.tree.get())?;
        self.pob
            .tree_specs()
            .into_iter()
            .enumerate()
            .find(|(_, t)| t.id == id)
            .map(|(index, spec)| TreeSpecWithNodes {
                spec,
                nodes: self.data.nodes.get(index).unwrap_or(&DEFAULT_NODES),
            })
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

/// An arbitrary combination of a Tree, Items and Skills.
///
/// A loadout can be show in the UI and loaded from an existing set from the build file.
#[derive(Debug)]
struct Loadout {
    tree: RcSignal<Option<TreeSpecId>>,
}

#[derive(Debug)]
pub struct TreeSpecWithNodes<'a> {
    pub spec: TreeSpec<'a>,
    pub nodes: &'a data::Nodes,
}
