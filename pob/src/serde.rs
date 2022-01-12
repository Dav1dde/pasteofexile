use serde::Deserialize;

#[derive(Debug)]
pub struct SerdePathOfBuilding {
    pob: PathOfBuilding,
}

impl crate::PathOfBuilding for SerdePathOfBuilding {
    fn from_xml(s: &str) -> anyhow::Result<Self> {
        Ok(Self {
            pob: quick_xml::de::from_str(s)?,
        })
    }

    fn level(&self) -> u8 {
        self.pob.build.level
    }

    fn class_name(&self) -> &str {
        &self.pob.build.class_name
    }

    fn ascendancy_name(&self) -> &str {
        &self.pob.build.ascend_class_name
    }

    fn notes(&self) -> &str {
        &self.pob.notes
    }
}

#[derive(Debug, Deserialize)]
struct PathOfBuilding {
    #[serde(rename = "Build")]
    build: Build,

    #[serde(rename = "Notes")]
    notes: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Build {
    level: u8,
    class_name: String,
    ascend_class_name: String,
}

#[cfg(test)]
mod tests {
    use crate::PathOfBuilding;

    use super::*;

    static V316_POISON_OCC: &str = include_str!("../test/316_poison_occ.xml");

    #[test]
    fn parse_v316_poison_occ() {
        let pob = SerdePathOfBuilding::from_xml(V316_POISON_OCC).unwrap();
        assert_eq!(96, pob.level());
        assert_eq!("Witch", pob.class_name());
        assert_eq!("Occultist", pob.ascendancy_name());
        assert_eq!(8516, pob.notes().len());
    }
}
