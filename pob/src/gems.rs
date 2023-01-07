pub fn granted_active_skills(skill_id: &str) -> &'static [&'static str] {
    match skill_id {
        "ViciousHexSupport" => &["Doom Blast"],
        _ => &[],
    }
}
