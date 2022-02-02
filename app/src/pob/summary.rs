use crate::pob::{self, Element};
use ::pob::{Config, Keystone, PathOfBuilding, PathOfBuildingExt, SerdePathOfBuilding, Stat};

static AMBER_50: &str = "dark:text-amber-50 text-slate-800";

// TODO: accept any PathOfBuilding
pub fn core_stats(pob: &SerdePathOfBuilding) -> Vec<Element> {
    let mut elements = Vec::with_capacity(5);

    Element::new("Life")
        .color("text-rose-500")
        .stat_int(pob.stat_parse(Stat::LifeUnreserved))
        .stat_percent_if(
            !pob.has_keystone(Keystone::ChaosInoculation),
            pob.stat(Stat::LifeInc),
        )
        .add_to(&mut elements);

    if pob.stat_at_least(Stat::EnergyShield, 10.0) {
        Element::new("ES")
            .title("Energy Shield")
            .color("text-cyan-200")
            .stat_int(pob.stat_parse(Stat::EnergyShield))
            .stat_percent_if(pob::is_hybrid(pob), pob.stat(Stat::EnergyShieldInc))
            .add_to(&mut elements);
    }

    Element::new("Mana")
        .color("text-blue-400")
        .stat_int(pob.stat_parse(Stat::ManaUnreserved))
        .stat_percent_if(
            pob.has_keystone(Keystone::MindOverMatter),
            pob.stat(Stat::ManaInc),
        )
        .add_to(&mut elements);

    Element::new("Pool")
        .title("Total Health Pool")
        .color(AMBER_50)
        .stat_int(Some(pob::hp_pool(pob) as f32))
        .add_to(&mut elements);

    elements
}

pub fn defense(pob: &SerdePathOfBuilding) -> Vec<Element> {
    let mut elements = Vec::with_capacity(5);

    Element::new("Resistances")
        .push_percent(
            "text-orange-500 dark:text-orange-400",
            pob.stat(Stat::FireResistance).unwrap_or("-60"),
        )
        .push_percent(
            "text-blue-400",
            pob.stat(Stat::ColdResistance).unwrap_or("-60"),
        )
        .push_percent(
            "text-yellow-600 dark:text-yellow-300",
            pob.stat(Stat::LightningResistance).unwrap_or("-60"),
        )
        .push_percent(
            "text-fuchsia-500",
            pob.stat(Stat::ChaosResistance).unwrap_or("-60"),
        )
        .add_to(&mut elements);

    if pob.stat_at_least(Stat::MeleeEvadeChance, 20.0) {
        Element::new("Evade")
            .color(AMBER_50)
            .stat_percent(pob.stat(Stat::MeleeEvadeChance))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::PhysicalDamageReduction, 10.0)
        && pob.config(Config::EnemeyHit).is_some()
    {
        Element::new("PDR")
            .title("Physical Damage Reduction")
            .color(AMBER_50)
            .stat_percent(pob.stat(Stat::PhysicalDamageReduction))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::SpellSuppressionChance, 30.0) {
        Element::new("Supp")
            .title("Spell Suppression")
            .color(AMBER_50)
            .stat_percent(pob.stat(Stat::SpellSuppressionChance))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::AttackDodgeChance, 20.0) {
        Element::new("Dodge")
            .color(AMBER_50)
            .stat_percent(pob.stat(Stat::AttackDodgeChance))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::SpellDodgeChance, 10.0) {
        Element::new("Spell Dodge")
            .color(AMBER_50)
            .stat_percent(pob.stat(Stat::SpellDodgeChance))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::BlockChance, 30.0) {
        Element::new("Block")
            .color(AMBER_50)
            .stat_percent(pob.stat(Stat::BlockChance))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::SpellBlockChance, 10.0) {
        Element::new("Spell Block")
            .color(AMBER_50)
            .stat_percent(pob.stat(Stat::SpellBlockChance))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::Armour, 5000.0) {
        Element::new("Armour")
            .color(AMBER_50)
            .stat_int(pob.stat_parse(Stat::Armour))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::Evasion, 5000.0) {
        Element::new("Evasion")
            .color(AMBER_50)
            .stat_int(pob.stat_parse(Stat::Evasion))
            .add_to(&mut elements);
    }

    elements
}

pub fn offense(pob: &SerdePathOfBuilding) -> Vec<Element> {
    let mut elements = Vec::with_capacity(5);

    // TODO: real minion support
    let is_minion = pob.minion_stat(Stat::CombinedDps).is_some();
    let dps = if is_minion {
        pob.minion_stat_parse(Stat::CombinedDps)
    } else {
        pob.stat_parse(Stat::CombinedDps)
    };
    let speed = if is_minion {
        pob.minion_stat_parse(Stat::Speed)
    } else {
        pob.stat_parse(Stat::Speed)
    };

    Element::new("DPS")
        .color(AMBER_50)
        .stat_int(dps)
        .add_to(&mut elements);

    // TODO: this is cast rate for spells
    Element::new("Speed")
        .color(AMBER_50)
        .stat_float(speed)
        .add_to(&mut elements);

    Element::new("Hit Rate")
        .color(AMBER_50)
        .stat_float(pob.stat_parse(Stat::HitRate))
        .add_to(&mut elements);

    Element::new("Hit Chance")
        .color(AMBER_50)
        .stat_percent(pob.stat(Stat::HitChance))
        .add_to(&mut elements);

    if pob::is_crit(pob) {
        Element::new("Crit Chance")
            .color(AMBER_50)
            .stat_percent_float(pob.stat_parse(Stat::CritChance))
            .add_to(&mut elements);

        if pob.stat_at_least(Stat::CritMultiplier, 1.0) {
            Element::new("Crit Multi")
                .color(AMBER_50)
                .stat_percent_int(pob.stat_parse(Stat::CritMultiplier).map(|v: f32| v * 100.0))
                .add_to(&mut elements);
        }
    }

    elements
}

pub fn config(pob: &SerdePathOfBuilding) -> Vec<Element> {
    let mut configs = Vec::with_capacity(5);

    let boss = pob.config(Config::Boss);
    if boss.is_true() {
        configs.push("Boss".to_owned());
    } else if let Some(boss) = boss.string() {
        configs.push(boss.to_owned());
    }

    if pob.config(Config::Focused).is_true() {
        configs.push("Focused".to_owned());
    }

    if pob.config(Config::EnemyShocked).is_true() {
        let effect = pob.config(Config::ShockEffect).number().unwrap_or(15.0) as i32;
        configs.push(format!("{}% Shock", effect));
    }

    if pob.config(Config::CoveredInAsh).is_true() {
        configs.push("Covered in Ash".into());
    }

    if pob.config(Config::FrenzyCharges).is_true() {
        if let Some(amount) = pob.config(Config::FrenzyChargesAmount).number() {
            configs.push(format!("{}x Frenzy", amount as i32));
        } else {
            configs.push("Frenzy".into());
        }
    }

    if pob.config(Config::PowerCharges).is_true() {
        if let Some(amount) = pob.config(Config::PowerChargesAmount).number() {
            configs.push(format!("{}x Power", amount as i32));
        } else {
            configs.push("Power".into());
        }
    }

    if let Some(amount) = pob.config(Config::WitherStacks).number() {
        if amount > 0.0 {
            configs.push(format!("{}x Wither", amount as i32));
        }
    }

    if configs.is_empty() {
        configs.push("None".to_owned());
    }

    let element = Element::new("Config")
        .color(AMBER_50)
        .stat_str(Some(configs.join(", ")));

    vec![element]
}
