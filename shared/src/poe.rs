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

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Class {
    Duelist,
    Marauder,
    Ranger,
    Scion,
    Shadow,
    Templar,
    Witch,
}
impl Class {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Duelist => "Duelist",
            Self::Marauder => "Marauder",
            Self::Ranger => "Ranger",
            Self::Scion => "Scion",
            Self::Shadow => "Shadow",
            Self::Templar => "Templar",
            Self::Witch => "Witch",
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
pub struct ClassSet(u8);

impl ClassSet {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn all() -> Self {
        Self::from_u8(!0)
    }

    pub const fn from_u8(val: u8) -> Self {
        Self(val & 0b1111111)
    }

    pub fn as_u8(&self) -> u8 {
        self.0
    }

    pub fn contains(&self, other: Class) -> bool {
        (*self & other).0 > 0
    }
}

impl std::fmt::Debug for ClassSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ClassSet(")?;

        use Class::*;
        let mut first = true;
        for class in [Duelist, Marauder, Ranger, Scion, Shadow, Templar, Witch] {
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
    Saboteur,
    Slayer,
    Trickster,
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
            Self::Saboteur => Class::Shadow,
            Self::Slayer => Class::Duelist,
            Self::Trickster => Class::Shadow,
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
            Self::Saboteur => "Saboteur",
            Self::Slayer => "Slayer",
            Self::Trickster => "Trickster",
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
            "Saboteur" => Self::Saboteur,
            "Slayer" => Self::Slayer,
            "Trickster" => Self::Trickster,

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
        assert_eq!(0b1000001, (Class::Duelist | Class::Witch).as_u8());
        assert!(ClassSet::from_u8(0b1000001).contains(Class::Duelist));
        assert!(ClassSet::from_u8(0b1000001).contains(Class::Witch));
        assert!(!ClassSet::from_u8(0b1000001).contains(Class::Ranger));
        assert_eq!(
            (Class::Duelist | Class::Witch),
            ClassSet::from([Class::Duelist, Class::Witch])
        );
        // Top most bit is unused, make sure it is discarded
        assert_eq!(ClassSet::from_u8(0b11000001), ClassSet::from_u8(0b01000001));
        assert_eq!(ClassSet::all(), ClassSet::from_u8(0b01111111));
    }
}
