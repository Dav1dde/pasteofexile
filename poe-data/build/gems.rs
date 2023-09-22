use std::fmt::Write;
use std::{fs::File, path::Path};

use serde::Deserialize;
use shared::ClassSet;

#[derive(Debug, Deserialize)]
struct Gem {
    id: String,
    // name: String,
    level: u8,
    color: String,
    #[serde(default)]
    vendors: Vec<Vendor>,
}

#[derive(Debug, Deserialize)]
struct Vendor {
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

    writeln!(output, "use super::{{Gem, Vendor}};")?;
    writeln!(output, "use shared::{{Color, ClassSet}};")?;

    for mut gem in data {
        let color = match gem.color.as_str() {
            "red" => "Color::Red",
            "green" => "Color::Green",
            "blue" => "Color::Blue",
            "white" => "Color::White",
            _ => anyhow::bail!("invalid gem color '{}'", gem.color),
        };

        let level = gem.level;

        let mut vendors = String::new();

        write!(vendors, "&[")?;
        gem.vendors.sort_by_key(|v| v.act);
        for vendor in gem.vendors {
            let classes = match vendor.class_ids {
                Some(class_ids) => class_ids
                    .iter()
                    .map(|id| id.parse::<shared::Class>())
                    .collect::<Result<ClassSet, _>>()?,
                None => ClassSet::all(),
            };
            let classes = format!("ClassSet::from_u8({})", classes.as_u8());

            write!(
                vendors,
                "Vendor {{ quest: {:?}, act: {}, npc: {:?}, classes: {classes} }},",
                vendor.quest, vendor.act, vendor.npc
            )?;
        }
        write!(vendors, "]")?;

        let value = format!("Gem {{ color: {color}, level: {level}, vendors: {vendors} }}");
        map.entry(gem.id, &value);
    }

    writeln!(
        output,
        "pub static GEMS: phf::Map<&'static str, Gem> = {};",
        map.build()
    )?;

    Ok(())
}
