use serde::{de, Deserialize};

use crate::serde::utils;

#[derive(Debug, Deserialize)]
pub(crate) struct PathOfBuilding {
    #[serde(rename = "Build")]
    pub build: Build,

    #[serde(rename = "Skills")]
    pub skills: Skills,

    #[serde(rename = "Tree")]
    pub tree: Tree,

    #[serde(default, rename = "Items")]
    pub items: Items,

    #[serde(default, rename = "Notes")]
    pub notes: String,

    #[serde(default, rename = "Config")]
    pub config: Config,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Build {
    pub level: u8,
    pub class_name: String,
    pub ascend_class_name: String,
    #[serde(rename = "$value")]
    pub stats: Vec<StatType>,
    pub main_socket_group: u8,
}

#[derive(Debug, Deserialize)]
pub(crate) enum StatType {
    PlayerStat(BuildStat),
    #[serde(rename = "FullDPSSkill")]
    FullDpsSkill(BuildStat),
    MinionStat(BuildStat),
    #[serde(other)]
    Unknown,
}

impl StatType {
    pub(crate) fn player(&self) -> Option<&BuildStat> {
        match self {
            Self::PlayerStat(stat) => Some(stat),
            _ => None,
        }
    }

    pub(crate) fn minion(&self) -> Option<&BuildStat> {
        match self {
            Self::MinionStat(stat) => Some(stat),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct BuildStat {
    #[serde(rename = "stat")]
    pub name: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Skills {
    #[serde(default, rename = "activeSkillSet")]
    pub active_skill_set: Option<u16>,

    // Newer exports have skills nested in skill sets.
    // QuickXML doesn't allow me to use an enum here.
    #[serde(default, rename = "SkillSet")]
    pub skill_sets: Vec<SkillSet>,
    #[serde(default, rename = "Skill")]
    pub skills: Vec<Skill>,
}

impl Skills {
    pub fn active_skills(&self) -> &[Skill] {
        self.active_skill_set
            .and_then(|active_skill_set| {
                self.skill_sets.iter().find(|ss| ss.id == active_skill_set)
            })
            .map(|ss| &ss.skills)
            .unwrap_or(&self.skills)
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct SkillSet {
    #[serde(rename = "id")]
    pub id: u16,
    #[serde(default, rename = "title")]
    pub title: Option<String>,
    #[serde(default, rename = "Skill")]
    pub skills: Vec<Skill>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Skill {
    #[serde(default, deserialize_with = "utils::u8_or_nil")]
    pub main_active_skill: u8,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub slot: Option<String>,
    #[serde(default, rename = "Gem")]
    pub gems: Vec<Gem>,
}

impl Skill {
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
    #[serde(default = "utils::default_true")]
    pub enabled: bool,
    #[serde(default, deserialize_with = "utils::lenient")]
    pub level: u8,
    #[serde(default, deserialize_with = "utils::lenient")]
    pub quality: u8,
}

impl Gem {
    pub fn is_support(&self) -> bool {
        if let Some(gem_id) = &self.gem_id {
            return gem_id.starts_with("Metadata/Items/Gems/Support");
        }
        if let Some(skill_id) = &self.skill_id {
            // `SupportVoidManipulation` but also `ViciousHexSupport`
            return skill_id.starts_with("Support") || skill_id.ends_with("Support");
        }
        self.name.contains("Support")
    }

    pub fn is_active(&self) -> bool {
        !self.is_support()
    }

    pub fn is_vaal(&self) -> bool {
        self.name.starts_with("Vaal ")
    }

    pub fn non_vaal_name(&self) -> &str {
        self.name.strip_prefix("Vaal ").unwrap_or(&self.name)
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
    #[serde(default, deserialize_with = "utils::comma_separated")]
    pub nodes: Vec<u32>,
    #[serde(default, deserialize_with = "utils::lua_table")]
    pub mastery_effects: Vec<(u32, u32)>,
    #[serde(default, rename = "URL")]
    pub url: Option<String>,
    #[serde(default, rename = "treeVersion")]
    pub version: Option<String>,
}

#[derive(Default, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Items {
    #[serde(default, rename = "Item")]
    pub items: Vec<Item>,
    #[serde(default, rename = "Slot")]
    pub slots: Vec<Slot>,
}

#[derive(Default, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Item {
    pub id: u16,
    // this might be parsable with serde_as into a `(String, Vec<()>)`
    #[serde(rename = "$value")]
    pub content: ItemContent,
}

#[derive(Default, Debug)]
pub(crate) struct ItemContent {
    pub content: String,
}

impl<'de> de::Deserialize<'de> for ItemContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = ItemContent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("expected pob item")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                // first element is the item content
                let content = seq.next_element::<String>()?.unwrap_or_default();
                // following elements are mod ranges, ignore them for now
                while seq.next_element::<()>()?.is_some() {}

                Ok(ItemContent { content })
            }
        }

        deserializer.deserialize_seq(Visitor)
    }
}

#[derive(Default, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Slot {
    pub item_id: u16,
}

#[derive(Default, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
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
