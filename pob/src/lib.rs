use anyhow::Context;
use flate2::bufread::ZlibDecoder;
use std::{
    io::{self, Read},
    str::FromStr,
};

mod passives;
mod serde;
mod stats;

pub use self::passives::Keystone;
pub use self::serde::SerdePathOfBuilding;
pub use self::stats::Stat;

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
    fn ascendancy_name(&self) -> Option<&str>;
    fn notes(&self) -> &str;

    fn stat(&self, stat: Stat) -> Option<&str>;
    fn main_skill_name(&self) -> Option<&str>;
    fn main_skill_supported_by(&self, skill: &str) -> bool;
    fn has_tree_node(&self, node: u32) -> bool;
}

pub trait PathOfBuildingExt: PathOfBuilding {
    fn ascendancy_or_class_name(&self) -> &str {
        self.ascendancy_name().unwrap_or_else(|| self.class_name())
    }

    fn main_skill_supported_by_any<T>(&self, skills: T) -> bool
    where
        T: IntoIterator,
        T::Item: AsRef<str>,
    {
        skills
            .into_iter()
            .any(|skill| self.main_skill_supported_by(skill.as_ref()))
    }

    fn stat_parse<T: FromStr>(&self, name: Stat) -> Option<T> {
        PathOfBuilding::stat(self, name).and_then(|x| x.parse().ok())
    }

    fn stat_at_least(&self, name: Stat, value: f32) -> bool {
        self.stat_parse::<f32>(name)
            .map(|v| v >= value)
            .unwrap_or(false)
    }

    fn stat_at_most(&self, name: Stat, value: f32) -> bool {
        self.stat_parse::<f32>(name)
            .map(|v| v <= value)
            .unwrap_or(false)
    }

    fn has_keystone(&self, keystone: Keystone) -> bool {
        // TODO: check on gear
        self.has_tree_node(keystone.node())
    }
}

impl<T: PathOfBuilding> PathOfBuildingExt for T {}

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
