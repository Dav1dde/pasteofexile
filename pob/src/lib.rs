use anyhow::Context;
use flate2::bufread::ZlibDecoder;
use std::io::{self, Read};

mod serde;

pub use self::serde::SerdePathOfBuilding;

pub trait PathOfBuilding {
    fn from_xml(xml: &str) -> anyhow::Result<Self>
    where
        Self: Sized;
    fn from_export(data: &str) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let inp = base64::decode_config(data.trim(), base64::URL_SAFE).context("base64 decode")?;
        let deflated = deflate(&inp).context("deflate")?;

        Self::from_xml(&deflated).context("parse from xml")
    }

    fn level(&self) -> u8;

    fn class_name(&self) -> &str;
    fn ascendancy_name(&self) -> &str;
    fn notes(&self) -> &str;
}

impl std::fmt::Debug for dyn PathOfBuilding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PathOfBuilding")
            .field("level", &self.level())
            .field("ascendancy_name", &self.ascendancy_name())
            .finish()
    }
}

fn deflate(inp: &[u8]) -> io::Result<String> {
    let mut deflater = ZlibDecoder::new(inp);
    let mut s = String::new();
    deflater.read_to_string(&mut s)?;
    Ok(s)
}
