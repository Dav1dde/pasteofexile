#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub enum Stat {
    Armour,
    AttackDodgeChance,
    AverageDamage,
    BlockChance,
    ChaosResistance,
    CombinedDps,
    ColdResistance,
    CritChance,
    CritMultiplier,
    Dexterity,
    EnduranceChargesMax,
    EnergyShield,
    EnergyShieldInc,
    Evasion,
    FireResistance,
    FullDps,
    Intelligence,
    Life,
    LifeInc,
    LifeUnreserved,
    LifeUnreservedPercent,
    LightningResistance,
    HitChance,
    HitRate,
    Mana,
    ManaInc,
    ManaUnreserved,
    MaxHitChaos,
    MaxHitCold,
    MaxHitFire,
    MaxHitLightning,
    MaxHitPhysical,
    MeleeEvadeChance,
    PhysicalDamageReduction,
    Speed,
    SpellBlockChance,
    SpellDodgeChance,
    SpellSuppressionChance,
    Strength,
    TotalEhp,
    Ward,
    Custom(&'static str),
}

impl Stat {
    fn name(&self) -> &'static str {
        match self {
            Self::Armour => "Armour",
            Self::AttackDodgeChance => "AttackDodgeChance",
            Self::AverageDamage => "AverageDamage",
            Self::BlockChance => "BlockChance",
            Self::ChaosResistance => "ChaosResist",
            Self::CombinedDps => "CombinedDPS",
            Self::ColdResistance => "ColdResist",
            Self::CritChance => "CritChance",
            Self::CritMultiplier => "CritMultiplier",
            Self::Dexterity => "Dex",
            Self::EnduranceChargesMax => "EnduranceChargesMax",
            Self::EnergyShield => "EnergyShield",
            Self::EnergyShieldInc => "Spec:EnergyShieldInc",
            Self::Evasion => "Evasion",
            Self::FireResistance => "FireResist",
            Self::FullDps => "FullDPS",
            Self::Intelligence => "Int",
            Self::Life => "Life",
            Self::LifeInc => "Spec:LifeInc",
            Self::LifeUnreserved => "LifeUnreserved",
            Self::LifeUnreservedPercent => "LifeUnreservedPercent",
            Self::LightningResistance => "LightningResist",
            Self::HitChance => "HitChance",
            Self::HitRate => "HitSpeed",
            Self::Mana => "Mana",
            Self::ManaInc => "Spec:ManaInc",
            Self::ManaUnreserved => "ManaUnreserved",
            Self::MaxHitChaos => "ChaosMaximumHitTaken",
            Self::MaxHitCold => "ColdMaximumHitTaken",
            Self::MaxHitFire => "FireMaximumHitTaken",
            Self::MaxHitLightning => "LightningMaximumHitTaken",
            Self::MaxHitPhysical => "PhysicalMaximumHitTaken",
            Self::MeleeEvadeChance => "MeleeEvadeChance",
            Self::PhysicalDamageReduction => "PhysicalDamageReduction",
            Self::Speed => "Speed",
            Self::SpellBlockChance => "SpellBlockChance",
            Self::SpellDodgeChance => "SpellDodgeChance",
            Self::SpellSuppressionChance => "SpellSuppressionChance",
            Self::Strength => "Str",
            Self::TotalEhp => "TotalEHP",
            Self::Ward => "Ward",
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
