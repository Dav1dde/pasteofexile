use std::collections::HashMap;

use serde::{de, Deserialize, Deserializer};

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

#[derive(Debug)]
pub(crate) struct Gem {
    pub name: String,
    pub skill_id: Option<String>,
    pub gem_id: Option<String>,
    pub quality_id: Option<String>,
    pub enabled: bool,
    pub level: u8,
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

impl<'de> Deserialize<'de> for Gem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Inner {
            #[serde(rename = "nameSpec")]
            name: String,
            skill_id: Option<String>,
            gem_id: Option<String>,
            quality_id: Option<String>,
            #[serde(default = "utils::default_true")]
            enabled: bool,
            #[serde(default, deserialize_with = "utils::lenient")]
            level: u8,
            #[serde(default, deserialize_with = "utils::lenient")]
            quality: u8,
        }

        let inner = Inner::deserialize(deserializer)?;

        let name = if inner.name.is_empty() {
            inner
                .skill_id
                .as_deref()
                .and_then(crate::gems::skill_name_fallback)
                .map(Into::into)
                .unwrap_or_default()
        } else {
            inner.name
        };

        Ok(Self {
            name,
            skill_id: inner.skill_id,
            gem_id: inner.gem_id,
            quality_id: inner.quality_id,
            enabled: inner.enabled,
            level: inner.level,
            quality: inner.quality,
        })
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
    #[serde(default, rename = "Sockets")]
    pub sockets: Sockets,
    #[serde(default, rename = "treeVersion")]
    pub version: Option<String>,
}

#[derive(Default, Debug, Deserialize)]
pub(crate) struct Sockets {
    #[serde(default, rename = "Socket")]
    pub sockets: Vec<Socket>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Socket {
    #[serde(default, rename = "nodeId")]
    pub node_id: u32,
    #[serde(default, rename = "itemId")]
    pub item_id: u16,
}

#[derive(Default, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Items {
    pub active_item_set: Option<u16>,
    #[serde(default, rename = "Item", deserialize_with = "deserialize_items")]
    pub items: HashMap<u16, Item>,
    #[serde(default, rename = "ItemSet")]
    pub item_sets: Vec<ItemSet>,
}

fn deserialize_items<'de, D>(deserializer: D) -> Result<HashMap<u16, Item>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'de> de::Visitor<'de> for Visitor {
        type Value = HashMap<u16, Item>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("list of pob items")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut result = HashMap::with_capacity(seq.size_hint().unwrap_or(0));

            while let Some(item) = seq.next_element::<Item>()? {
                result.insert(item.id, item);
            }

            Ok(result)
        }
    }

    deserializer.deserialize_seq(Visitor)
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
    pub name: String,
}

#[derive(Default, Debug, Deserialize)]
pub(crate) struct ItemSet {
    pub id: u16,
    pub title: Option<String>,
    #[serde(default, rename = "$value")]
    pub gear: Gear,
}

#[derive(Debug, Default)]
pub(crate) struct Gear {
    pub weapon1: Option<u16>,
    pub weapon2: Option<u16>,
    pub weapon1_swap: Option<u16>,
    pub weapon2_swap: Option<u16>,
    pub helmet: Option<u16>,
    pub body_armour: Option<u16>,
    pub gloves: Option<u16>,
    pub boots: Option<u16>,
    pub amulet: Option<u16>,
    pub ring1: Option<u16>,
    pub ring2: Option<u16>,
    pub belt: Option<u16>,
    pub flask1: Option<u16>,
    pub flask2: Option<u16>,
    pub flask3: Option<u16>,
    pub flask4: Option<u16>,
    pub flask5: Option<u16>,
    pub sockets: Vec<u16>,
}

impl<'de> de::Deserialize<'de> for Gear {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Gear;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("expected pob item")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut result = Gear::default();

                #[derive(Deserialize)]
                enum Inner {
                    Slot(Slot),
                    // There are non slot entries mixed into the slots
                    // just ignore them.
                    #[serde(other)]
                    Unknown,
                }

                while let Some(slot) = seq.next_element::<Inner>()? {
                    let Inner::Slot(slot) = slot else {
                        continue;
                    };
                    if slot.item_id == 0 {
                        continue;
                    }

                    match slot.name.as_str() {
                        "Weapon 1" => result.weapon1 = Some(slot.item_id),
                        "Weapon 2" => result.weapon2 = Some(slot.item_id),
                        "Weapon 1 Swap" => result.weapon1_swap = Some(slot.item_id),
                        "Weapon 2 Swap" => result.weapon2_swap = Some(slot.item_id),
                        "Helmet" => result.helmet = Some(slot.item_id),
                        "Body Armour" => result.body_armour = Some(slot.item_id),
                        "Gloves" => result.gloves = Some(slot.item_id),
                        "Boots" => result.boots = Some(slot.item_id),
                        "Amulet" => result.amulet = Some(slot.item_id),
                        "Ring 1" => result.ring1 = Some(slot.item_id),
                        "Ring 2" => result.ring2 = Some(slot.item_id),
                        "Belt" => result.belt = Some(slot.item_id),
                        "Flask 1" => result.flask1 = Some(slot.item_id),
                        "Flask 2" => result.flask2 = Some(slot.item_id),
                        "Flask 3" => result.flask3 = Some(slot.item_id),
                        "Flask 4" => result.flask4 = Some(slot.item_id),
                        "Flask 5" => result.flask5 = Some(slot.item_id),
                        _ => result.sockets.push(slot.item_id),
                    }
                }

                Ok(result)
            }
        }

        deserializer.deserialize_seq(Visitor)
    }
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
