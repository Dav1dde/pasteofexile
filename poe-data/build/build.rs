use std::{env, fs::File, path::Path};

mod gems;

pub fn main() -> anyhow::Result<()> {
    let gems_path = Path::new(&env::var_os("OUT_DIR").unwrap()).join("gems.rs");
    gems::generate(&mut File::create(gems_path)?)?;

    Ok(())
}
