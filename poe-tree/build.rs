use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use poe_api::api::SkillTreeData;

fn get_trees() -> impl Iterator<Item = String> {
    env::vars().filter_map(|(name, _)| {
        name.strip_prefix("CARGO_FEATURE_TREE_")
            .map(|x| x.replace('_', "."))
    })
}

pub fn main() -> anyhow::Result<()> {
    for version in get_trees() {
        let data_path = Path::new(".").join("data").join(format!("{version}.json"));

        let data = std::fs::read_to_string(data_path)?;
        let data: SkillTreeData = serde_json::from_str(&data)?;

        let dest_path = Path::new(&env::var_os("OUT_DIR").unwrap())
            .join(format!("tree{}.rs", version.replace('.', "_")));
        let mut output = File::create(dest_path)?;

        generate(&data, &mut output)?;
    }

    Ok(())
}

fn generate(data: &SkillTreeData, output: &mut dyn Write) -> anyhow::Result<()> {
    let mut map = phf_codegen::Map::new();

    writeln!(output, "#[allow(unused_imports)]")?;
    writeln!(output, "use crate::{{Kind, MasteryEffect, Node}};")?;

    for node in data.nodes.values() {
        let kind = if node.is_mastery {
            "Kind::Mastery"
        } else if node.is_keystone {
            "Kind::Keystone"
        } else if node.is_notable {
            "Kind::Notable"
        } else {
            "Kind::Node"
        };

        let mastery_effects = node
            .mastery_effects
            .iter()
            .map(|me| {
                format!(
                    "MasteryEffect {{ effect: {}, stats: &{:?} }}",
                    me.effect, me.stats
                )
            })
            .collect::<Vec<_>>()
            .join(", ");

        // Only need keystone icons
        let icon = if node.is_keystone {
            node.icon.as_ref()
        } else if node.is_mastery {
            node.active_icon.as_ref()
        } else {
            None
        };
        let icon = icon
                .map(|icon| icon.strip_suffix(".png").unwrap_or(icon))
            // files on the cdn are lowercase (directly extracted from the bundle)
                .map(|icon| icon.to_lowercase());

        let n = format!(
            r#"Node {{ kind: {kind}, name: "{}", stats: &{:?}, mastery_effects: &[{mastery_effects}], icon: {icon:?} }}"#,
            node.name, node.stats
        );

        map.entry(node.skill, &n);
    }

    writeln!(
        output,
        "pub static TREE: phf::Map<u32, Node> = {};",
        map.build()
    )?;

    Ok(())
}
