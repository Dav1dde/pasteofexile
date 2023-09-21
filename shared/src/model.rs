use std::num::NonZeroU8;

use serde::{Deserialize, Serialize};

use crate::{AscendancyOrClass, PasteId};

#[derive(Debug)]
pub struct ListPaste {
    pub name: String, // TODO: this should be a PasteId I think
    pub metadata: PasteMetadata,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
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

#[derive(Debug, Deserialize, Serialize)]
pub struct PasteMetadata {
    pub title: String,
    pub ascendancy_or_class: AscendancyOrClass,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_skill_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<NonZeroU8>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub private: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PasteSummary {
    pub id: PasteId,
    pub title: String,
    pub ascendancy_or_class: AscendancyOrClass,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_skill_name: Option<String>,
    pub last_modified: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<NonZeroU8>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub private: bool,
}

impl PasteSummary {
    pub fn to_url(&self) -> String {
        self.id.to_url()
    }
}

fn is_false(v: &bool) -> bool {
    !v
}
