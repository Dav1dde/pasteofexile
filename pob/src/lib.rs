use std::str::FromStr;

mod config;
mod error;
mod gems;
mod items;
mod passives;
mod serde;
mod stats;
mod utils;

pub use self::config::{Config, ConfigValue};
pub use self::error::{Error, Result};
pub use self::items::{Item, Mod, Rarity};
pub use self::passives::Keystone;
pub use self::serde::SerdePathOfBuilding;
pub use self::stats::Stat;
pub use self::utils::decompress;

pub trait PathOfBuilding {
    fn level(&self) -> u8;

    fn class_name(&self) -> &str;
    fn ascendancy_name(&self) -> Option<&str>;
    fn notes(&self) -> &str;

    fn stat(&self, stat: Stat) -> Option<&str>;
    fn minion_stat(&self, stat: Stat) -> Option<&str>;
    fn config(&self, config: Config) -> ConfigValue;
    fn main_skill_name(&self) -> Option<&str>;
    fn main_skill_supported_by(&self, skill: &str) -> bool;

    fn skill_sets(&self) -> Vec<SkillSet>;

    fn item_sets(&self) -> Vec<ItemSet>;

    fn tree_specs(&self) -> Vec<TreeSpec>;
    fn has_tree_node(&self, node: u32) -> bool;
    fn has_keystone(&self, keystone: Keystone) -> bool;
}

#[derive(Debug)]
pub struct TreeSpec<'a> {
    pub title: Option<&'a str>,
    pub url: Option<&'a str>,
    pub version: Option<&'a str>,
    pub nodes: &'a [u32],
    pub mastery_effects: &'a [(u32, u32)],

    /// Whether the tree spec is active/selected
    pub active: bool,
}

#[derive(Debug)]
pub struct SkillSet<'a> {
    pub id: u16,
    pub title: Option<&'a str>,
    pub skills: Vec<Skill<'a>>,
    pub is_selected: bool,
}

#[derive(Debug)]
pub struct Skill<'a> {
    pub is_selected: bool,
    pub is_enabled: bool,
    pub label: Option<&'a str>,
    pub slot: Option<&'a str>,
    pub gems: Vec<Gem<'a>>,
}

#[derive(Debug)]
pub struct Gem<'a> {
    pub name: &'a str,
    pub skill_id: Option<&'a str>,
    pub level: u8,
    pub quality: u8,
    pub is_enabled: bool,
    pub is_active: bool,
    pub is_support: bool,
    pub is_selected: bool,
}

#[derive(Debug, Default)]
pub struct ItemSet<'a> {
    pub id: u16,
    pub title: Option<&'a str>,
    pub gear: Gear<'a>,
}

#[derive(Debug, Default)]
pub struct Gear<'a> {
    pub weapon1: Option<&'a str>,
    pub weapon2: Option<&'a str>,
    pub helmet: Option<&'a str>,
    pub body_armour: Option<&'a str>,
    pub gloves: Option<&'a str>,
    pub boots: Option<&'a str>,
    pub amulet: Option<&'a str>,
    pub ring1: Option<&'a str>,
    pub ring2: Option<&'a str>,
    pub belt: Option<&'a str>,
    pub flask1: Option<&'a str>,
    pub flask2: Option<&'a str>,
    pub flask3: Option<&'a str>,
    pub flask4: Option<&'a str>,
    pub flask5: Option<&'a str>,
    pub sockets: Vec<&'a str>,
}

pub trait PathOfBuildingExt: PathOfBuilding {
    fn ascendancy_or_class_name(&self) -> &str {
        self.ascendancy_name().unwrap_or_else(|| self.class_name())
    }

    fn main_skill_supported_by_any<T>(&self, skills: T) -> bool
    where
        T: IntoIterator,
        T::Item: AsRef<str>,
    {
        skills
            .into_iter()
            .any(|skill| self.main_skill_supported_by(skill.as_ref()))
    }

    fn stat_parse<T: FromStr>(&self, name: Stat) -> Option<T> {
        PathOfBuilding::stat(self, name).and_then(|x| x.parse().ok())
    }

    fn stat_at_least(&self, name: Stat, value: f32) -> bool {
        self.stat_parse::<f32>(name)
            .map(|v| v >= value)
            .unwrap_or(false)
    }

    fn stat_at_most(&self, name: Stat, value: f32) -> bool {
        self.stat_parse::<f32>(name)
            .map(|v| v <= value)
            .unwrap_or(false)
    }

    fn minion_stat_parse<T: FromStr>(&self, name: Stat) -> Option<T> {
        PathOfBuilding::minion_stat(self, name).and_then(|x| x.parse().ok())
    }

    fn minion_stat_at_least(&self, name: Stat, value: f32) -> bool {
        self.minion_stat_parse::<f32>(name)
            .map(|v| v >= value)
            .unwrap_or(false)
    }

    fn minion_stat_at_most(&self, name: Stat, value: f32) -> bool {
        self.minion_stat_parse::<f32>(name)
            .map(|v| v <= value)
            .unwrap_or(false)
    }

    fn max_tree_version(&self) -> Option<String> {
        self.tree_specs()
            .into_iter()
            .filter_map(|spec| spec.version.map(|v| (v.len(), v)))
            .max()
            .map(|(_, version)| version.replace('_', "."))
    }
}

impl<T: PathOfBuilding> PathOfBuildingExt for T {}

impl std::fmt::Debug for dyn PathOfBuilding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PathOfBuilding")
            .field("level", &self.level())
            .field("ascendancy_name", &self.ascendancy_name())
            .finish()
    }
}
