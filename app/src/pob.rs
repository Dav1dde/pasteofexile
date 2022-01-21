use pob::{Keystone, PathOfBuilding, PathOfBuildingExt, Stat};

macro_rules! push_if {
    ($vec:ident, $expr:expr, $value:expr) => {{
        if $expr {
            $vec.push($value);
            true
        } else {
            false
        }
    }};
}

#[inline]
fn is_crit<T: PathOfBuilding>(pob: &T) -> bool {
    !pob.has_keystone(Keystone::ElementalOverload) && pob.stat_at_least(Stat::CritChance, 20.0)
}

#[inline]
fn is_low_life<T: PathOfBuilding>(pob: &T) -> bool {
    pob.stat_at_most(Stat::LifeUnreservedPercent, 50.0)
}

#[inline]
fn is_hybrid<T: PathOfBuilding>(pob: &T) -> bool {
    !pob.has_keystone(Keystone::ChaosInoculation)
        && !pob.has_keystone(Keystone::EldritchBattery)
        && !is_low_life(pob)
        && pob.stat_at_least(
            Stat::EnergyShield,
            pob.stat_parse(Stat::LifeUnreserved).unwrap_or(0.0) * 0.25,
        )
}

pub fn title<T: PathOfBuilding>(pob: &T) -> String
where
    T: std::fmt::Debug,
{
    let mut items = Vec::with_capacity(5);
    let level = format!("Level {}", pob.level());
    items.push(level.as_str());

    push_if!(items, is_low_life(pob), "LL");
    push_if!(items, is_hybrid(pob), "Hybrid");
    push_if!(items, pob.has_keystone(Keystone::ChaosInoculation), "CI");
    push_if!(items, pob.has_keystone(Keystone::MindOverMatter), "MoM");

    push_if!(items, is_crit(pob), "Crit");
    push_if!(
        items,
        pob.main_skill_supported_by_any(&[
            "Cast On Critical Strike",
            "Awakened Cast On Critical Strike"
        ]),
        "CoC"
    );

    if let Some(main_skill) = pob.main_skill_name() {
        items.push(main_skill);
    }

    if pob.main_skill_supported_by_any(&["Spell Totem", "Ballista Totem"]) {
        items.push("Totem");
    } else if pob.main_skill_supported_by_any(&["Blastchain Main", "High-Impact Mine"]) {
        items.push("Mine");
    } else if pob.main_skill_supported_by("Trap") {
        items.push("Trap");
    }

    items.push(pob.ascendancy_or_class_name());

    items.join(" ")
}
