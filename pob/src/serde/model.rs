use crate::serde::utils::u8_or_nil;
use serde::Deserialize;
use serde_with::{rust::StringWithSeparator, CommaSeparator};
use std::borrow::Cow;

#[derive(Debug, Deserialize)]
pub(crate) struct PathOfBuilding<'a> {
    #[serde(borrow, rename = "Build")]
    pub build: Build<'a>,

    #[serde(borrow, rename = "Skills")]
    pub skills: Skills<'a>,

    #[serde(borrow, rename = "Tree")]
    pub tree: Tree<'a>,

    #[serde(borrow, rename = "Notes")]
    pub notes: Cow<'a, str>,

    #[serde(borrow, default, rename = "Config")]
    pub config: Config<'a>,

    #[serde(borrow, rename = "Items")]
    pub items: Items<'a>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Items<'a> {
    #[serde(borrow, rename = "Item")]
    pub item: Vec<Cow<'a, str>>,

}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Build<'a> {
    pub level: u8,
    #[serde(borrow)]
    pub class_name: Cow<'a, str>,
    #[serde(borrow)]
    pub ascend_class_name: Cow<'a, str>,
    #[serde(borrow, default, rename = "PlayerStat")]
    pub player_stats: Vec<BuildStat<'a>>,
    #[serde(borrow, default, rename = "MinionStat")]
    pub minion_stats: Vec<BuildStat<'a>>,
    pub main_socket_group: u8,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BuildStat<'a> {
    #[serde(borrow, rename = "stat")]
    pub name: Cow<'a, str>,
    #[serde(borrow)]
    pub value: Cow<'a, str>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Skills<'a> {
    #[serde(borrow, default, rename = "$value")]
    pub skills: Vec<Skill<'a>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Skill<'a> {
    #[serde(default, deserialize_with = "u8_or_nil")]
    pub main_active_skill: u8,
    #[serde(borrow, default, rename = "$value")]
    pub gems: Vec<Gem<'a>>,
}

impl<'a> Skill<'a> {
    pub fn active_gems(&self) -> impl Iterator<Item = &Gem> {
        self.gems.iter().filter(|gem| gem.is_active())
    }

    pub fn support_gems(&self) -> impl Iterator<Item = &Gem> {
        self.gems.iter().filter(|gem| gem.is_support())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Gem<'a> {
    #[serde(borrow, rename = "nameSpec")]
    pub name: Cow<'a, str>,
    #[serde(borrow)]
    pub skill_id: Option<Cow<'a, str>>,
    #[serde(borrow)]
    pub gem_id: Option<Cow<'a, str>>,
}

impl<'a> Gem<'a> {
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
pub(crate) struct Tree<'a> {
    pub active_spec: u8,
    #[serde(borrow, rename = "Spec")]
    pub specs: Vec<Spec<'a>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Spec<'a> {
    #[serde(borrow, default)]
    pub title: Option<Cow<'a, str>>,
    #[serde(default, with = "StringWithSeparator::<CommaSeparator>")]
    pub nodes: Vec<u32>,
    #[serde(borrow, default, rename = "URL")]
    pub url: Option<Cow<'a, str>>,
}

#[derive(Default, Debug, Deserialize)]
pub(crate) struct Config<'a> {
    #[serde(borrow, default, rename = "Input")]
    pub input: Vec<Input<'a>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Input<'a> {
    #[serde(borrow)]
    pub name: Cow<'a, str>,
    #[serde(borrow)]
    pub string: Option<Cow<'a, str>>,
    pub boolean: Option<bool>,
    pub number: Option<f32>,
}
