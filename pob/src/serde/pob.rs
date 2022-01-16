use crate::serde::model::*;
use crate::Stat;

#[derive(Debug)]
pub struct SerdePathOfBuilding {
    pob: PathOfBuilding,
}

impl SerdePathOfBuilding {
    fn main_skill(&self) -> Option<&Skill> {
        let index = self.pob.build.main_socket_group;
        if index < 1 {
            return None;
        }
        self.pob.skills.skills.get(index as usize - 1)
    }
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

    fn ascendancy_name(&self) -> Option<&str> {
        if self.pob.build.ascend_class_name != "None" {
            Some(&self.pob.build.ascend_class_name)
        } else {
            None
        }
    }

    fn notes(&self) -> &str {
        &self.pob.notes
    }

    fn stat(&self, stat: Stat) -> Option<&str> {
        self.pob
            .build
            .stats
            .iter()
            .find(|x| stat == x.name)
            .map(|stat| stat.value.as_str())
    }

    fn main_skill_name(&self) -> Option<&str> {
        self.main_skill()
            .and_then(|skill| {
                let index = skill.main_active_skill;
                match index {
                    0 => None,
                    index => skill.active_gems().nth(index as usize - 1),
                }
            })
            .map(|gem| gem.name.as_str())
    }

    fn main_skill_supported_by(&self, skill: &str) -> bool {
        self.main_skill()
            .iter()
            .flat_map(|x| x.support_gems())
            .any(|gem| gem.name == skill)
    }

    fn has_tree_node(&self, node: u32) -> bool {
        let index = self.pob.tree.active_spec;
        if index < 1 {
            return false;
        }
        self.pob
            .tree
            .specs
            .get(index as usize - 1)
            .map(|spec| spec.nodes.contains(&node))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use crate::{PathOfBuilding, PathOfBuildingExt};

    use super::*;

    static V316_EMPTY: &str = include_str!("../../test/316_empty.xml");
    static V316_POISON_OCC: &str = include_str!("../../test/316_poison_occ.xml");

    #[test]
    fn parse_v316_empty() {
        let pob = SerdePathOfBuilding::from_xml(V316_EMPTY).unwrap();
        assert_eq!(1, pob.level());
        assert_eq!("Scion", pob.class_name());
        assert_eq!(None, pob.ascendancy_name());
        assert_eq!("Scion", pob.ascendancy_or_class_name());
        assert_eq!(Some("1.8857142857143"), pob.stat(Stat::AverageDamage));
        assert_eq!(Some("3"), pob.stat(Stat::EnduranceChargesMax));
        assert_eq!(Some(3), pob.stat_parse(Stat::EnduranceChargesMax));
        assert_eq!(None, pob.stat_parse::<u8>(Stat::AverageDamage));
    }

    #[test]
    fn parse_v316_poison_occ() {
        let pob = SerdePathOfBuilding::from_xml(V316_POISON_OCC).unwrap();
        assert_eq!(96, pob.level());
        assert_eq!("Witch", pob.class_name());
        assert_eq!(Some("Occultist"), pob.ascendancy_name());
        assert_eq!("Occultist", pob.ascendancy_or_class_name());
        assert_eq!(8516, pob.notes().len());
        assert_eq!(Some("Poisonous Concoction"), pob.main_skill_name());
        assert!(pob.main_skill_supported_by("Unbound Ailments"));
        assert!(!pob.main_skill_supported_by("Unbound Ailments 2.0"));
    }
}
