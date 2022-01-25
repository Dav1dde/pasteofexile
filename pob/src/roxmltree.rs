use crate::PathOfBuilding;
use roxmltree::Document;

// TODO: give this a lifetime and make a owned and borrowed variant
pub struct RoXmlPathOfBuilding<'a> {
    // data: *const str,
    document: Document<'a>,
}

// impl Drop for RoXmlPathOfBuilding {
//     fn drop(&mut self) {
//         drop(unsafe { Box::from_raw(self.data as *mut str) })
//     }
// }

impl<'a> RoXmlPathOfBuilding<'a> {
    pub fn from_xml(xml: &'a str) -> crate::Result<RoXmlPathOfBuilding<'a>> {

        // let data = Box::into_raw(xml.into_owned().into_boxed_str());
        //let document = Document::parse(unsafe { &*data as &'static str }).unwrap();

        let document = Document::parse(xml).unwrap();

        Ok(Self { document })
    }
}


impl<'a> PathOfBuilding for RoXmlPathOfBuilding<'a> {
    fn level(&self) -> u8 {
        todo!()
    }

    fn class_name(&self) -> &str {
        todo!()
    }

    fn ascendancy_name(&self) -> Option<&str> {
        todo!()
    }

    fn notes(&self) -> &str {
        todo!()
    }

    fn stat(&self, stat: crate::Stat) -> Option<&str> {
        todo!()
    }

    fn minion_stat(&self, stat: crate::Stat) -> Option<&str> {
        todo!()
    }

    fn config(&self, config: crate::Config) -> crate::ConfigValue {
        todo!()
    }

    fn main_skill_name(&self) -> Option<&str> {
        todo!()
    }

    fn main_skill_supported_by(&self, skill: &str) -> bool {
        todo!()
    }

    fn tree_specs(&self) -> Vec<crate::TreeSpec> {
        todo!()
    }

    fn has_tree_node(&self, node: u32) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{PathOfBuilding, PathOfBuildingExt};

    use super::*;

    static V316_EMPTY: &str = include_str!("../test/316_empty.xml");
    static V316_POISON_OCC: &str = include_str!("../test/316_poison_occ.xml");

    #[test]
    fn parse_v316_empty() {
        let pob = RoXmlPathOfBuilding::from_xml(V316_EMPTY).unwrap();
        // assert_eq!(1, pob.level());
        // assert_eq!("Scion", pob.class_name());
        // assert_eq!(None, pob.ascendancy_name());
        // assert_eq!("Scion", pob.ascendancy_or_class_name());
        // assert_eq!(Some("1.8857142857143"), pob.stat(Stat::AverageDamage));
        // assert_eq!(Some("3"), pob.stat(Stat::EnduranceChargesMax));
        // assert_eq!(Some(3), pob.stat_parse(Stat::EnduranceChargesMax));
        // assert_eq!(None, pob.stat_parse::<u8>(Stat::AverageDamage));
        // TODO: test configs
    }

    #[test]
    fn parse_v316_poison_occ() {
        let pob = RoXmlPathOfBuilding::from_xml(V316_POISON_OCC).unwrap();
        // assert_eq!(96, pob.level());
        // assert_eq!("Witch", pob.class_name());
        // assert_eq!(Some("Occultist"), pob.ascendancy_name());
        // assert_eq!("Occultist", pob.ascendancy_or_class_name());
        // assert_eq!(8516, pob.notes().len());
        // assert_eq!(Some("Poisonous Concoction"), pob.main_skill_name());
        // assert!(!pob.main_skill_supported_by(pob.main_skill_name().unwrap()));
        // assert!(pob.main_skill_supported_by("Unbound Ailments"));
        // assert!(!pob.main_skill_supported_by("Unbound Ailments 2.0"));
        // assert!(pob.main_skill_supported_by("Lifetap")); // no gem_id
        // assert!(pob.minion_stat(Stat::AverageDamage).is_none());
        // assert_eq!(Some("1"), pob.minion_stat(Stat::EnduranceChargesMax));
        // TODO: test configs
    }
}
