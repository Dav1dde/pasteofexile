use std::fmt::Write;
use std::{fs::File, path::Path};

use serde::Deserialize;
use shared::ClassSet;

#[derive(Debug, Deserialize)]
struct Gem {
    id: String,
    // name: String,
    color: String,
    #[serde(default)]
    rewards: Vec<Reward>,
}

#[derive(Debug, Deserialize)]
struct Reward {
    quest: String,
    act: u8,
    class_ids: Option<Vec<String>>,
    npc: String,
}

pub fn generate(output: &mut dyn std::io::Write) -> anyhow::Result<()> {
    let path = Path::new("data").join("gems.json");

    let data = File::open(path)?;
    let data: Vec<Gem> = serde_json::from_reader(data)?;

    let mut map = phf_codegen::Map::new();

    writeln!(output, "#[allow(unused_imports)]")?;
    writeln!(output, "use super::{{Color, Gem, Reward}};")?;
    writeln!(output, "use shared::{{Class, ClassSet}};")?;

    for gem in data {
        let color = match gem.color.as_str() {
            "red" => "Color::Red",
            "green" => "Color::Green",
            "blue" => "Color::Blue",
            "white" => "Color::White",
            _ => anyhow::bail!("invalid gem color '{}'", gem.color),
        };

        let mut rewards = String::new();

        write!(rewards, "&[")?;
        for reward in gem.rewards {
            let classes = reward
                .class_ids
                .unwrap_or_default()
                .iter()
                .map(|id| id.parse::<shared::Class>())
                .collect::<Result<ClassSet, _>>()?
                .as_u8();

            write!(
                rewards,
                "Reward {{ quest: {:?}, act: {}, npc: {:?}, classes: ClassSet::from_u8({classes}) }},",
                reward.quest, reward.act, reward.npc
            )?;
        }
        write!(rewards, "]")?;

        let value = format!("Gem {{ color: {color}, rewards: {rewards} }}");
        map.entry(gem.id, &value);
    }

    writeln!(
        output,
        "pub static GEMS: phf::Map<&'static str, Gem> = {};",
        map.build()
    )?;

    Ok(())
}
