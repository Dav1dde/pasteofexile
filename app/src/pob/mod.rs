use pob::{Keystone, PathOfBuilding, PathOfBuildingExt, Stat};

mod element;
pub mod formatting;
pub mod summary;

pub use self::element::Element;

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

pub fn is_crit<T: PathOfBuilding>(pob: &T) -> bool {
    !pob.has_keystone(Keystone::ElementalOverload) && pob.stat_at_least(Stat::CritChance, 20.0)
}

pub fn is_low_life<T: PathOfBuilding>(pob: &T) -> bool {
    pob.stat_at_most(Stat::LifeUnreservedPercent, 50.0)
}

pub fn is_hybrid<T: PathOfBuilding>(pob: &T) -> bool {
    !pob.has_keystone(Keystone::ChaosInoculation)
        && !pob.has_keystone(Keystone::EldritchBattery)
        && !is_low_life(pob)
        && pob.stat_at_least(
            Stat::EnergyShield,
            pob.stat_parse(Stat::LifeUnreserved).unwrap_or(0.0) * 0.25,
        )
}

pub fn hp_pool<T: PathOfBuilding>(pob: &T) -> u32 {
    let mut ehp = pob.stat_parse(Stat::LifeUnreserved).unwrap_or(1);

    if pob.has_keystone(Keystone::ChaosInoculation) {
        // CI doesn't work with MoM
        return 1 + pob.stat_parse(Stat::EnergyShield).unwrap_or(0);
    }

    if !pob.has_keystone(Keystone::EldritchBattery) {
        ehp += pob.stat_parse(Stat::EnergyShield).unwrap_or(0);
    }

    if pob.has_keystone(Keystone::MindOverMatter) {
        let mom_percent = 0.35; // TODO figure out exact %

        let mut mana = pob.stat_parse(Stat::ManaUnreserved).unwrap_or(0);
        if pob.has_keystone(Keystone::EldritchBattery) {
            mana += pob.stat_parse(Stat::EnergyShield).unwrap_or(0);
        }

        // https://old.reddit.com/r/pathofexile/comments/8lio2g/how_to_calculate_ehp_with_mom/dzg03d1/
        // Maxiumum amount of Mana
        let max_mana_soak = (ehp as f32 * (mom_percent / (1.0 - mom_percent))) as u32;

        // More mana than max_mana_soak does not add to total EHP
        ehp += mana.min(max_mana_soak);
    }

    ehp
}

#[derive(Default)]
pub struct TitleConfig {
    pub no_title: bool,
}

pub fn title<T: PathOfBuilding>(pob: &T) -> String {
    title_with_config(pob, &TitleConfig::default())
}

pub fn title_with_config<T: PathOfBuilding>(pob: &T, config: &TitleConfig) -> String {
    let mut items = Vec::with_capacity(5);

    let level = format!("Level {}", pob.level());
    if !config.no_title {
        items.push(level.as_str());
    }

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
