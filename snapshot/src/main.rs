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

time::serde::format_description!(date_format, OffsetDateTime, "[year]-[month]-[day]");

#[derive(Debug, Serialize)]
struct Row<'a> {
    name: &'a str,
    user: Option<&'a str>,
    class: &'a str,
    ascendancy: Option<&'a str>,
    main_skill: Option<&'a str>,
    version: Option<&'a str>,
    #[serde(with = "date_format::option")]
    last_modified: Option<time::OffsetDateTime>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let snapshot = File::open(&args.snapshot)?;
    let mut snapshot = ZipArchive::new(snapshot)?;

    let mut output = csv::Writer::from_path(args.output)?;
    let mut buffer = String::new();

    for index in 0..snapshot.len() {
        let mut file = snapshot.by_index(index)?;
        if !file.is_file() {
            continue;
        }

        buffer.clear();
        file.read_to_string(&mut buffer)?;

        if let Err(err) = process_file(&file, &buffer, &mut output) {
            println!("[{}]: {}", file.name(), err);
        }
    }

    Ok(())
}

fn process_file(
    file: &zip::read::ZipFile,
    content: &str,
    output: &mut csv::Writer<File>,
) -> anyhow::Result<()> {
    let pob = pob::SerdePathOfBuilding::from_export(content)?;

    let (user, name) = if let Some(path) = file.name().strip_prefix("user/") {
        let mut parts = path.split('/');
        let user = parts.next().ok_or_else(|| anyhow::anyhow!("could not parse user"))?;
        if parts.next() != Some("pastes") {
            anyhow::bail!("invalid pattern, expected directory pastes");
        }
        let name = parts.next().ok_or_else(|| anyhow::anyhow!("could not parse file for user"))?;
        (Some(user), name.to_owned())
    } else {
        (None, file.name().replace('/', ""))
    };

    let version = pob.max_tree_version();
    let row = Row {
        name: &name,
        user,
        class: pob.class_name(),
        ascendancy: pob.ascendancy_name(),
        main_skill: pob.main_skill_name(),
        version: version.as_deref(),
        last_modified: file.last_modified().to_time().ok(),
    };

    output.serialize(&row)?;

    Ok(())
}
