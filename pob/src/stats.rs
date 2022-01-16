#[derive(Debug)]
#[non_exhaustive]
pub enum Stat {
    AverageDamage,
    CritChance,
    EnduranceChargesMax,
    EnergyShield,
    LifeUnreserved,
    LifeUnreservedPercent,
    Custom(&'static str),
}

impl Stat {
    fn name(&self) -> &'static str {
        match self {
            Self::AverageDamage => "AverageDamage",
            Self::CritChance => "CritChance",
            Self::EnduranceChargesMax => "EnduranceChargesMax",
            Self::EnergyShield => "EnergyShield",
            Self::LifeUnreserved => "LifeUnreserved",
            Self::LifeUnreservedPercent => "LifeUnreservedPercent",
            Self::Custom(s) => s,
        }
    }
}

impl From<Stat> for &'static str {
    fn from(stat: Stat) -> Self {
        stat.name()
    }
}

impl PartialEq<str> for Stat {
    fn eq(&self, other: &str) -> bool {
        self.name() == other
    }
}

impl PartialEq<Stat> for &str {
    fn eq(&self, other: &Stat) -> bool {
        &other == self
    }
}

impl PartialEq<String> for Stat {
    fn eq(&self, other: &String) -> bool {
        self.name() == other
    }
}

impl PartialEq<Stat> for String {
    fn eq(&self, other: &Stat) -> bool {
        other == self
    }
}
