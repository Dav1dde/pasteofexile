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

        Ok(Self {
            content,
            pob,
            data,
            active_tree: create_rc_signal(active_tree),
        })
    }

    pub fn trees(&self) -> impl Iterator<Item = (&data::Nodes, TreeSpec)> {
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
