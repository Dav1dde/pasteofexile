use serde::{Deserialize, Serialize};

mod paste;

pub use paste::*;

#[derive(Debug)]
pub struct ListPaste {
    pub name: String, // TODO: this should be a PasteId I think
    pub metadata: Option<PasteMetadata>,
    pub last_modified: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Paste {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<PasteMetadata>,
    #[serde(default, skip_serializing_if = "crate::utils::is_zero")]
    pub last_modified: u64,
    pub content: String,
    /// A list of node description to display per tree spec.
    ///
    /// List is in the same order as the tree specs.
    #[serde(default)]
    pub nodes: Vec<Nodes>,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct Nodes {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keystones: Vec<Node>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub masteries: Vec<Node>,
}

impl Nodes {
    pub fn is_empty(&self) -> bool {
        self.keystones.is_empty() && self.masteries.is_empty()
    }
}

#[derive(Default, Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Node {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stats: Vec<String>,
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct PasteMetadata {
    pub title: String,
    pub ascendancy_or_class: String, // TODO: this should be an enum
    pub version: Option<String>,
    pub main_skill_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PasteSummary {
    pub id: PasteId,
    pub title: String,
    pub ascendancy_or_class: String, // TODO: this should be an enum
    pub version: Option<String>,
    pub main_skill_name: Option<String>,
    pub last_modified: u64,
}

impl PasteSummary {
    pub fn to_url(&self) -> String {
        self.id.to_url()
    }
}
