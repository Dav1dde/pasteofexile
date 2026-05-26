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

macro_rules! string_enum {
    (
        enum $name:ident {
            $(
                $(#[$variant_attr:meta])*
                $variant:ident $(($display:literal))? $(| $alias:literal)*
            ),+ $(,)?
        }

        error = $error:literal;
    ) => {
        #[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
        pub enum $name {
            $(
                $(#[$variant_attr])*
                $variant,
            )+
        }

        impl $name {
            pub fn all() -> &'static [Self] {
                &[$(Self::$variant,)+]
            }

            pub fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$variant => string_enum!(@display $variant $($display)?),)+
                }
            }
        }

        impl FromStr for $name {
            type Err = Invalid;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                $(
                    if s == string_enum!(@display $variant $($display)?)
                        $(|| s == $alias)*
                    {
                        return Ok(Self::$variant);
                    }
                )+
                Err(Invalid($error))
            }
        }
    };
    (@display $variant:ident $display:literal) => {
        $display
    };
    (@display $variant:ident) => {
        stringify!($variant)
    };
}

string_enum! {
    enum Class {
        Duelist | "StrDex",
        Marauder | "Str",
        Ranger | "Dex",
        Scion | "StrDexInt",
        Shadow | "DexInt",
        Templar | "StrInt",
        Witch | "Int",

        // PoE 2
        Warrior,
        Mercenary,
        Huntress,
        Monk,
        Sorceress,
        Druid,
    }

    error = "Class";
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
        for &class in Class::all() {
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

macro_rules! ascendancy {
    ($($variant:ident $(($name:literal))? => $class:ident),+ $(,)?) => {
        string_enum! {
            enum Ascendancy {
                $($variant $(($name))?,)+
            }

            error = "Ascendancy";
        }

        impl Ascendancy {
            pub fn class(&self) -> Class {
                match self {
                    $(Self::$variant => Class::$class,)+
                }
            }
        }
    };
}

ascendancy!(
    Ascendant => Scion,
    Assassin => Shadow,
    Berserker => Marauder,
    Champion => Duelist,
    Chieftain => Marauder,
    Deadeye => Ranger,
    Elementalist => Witch,
    Gladiator => Duelist,
    Guardian => Templar,
    Hierophant => Templar,
    Inquisitor => Templar,
    Juggernaut => Marauder,
    Necromancer => Witch,
    Occultist => Witch,
    Pathfinder => Ranger,
    Raider => Ranger,
    Warden => Ranger,
    Saboteur => Shadow,
    Slayer => Duelist,
    Trickster => Shadow,
    Reliquarian => Scion,

    // PoE 2
    BloodMage("Blood Mage") => Witch,
    Infernalist => Witch,
    Lich => Witch,
    AbyssalLich("Abyssal Lich") => Witch,
    Titan => Warrior,
    Warbringer => Warrior,
    SmithOfKitava("Smith of Kitava") => Warrior,
    WitchHunter("Witchhunter") => Mercenary,
    GemlingLegionnaire("Gemling Legionnaire") => Mercenary,
    Tactician => Mercenary,
    Ritualist => Huntress,
    Amazon => Huntress,
    SpiritWalker("Spirit Walker") => Huntress,
    Invoker => Monk,
    AcolyteOfChayula("Acolyte of Chayula") => Monk,
    MartialArtist("Martial Artist") => Monk,
    Stormweaver => Sorceress,
    Chronomancer => Sorceress,
    DiscipleOfVarashta("Disciple of Varashta") => Sorceress,
    Oracle => Druid,
    Shaman => Druid,

    // Legacy of Phrecia
    Antiquarian => Marauder,
    Behemoth => Marauder,
    AncestralCommander("Ancestral Commander") => Marauder,
    Gambler => Duelist,
    Paladin => Duelist,
    Aristocrat => Duelist,
    ServantOfArakaali("Servant of Arakaali") => Shadow,
    Surfcaster => Shadow,
    BlindProphet("Blind Prophet") => Shadow,
    DaughterOfOshabi("Daughter of Oshabi") => Ranger,
    Whisperer => Ranger,
    Wildspeaker => Ranger,
    Harbinger => Witch,
    Herald => Witch,
    BogShaman("Bog Shaman") => Witch,
    ArchitectOfChaos("Architect of Chaos") => Templar,
    Polytheist => Templar,
    Puppeteer => Templar,
    Scavenger => Scion,
);

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

string_enum! {
    enum PantheonMajorGod {
        BrineKing("Soul of the Brine King") | "TheBrineKing",
        Lunaris("Soul of Lunaris") | "Lunaris",
        Solaris("Soul of Solaris") | "Solaris",
        Arakaali("Soul of Arakaali") | "Arakaali",
    }

    error = "Pantheon Major God";
}

string_enum! {
    enum PantheonMinorGod {
        Gruthkul("Soul of Gruthkul") | "Gruthkul",
        Yugul("Soul of Yugul") | "Lunaris",
        Abberath("Soul of Abberath") | "Solaris",
        Tukohama("Soul of Tukohama") | "Tukohama",
        Garukhan("Soul of Garukhan") | "Garukhan",
        Ralakesh("Soul of Ralakesh") | "Ralakesh",
        Ryslatha("Soul of Ryslatha") | "Ryslatha",
        Shakari("Soul of Shakari") | "Shakari",
    }

    error = "Pantheon Minor God";
}

string_enum! {
    enum Bandit {
        Alira,
        Kraityn,
        Oak,
    }

    error = "Bandit";
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

    #[test]
    fn test_class_from_str_aliases() {
        assert_eq!("Dex".parse::<Class>().unwrap(), Class::Ranger);
        assert_eq!("DexInt".parse::<Class>().unwrap(), Class::Shadow);
        assert_eq!("StrDexInt".parse::<Class>().unwrap(), Class::Scion);
        assert_eq!("Warrior".parse::<Class>().unwrap(), Class::Warrior);
    }

    #[test]
    fn test_pantheon_names_and_parse_tokens() {
        assert_eq!(
            PantheonMajorGod::BrineKing.as_str(),
            "Soul of the Brine King"
        );
        assert_eq!(
            "TheBrineKing".parse::<PantheonMajorGod>().unwrap(),
            PantheonMajorGod::BrineKing
        );
        assert_eq!(
            "Lunaris".parse::<PantheonMinorGod>().unwrap(),
            PantheonMinorGod::Yugul
        );
        assert_eq!(
            "Soul of Lunaris".parse::<PantheonMajorGod>().unwrap(),
            PantheonMajorGod::Lunaris
        );
    }
}
