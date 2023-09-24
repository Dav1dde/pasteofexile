/// A support/gem can grant additional skills which can be configured in PoB.
///
/// PoB treates those granted skills as normal skills 'attached' to the gem.
/// Which means offests/indices for 'active skill' includes gems with
/// a skill attached to it.
pub fn granted_active_skills(skill_id: &str) -> &'static [&'static str] {
    match skill_id {
        "SupportBluntWeapon" => &["Shockwave"],
        "ViciousHexSupport" => &["Doom Blast"],
        _ => &[],
    }
}

// Workaround for [`#3173`], PoB exports skills granted by uniques with an empty name.
//
// This list is not complete.
//
// [`#3173`]: https://github.com/PathOfBuildingCommunity/PathOfBuilding/issues/3173
pub fn skill_name_fallback(skill_id: &str) -> Option<&'static str> {
    let name = match skill_id {
        "UniqueAnimateWeapon" => "Manifest Dancing Dervish",
        "ChaosDegenAuraUnique" => "Death Aura",
        "IcestormUniqueStaff12" => "Icestorm",
        "TriggeredMoltenStrike" => "Molten Burst",
        "TriggeredSummonSpider" => "Raise Spiders",
        "AvianTornado" => "Tornado",
        _ => return None,
    };

    Some(name)
}

/// Some items are exported with the wrong id from pob,
/// See: https://github.com/PathOfBuildingCommunity/PathOfBuilding/blob/a1d3339/src/Export/Scripts/skills.lua#L122-L133
///
/// Maps the id to the correct gem id as the game knows it.
pub fn pob_id_as_game_id(id: &str) -> Option<&'static str> {
    match id {
        "Metadata/Items/Gems/Smite" => Some("Metadata/Items/Gems/SkillGemSmite"),
        "Metadata/Items/Gems/ConsecratedPath" => {
            Some("Metadata/Items/Gems/SkillGemConsecratedPath")
        }
        "Metadata/Items/Gems/VaalAncestralWarchief" => {
            Some("Metadata/Items/Gems/SkillGemVaalAncestralWarchief")
        }
        "Metadata/Items/Gems/HeraldOfAgony" => Some("Metadata/Items/Gems/SkillGemHeraldOfAgony"),
        "Metadata/Items/Gems/HeraldOfPurity" => Some("Metadata/Items/Gems/SkillGemHeraldOfPurity"),
        "Metadata/Items/Gems/ScourgeArrow" => Some("Metadata/Items/Gems/SkillGemScourgeArrow"),
        "Metadata/Items/Gems/RainOfSpores" => Some("Metadata/Items/Gems/SkillGemToxicRain"),
        "Metadata/Items/Gems/SummonRelic" => Some("Metadata/Items/Gems/SkillGemSummonRelic"),

        // Other gems, not listed in the code
        "Metadata/Items/Gems/SkillGemNewArcticArmour" => {
            Some("Metadata/Items/Gems/SkillGemArcticArmour")
        }
        _ => None,
    }
}
