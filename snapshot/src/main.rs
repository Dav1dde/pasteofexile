use clap::Parser;
use pob::{PathOfBuilding, PathOfBuildingExt};
use serde::Serialize;
use std::{fs::File, io::Read, path::PathBuf};
use zip::read::ZipArchive;

#[derive(Debug, Parser)]
struct Args {
    #[clap(short, long)]
    output: PathBuf,

    snapshot: PathBuf,
}

#[derive(Debug, Serialize)]
struct Row<'a> {
    name: &'a str,
    user: Option<&'a str>,
    class: &'a str,
    ascendancy: Option<&'a str>,
    main_skill: Option<&'a str>,
    version: Option<&'a str>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let snapshot = File::open(&args.snapshot)?;
    let mut snapshot = ZipArchive::new(snapshot)?;

    let mut output = csv::Writer::from_path(args.output)?;

    for index in 0..snapshot.len() {
        let mut file = snapshot.by_index(index)?;
        if !file.is_file() {
            continue;
        }

        let mut content = String::new();
        file.read_to_string(&mut content)?;

        if let Err(err) = process_file(file.name(), &content, &mut output) {
            println!("[{}]: {}", file.name(), err);
        }
    }

    Ok(())
}

fn process_file(name: &str, content: &str, output: &mut csv::Writer<File>) -> anyhow::Result<()> {
    let pob = pob::SerdePathOfBuilding::from_export(content)?;

    let version = pob.max_tree_version();
    let row = Row {
        name: &name.replace('/', ""),
        user: None,
        class: pob.class_name(),
        ascendancy: pob.ascendancy_name(),
        main_skill: pob.main_skill_name(),
        version: version.as_deref(),
    };

    output.serialize(&row)?;

    Ok(())
}
