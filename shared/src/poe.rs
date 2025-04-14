use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Invalid(&'static str);

impl std::fmt::Display for Invalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid {}", self.0)
    }
}

impl std::error::Error for Invalid {}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum Color {
    Red,
    Green,
    Blue,
    White,
}

/// The major Path of Exile game version.
#[derive(Default, Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum GameVersion {
    /// Path of Exile 1
    #[default]
    One,
    /// Path of Exile 2
    Two,
}

impl GameVersion {
    pub fn is_poe1(self) -> bool {
        self == Self::One
    }

    pub fn is_poe2(self) -> bool {
        self == Self::Two
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Class {
    Duelist,
    Marauder,
    Ranger,
    Scion,
    Shadow,
    Templar,
    Witch,

    // PoE 2
    Warrior,
    Mercenary,
    Huntress,
    Monk,
    Sorceress,
    Druid,
}

impl Class {
    pub fn all() -> [Self; 13] {
        [
            Self::Duelist,
            Self::Marauder,
            Self::Ranger,
            Self::Scion,
            Self::Shadow,
            Self::Templar,
            Self::Witch,
            Self::Warrior,
            Self::Mercenary,
            Self::Huntress,
            Self::Monk,
            Self::Sorceress,
            Self::Druid,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Duelist => "Duelist",
            Self::Marauder => "Marauder",
            Self::Ranger => "Ranger",
            Self::Scion => "Scion",
            Self::Shadow => "Shadow",
            Self::Templar => "Templar",
            Self::Witch => "Witch",

            Self::Warrior => "Warrior",
            Self::Mercenary => "Mercenary",
            Self::Huntress => "Huntress",
            Self::Monk => "Monk",
            Self::Sorceress => "Sorceress",
            Self::Druid => "Druid",
        }
    }
}

impl FromStr for Class {
    type Err = Invalid;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Dex" => Self::Ranger,
            "DexInt" => Self::Shadow,
            "Int" => Self::Witch,
            "Str" => Self::Marauder,
            "StrDex" => Self::Duelist,
            "StrDexInt" => Self::Scion,
            "StrInt" => Self::Templar,

            "Duelist" => Self::Duelist,
            "Marauder" => Self::Marauder,
            "Ranger" => Self::Ranger,
            "Scion" => Self::Scion,
            "Shadow" => Self::Shadow,
            "Templar" => Self::Templar,
            "Witch" => Self::Witch,

            "Warrior" => Self::Warrior,
            "Mercenary" => Self::Mercenary,
            "Huntress" => Self::Huntress,
            "Monk" => Self::Monk,
            "Sorceress" => Self::Sorceress,
            "Druid" => Self::Druid,

            _ => return Err(Invalid("Class")),
        })
    }
}

impl std::ops::BitOr for Class {
    type Output = ClassSet;

    fn bitor(self, rhs: Self) -> Self::Output {
        ClassSet::new() | self | rhs
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClassSet(u16);

impl ClassSet {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn all() -> Self {
        Self::from_u16(!0)
    }

    pub const fn from_u16(val: u16) -> Self {
        Self(val & 0b1111111111111)
    }

    pub fn as_u16(&self) -> u16 {
        self.0
    }

    pub fn contains(&self, other: Class) -> bool {
        (*self & other).0 > 0
    }
}

impl Default for ClassSet {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ClassSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ClassSet(")?;

        let mut first = true;
        for class in Class::all() {
            if self.contains(class) {
                if !first {
                    write!(f, " | ")?;
                }
                write!(f, "{class:?}")?;
                first = false;
            }
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl std::ops::BitOr<Class> for ClassSet {
    type Output = ClassSet;

    fn bitor(self, rhs: Class) -> Self::Output {
        Self(self.0 | 1 << (rhs as u8))
    }
}

impl std::ops::BitAnd<Class> for ClassSet {
    type Output = ClassSet;

    fn bitand(self, rhs: Class) -> Self::Output {
        Self(self.0 & 1 << (rhs as u8))
    }
}

impl<const N: usize> From<[Class; N]> for ClassSet {
    fn from(value: [Class; N]) -> Self {
        let mut result = Self::new();
        for class in value {
            result = result | class;
        }
        result
    }
}

impl FromIterator<Class> for ClassSet {
    fn from_iter<T: IntoIterator<Item = Class>>(iter: T) -> Self {
        let mut result = Self::new();
        for class in iter {
            result = result | class;
        }
        result
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ascendancy {
    Ascendant,
    Assassin,
    Berserker,
    Champion,
    Chieftain,
    Deadeye,
    Elementalist,
    Gladiator,
    Guardian,
    Hierophant,
    Inquisitor,
    Juggernaut,
    Necromancer,
    Occultist,
    Pathfinder,
    Raider,
    Warden,
    Saboteur,
    Slayer,
    Trickster,

    // PoE 2
    BloodMage,
    Infernalist,
    Lich,
    Titan,
    Warbringer,
    SmithOfKitava,
    WitchHunter,
    GemlingLegionnaire,
    Tactician,
    Ritualist,
    Amazon,
    Invoker,
    AcolyteOfChayula,
    Stormweaver,
    Chronomancer,

    // Legacy of Phrecia
    Antiquarian,
    Behemoth,
    AncestralCommander,
    Gambler,
    Paladin,
    Aristocrat,
    ServantOfArakaali,
    Surfcaster,
    BlindProphet,
    DaughterOfOshabi,
    Whisperer,
    Wildspeaker,
    Harbinger,
    Herald,
    BogShaman,
    ArchitectOfChaos,
    Polytheist,
    Puppeteer,
    Scavenger,
}

impl Ascendancy {
    pub fn class(&self) -> Class {
        match self {
            Self::Ascendant => Class::Scion,
            Self::Assassin => Class::Shadow,
            Self::Berserker => Class::Marauder,
            Self::Champion => Class::Duelist,
            Self::Chieftain => Class::Marauder,
            Self::Deadeye => Class::Ranger,
            Self::Elementalist => Class::Witch,
            Self::Gladiator => Class::Duelist,
            Self::Guardian => Class::Templar,
            Self::Hierophant => Class::Templar,
            Self::Inquisitor => Class::Templar,
            Self::Juggernaut => Class::Marauder,
            Self::Necromancer => Class::Witch,
            Self::Occultist => Class::Witch,
            Self::Pathfinder => Class::Ranger,
            Self::Raider => Class::Ranger,
            Self::Warden => Class::Ranger,
            Self::Saboteur => Class::Shadow,
            Self::Slayer => Class::Duelist,
            Self::Trickster => Class::Shadow,
            Self::BloodMage => Class::Witch,
            Self::Infernalist => Class::Witch,
            Self::Lich => Class::Witch,
            Self::Titan => Class::Warrior,
            Self::Warbringer => Class::Warrior,
            Self::SmithOfKitava => Class::Warrior,
            Self::WitchHunter => Class::Mercenary,
            Self::GemlingLegionnaire => Class::Mercenary,
            Self::Tactician => Class::Mercenary,
            Self::Ritualist => Class::Huntress,
            Self::Amazon => Class::Huntress,
            Self::Invoker => Class::Monk,
            Self::AcolyteOfChayula => Class::Monk,
            Self::Stormweaver => Class::Sorceress,
            Self::Chronomancer => Class::Sorceress,
            Self::Antiquarian => Class::Marauder,
            Self::Behemoth => Class::Marauder,
            Self::AncestralCommander => Class::Marauder,
            Self::Gambler => Class::Duelist,
            Self::Paladin => Class::Duelist,
            Self::Aristocrat => Class::Duelist,
            Self::ServantOfArakaali => Class::Shadow,
            Self::Surfcaster => Class::Shadow,
            Self::BlindProphet => Class::Shadow,
            Self::DaughterOfOshabi => Class::Ranger,
            Self::Whisperer => Class::Ranger,
            Self::Wildspeaker => Class::Ranger,
            Self::Harbinger => Class::Witch,
            Self::Herald => Class::Witch,
            Self::BogShaman => Class::Witch,
            Self::ArchitectOfChaos => Class::Templar,
            Self::Polytheist => Class::Templar,
            Self::Puppeteer => Class::Templar,
            Self::Scavenger => Class::Scion,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ascendant => "Ascendant",
            Self::Assassin => "Assassin",
            Self::Berserker => "Berserker",
            Self::Champion => "Champion",
            Self::Chieftain => "Chieftain",
            Self::Deadeye => "Deadeye",
            Self::Elementalist => "Elementalist",
            Self::Gladiator => "Gladiator",
            Self::Guardian => "Guardian",
            Self::Hierophant => "Hierophant",
            Self::Inquisitor => "Inquisitor",
            Self::Juggernaut => "Juggernaut",
            Self::Necromancer => "Necromancer",
            Self::Occultist => "Occultist",
            Self::Pathfinder => "Pathfinder",
            Self::Raider => "Raider",
            Self::Warden => "Warden",
            Self::Saboteur => "Saboteur",
            Self::Slayer => "Slayer",
            Self::Trickster => "Trickster",
            Self::BloodMage => "Blood Mage",
            Self::Infernalist => "Infernalist",
            Self::Lich => "Lich",
            Self::Titan => "Titan",
            Self::Warbringer => "Warbringer",
            Self::SmithOfKitava => "Smith of Kitava",
            Self::WitchHunter => "Witchhunter",
            Self::GemlingLegionnaire => "Gemling Legionnaire",
            Self::Tactician => "Tactician",
            Self::Ritualist => "Ritualist",
            Self::Amazon => "Amazon",
            Self::Invoker => "Invoker",
            Self::AcolyteOfChayula => "Acolyte of Chayula",
            Self::Stormweaver => "Stormweaver",
            Self::Chronomancer => "Chronomancer",
            Self::Antiquarian => "Antiquarian",
            Self::Behemoth => "Behemoth",
            Self::AncestralCommander => "Ancestral Commander",
            Self::Gambler => "Gambler",
            Self::Paladin => "Paladin",
            Self::Aristocrat => "Aristocrat",
            Self::ServantOfArakaali => "Servant of Arakaali",
            Self::Surfcaster => "Surfcaster",
            Self::BlindProphet => "Blind Prophet",
            Self::DaughterOfOshabi => "Daughter of Oshabi",
            Self::Whisperer => "Whisperer",
            Self::Wildspeaker => "Wildspeaker",
            Self::Harbinger => "Harbinger",
            Self::Herald => "Herald",
            Self::BogShaman => "Bog Shaman",
            Self::ArchitectOfChaos => "Architect of Chaos",
            Self::Polytheist => "Polytheist",
            Self::Puppeteer => "Puppeteer",
            Self::Scavenger => "Scavenger",
        }
    }
}

impl FromStr for Ascendancy {
    type Err = Invalid;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Ascendant" => Self::Ascendant,
            "Assassin" => Self::Assassin,
            "Berserker" => Self::Berserker,
            "Champion" => Self::Champion,
            "Chieftain" => Self::Chieftain,
            "Deadeye" => Self::Deadeye,
            "Elementalist" => Self::Elementalist,
            "Gladiator" => Self::Gladiator,
            "Guardian" => Self::Guardian,
            "Hierophant" => Self::Hierophant,
            "Inquisitor" => Self::Inquisitor,
            "Juggernaut" => Self::Juggernaut,
            "Necromancer" => Self::Necromancer,
            "Occultist" => Self::Occultist,
            "Pathfinder" => Self::Pathfinder,
            "Raider" => Self::Raider,
            "Warden" => Self::Warden,
            "Saboteur" => Self::Saboteur,
            "Slayer" => Self::Slayer,
            "Trickster" => Self::Trickster,

            "Blood Mage" => Self::BloodMage,
            "Infernalist" => Self::Infernalist,
            "Lich" => Self::Lich,
            "Titan" => Self::Titan,
            "Warbringer" => Self::Warbringer,
            "Smith of Kitava" => Self::SmithOfKitava,
            "Witchhunter" => Self::WitchHunter,
            "Gemling Legionnaire" => Self::GemlingLegionnaire,
            "Tactician" => Self::Tactician,
            "Ritualist" => Self::Ritualist,
            "Amazon" => Self::Amazon,
            "Invoker" => Self::Invoker,
            "Acolyte of Chayula" => Self::AcolyteOfChayula,
            "Stormweaver" => Self::Stormweaver,
            "Chronomancer" => Self::Chronomancer,

            "Antiquarian" => Self::Antiquarian,
            "Behemoth" => Self::Behemoth,
            "Ancestral Commander" => Self::AncestralCommander,
            "Gambler" => Self::Gambler,
            "Paladin" => Self::Paladin,
            "Aristocrat" => Self::Aristocrat,
            "Servant of Arakaali" => Self::ServantOfArakaali,
            "Surfcaster" => Self::Surfcaster,
            "Blind Prophet" => Self::BlindProphet,
            "Daughter of Oshabi" => Self::DaughterOfOshabi,
            "Whisperer" => Self::Whisperer,
            "Wildspeaker" => Self::Wildspeaker,
            "Harbinger" => Self::Harbinger,
            "Herald" => Self::Herald,
            "Bog Shaman" => Self::BogShaman,
            "Architect of Chaos" => Self::ArchitectOfChaos,
            "Polytheist" => Self::Polytheist,
            "Puppeteer" => Self::Puppeteer,
            "Scavenger" => Self::Scavenger,

            _ => return Err(Invalid("Ascendancy")),
        })
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum AscendancyOrClass {
    Ascendancy(Ascendancy),
    Class(Class),
}

impl AscendancyOrClass {
    pub fn class(&self) -> Class {
        match self {
            AscendancyOrClass::Ascendancy(asc) => asc.class(),
            AscendancyOrClass::Class(class) => *class,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ascendancy(asc) => asc.as_str(),
            Self::Class(class) => class.as_str(),
        }
    }
}

impl From<Ascendancy> for AscendancyOrClass {
    fn from(value: Ascendancy) -> Self {
        Self::Ascendancy(value)
    }
}

impl From<Class> for AscendancyOrClass {
    fn from(value: Class) -> Self {
        Self::Class(value)
    }
}

impl FromStr for AscendancyOrClass {
    type Err = Invalid;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(ascendancy) = s.parse() {
            return Ok(Self::Ascendancy(ascendancy));
        }
        if let Ok(class) = s.parse() {
            return Ok(Self::Class(class));
        }
        Err(Invalid("Ascendancy or Class"))
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum PantheonMajorGod {
    BrineKing,
    Lunaris,
    Solaris,
    Arakaali,
}

impl PantheonMajorGod {
    pub fn name(self) -> &'static str {
        match self {
            Self::BrineKing => "Soul of the Brine King",
            Self::Lunaris => "Soul of Lunaris",
            Self::Solaris => "Soul of Solaris",
            Self::Arakaali => "Soul of Arakaali",
        }
    }
}

impl FromStr for PantheonMajorGod {
    type Err = Invalid;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "TheBrineKing" => Self::BrineKing,
            "Lunaris" => Self::Lunaris,
            "Solaris" => Self::Solaris,
            "Arakaali" => Self::Arakaali,
            _ => return Err(Invalid("Pantheon Major God")),
        })
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum PantheonMinorGod {
    Gruthkul,
    Yugul,
    Abberath,
    Tukohama,
    Garukhan,
    Ralakesh,
    Ryslatha,
    Shakari,
}

impl PantheonMinorGod {
    pub fn name(self) -> &'static str {
        match self {
            Self::Gruthkul => "Soul of Gruthkul",
            Self::Yugul => "Soul of Yugul",
            Self::Abberath => "Soul of Abberath",
            Self::Tukohama => "Soul of Tukohama",
            Self::Garukhan => "Soul of Garukhan",
            Self::Ralakesh => "Soul of Ralakesh",
            Self::Ryslatha => "Soul of Ryslatha",
            Self::Shakari => "Soul of Shakari",
        }
    }
}

impl FromStr for PantheonMinorGod {
    type Err = Invalid;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Gruthkul" => Self::Gruthkul,
            "Lunaris" => Self::Yugul,
            "Solaris" => Self::Abberath,
            "Tukohama" => Self::Tukohama,
            "Garukhan" => Self::Garukhan,
            "Ralakesh" => Self::Ralakesh,
            "Ryslatha" => Self::Ryslatha,
            "Shakari" => Self::Shakari,
            _ => return Err(Invalid("Pantheon Minor God")),
        })
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Bandit {
    Alira,
    Kraityn,
    Oak,
}

impl Bandit {
    pub fn name(self) -> &'static str {
        match self {
            Bandit::Alira => "Alira",
            Bandit::Kraityn => "Kraityn",
            Bandit::Oak => "Oak",
        }
    }
}

impl FromStr for Bandit {
    type Err = Invalid;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Alira" => Self::Alira,
            "Kraityn" => Self::Kraityn,
            "Oak" => Self::Oak,
            _ => return Err(Invalid("Bandit")),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_set() {
        assert_eq!(0b1000001, (Class::Duelist | Class::Witch).as_u16());
        assert!(ClassSet::from_u16(0b1000001).contains(Class::Duelist));
        assert!(ClassSet::from_u16(0b1000001).contains(Class::Witch));
        assert!(!ClassSet::from_u16(0b1000001).contains(Class::Ranger));
        assert_eq!(
            (Class::Duelist | Class::Witch),
            ClassSet::from([Class::Duelist, Class::Witch])
        );
        // Top most 3 bits are unused, make sure it is discarded
        assert_eq!(
            ClassSet::from_u16(0b1111000001000001),
            ClassSet::from_u16(0b0001000001000001)
        );
        assert_eq!(ClassSet::all(), ClassSet::from_u16(0b01111111111111));
    }
}
