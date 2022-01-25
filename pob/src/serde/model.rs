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

    #[serde(default, rename = "Notes")]
    pub notes: String,

    #[serde(default, rename = "Config")]
    pub config: Config,

    #[serde(rename = "Items")]
    pub items: Items,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Items {
    #[serde(rename = "Item")]
    pub item: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Build {
    pub level: u8,
    pub class_name: String,
    pub ascend_class_name: String,
    #[serde(default, rename = "PlayerStat")]
    pub player_stats: Vec<BuildStat>,
    #[serde(default, rename = "MinionStat")]
    pub minion_stats: Vec<BuildStat>,
    pub main_socket_group: u8,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BuildStat {
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
    #[serde(default, deserialize_with = "u8_or_nil")]
    pub main_active_skill: u8,
    #[serde(default, rename = "$value")]
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
    pub skill_id: Option<String>,
    pub gem_id: Option<String>,
}

impl Gem {
    pub fn is_support(&self) -> bool {
        if let Some(gem_id) = &self.gem_id {
            return gem_id.starts_with("Metadata/Items/Gems/Support");
        }
        if let Some(skill_id) = &self.skill_id {
            return skill_id.starts_with("Support");
        }
        self.name.contains("Support")
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
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default, with = "StringWithSeparator::<CommaSeparator>")]
    pub nodes: Vec<u32>,
    #[serde(default, rename = "URL")]
    pub url: Option<String>,
}

#[derive(Default, Debug, Deserialize)]
pub(crate) struct Config {
    #[serde(default, rename = "Input")]
    pub input: Vec<Input>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Input {
    pub name: String,
    pub string: Option<String>,
    pub boolean: Option<bool>,
    pub number: Option<f32>,
}
