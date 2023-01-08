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
