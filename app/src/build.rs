use std::{convert::TryFrom, rc::Rc};

use pob::{PathOfBuilding, SerdePathOfBuilding, TreeSpec};
use shared::model::Nodes;

#[derive(Clone)]
pub struct Build(Rc<Inner>);

impl Build {
    pub fn pob(&self) -> &impl PathOfBuilding {
        &self.0.pob
    }

    pub fn content(&self) -> &str {
        &self.0.content
    }

    pub fn nodes(&self) -> &[Nodes] {
        &self.0.nodes
    }
}

impl Build {
    pub fn new(content: String, nodes: Vec<Nodes>) -> crate::Result<Self> {
        let pob = SerdePathOfBuilding::from_export(&content)?;

        let inner = Inner {
            content,
            pob,
            nodes,
        };

        Ok(Self(Rc::new(inner)))
    }

    pub fn trees(&self) -> impl Iterator<Item = (&Nodes, TreeSpec)> {
        static DEFAULT_NODES: Nodes = Nodes {
            keystones: Vec::new(),
            masteries: Vec::new(),
        };

        let nodes = self.0.nodes.iter().chain(std::iter::repeat(&DEFAULT_NODES));
        let specs = self.0.pob.tree_specs().into_iter();

        std::iter::zip(nodes, specs)
    }
}

impl std::ops::Deref for Build {
    type Target = SerdePathOfBuilding;

    fn deref(&self) -> &Self::Target {
        &self.0.pob
    }
}

impl TryFrom<crate::context::Paste> for Build {
    type Error = crate::Error;

    fn try_from(paste: crate::context::Paste) -> Result<Self, Self::Error> {
        let pob = SerdePathOfBuilding::from_export(&paste.content)?;

        let inner = Inner {
            content: paste.content,
            pob,
            nodes: paste.nodes,
        };

        Ok(Self(Rc::new(inner)))
    }
}

impl TryFrom<shared::model::Paste> for Build {
    type Error = crate::Error;

    fn try_from(paste: shared::model::Paste) -> Result<Self, Self::Error> {
        let pob = SerdePathOfBuilding::from_export(&paste.content)?;

        let inner = Inner {
            content: paste.content,
            pob,
            nodes: paste.nodes,
        };

        Ok(Self(Rc::new(inner)))
    }
}

struct Inner {
    content: String,
    pob: SerdePathOfBuilding,
    nodes: Vec<Nodes>,
}
