use crate::serde::utils::u8_or_nil;
use serde::Deserialize;
use serde_with::{rust::StringWithSeparator, CommaSeparator};

#[derive(Debug, Deserialize)]
pub(crate) struct PathOfBuilding {
    #[serde(rename = "Build")]
    pub build: Build,

    #[serde(rename = "Skills")]
    pub skills: Skills,

    #[serde(rename = "Tree")]
    pub tree: Tree,

    #[serde(rename = "Notes")]
    pub notes: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Build {
    pub level: u8,
    pub class_name: String,
    pub ascend_class_name: String,
    #[serde(rename = "$value")]
    pub stats: Vec<PlayerStat>,
    pub main_socket_group: u8,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PlayerStat {
    #[serde(rename = "stat")]
    pub name: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Skills {
    #[serde(default, rename = "$value")]
    pub skills: Vec<Skill>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Skill {
    #[serde(deserialize_with = "u8_or_nil")]
    pub main_active_skill: u8, // can be "nil"
    #[serde(rename = "$value")]
    pub gems: Vec<Gem>,
}

impl Skill {
    pub fn active_gems(&self) -> impl Iterator<Item = &Gem> {
        self.gems.iter().filter(|gem| gem.is_active())
    }

    pub fn support_gems(&self) -> impl Iterator<Item = &Gem> {
        self.gems.iter().filter(|gem| gem.is_support())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Gem {
    #[serde(rename = "nameSpec")]
    pub name: String,
    pub gem_id: String,
}

impl Gem {
    pub fn is_support(&self) -> bool {
        self.gem_id.starts_with("Metadata/Items/Gems/Support")
    }

    pub fn is_active(&self) -> bool {
        !self.is_support()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Tree {
    pub active_spec: u8,
    #[serde(rename = "Spec")]
    pub specs: Vec<Spec>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Spec {
    // #[serde(default)]
    // title: Option<String>,
    #[serde(default, with = "StringWithSeparator::<CommaSeparator>")]
    pub nodes: Vec<u32>,
    // #[serde(rename = "URL")]
    // url: String,
}
