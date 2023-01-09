use std::io::Write;
use std::{
    collections::HashMap,
    env,
    fs::{read_to_string, File},
    path::Path,
};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut out =
        File::create(Path::new(&env::var_os("OUT_DIR").unwrap()).join("app_metadata.rs"))?;

    let link_re = regex::Regex::new(r#"<link ([^>]+)>"#).unwrap();
    let index = read_to_string("../app/dist/index.html").expect("app/dist/index.html");

    let early_hints = link_re
        .captures_iter(&index)
        .filter_map(|c| to_early_hint(&c[1]))
        .collect::<Vec<_>>()
        .join(", ");
    writeln!(out, "pub const EARLY_HINTS: &str = r#\"{early_hints}\"#;")?;

    println!("cargo:rerun-if-changed=../app/dist/");

    Ok(())
}

fn to_early_hint(content: &str) -> Option<String> {
    let args_re = regex::Regex::new(r#"(?P<name>\w+)="(?P<value>[^"]*)""#).unwrap();

    let mut args = args_re
        .captures_iter(content)
        .map(|m| (m["name"].to_owned(), m["value"].to_owned()))
        .collect::<HashMap<_, _>>();

    let href = args.remove("href")?;
    let rel = args.remove("rel")?;

    if rel == "modulepreload" {
        // must be a script tag, preload it as script
        args.insert("as".to_owned(), "script".to_owned());
        args.insert("crossorigin".to_owned(), "".to_owned());
    } else if rel != "preload" && rel != "preconnect" {
        // we only care about 103 early hints here
        return None;
    }

    let args = args
        .into_iter()
        .map(|(k, v)| {
            if v.is_empty() {
                k
            } else {
                format!("{k}=\"{v}\"")
            }
        })
        .collect::<Vec<_>>()
        .join("; ");

    Some(format!("<{href}>;rel=preload;{args}"))
}
