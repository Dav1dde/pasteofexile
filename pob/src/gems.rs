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
