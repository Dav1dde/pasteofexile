use std::{env, fs::File, path::Path};

mod gems;

pub fn main() -> anyhow::Result<()> {
    let out_poe1 = Path::new(&env::var_os("OUT_DIR").unwrap()).join("gems.rs");
    let out_poe2 = Path::new(&env::var_os("OUT_DIR").unwrap()).join("gems2.rs");
    let data_poe1 = Path::new("data").join("gems.json");
    let data_poe2 = Path::new("data").join("gems2.json");

    gems::generate(data_poe1, &mut File::create(out_poe1)?)?;
    gems::generate(data_poe2, &mut File::create(out_poe2)?)?;

    Ok(())
}
