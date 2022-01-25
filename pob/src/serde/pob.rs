use crate::serde::model::*;
use crate::{Config, ConfigValue, Error, Result, Stat};

#[derive(Debug)]
pub struct SerdePathOfBuilding {
    pob: PathOfBuilding,
}

impl SerdePathOfBuilding {
    pub fn from_xml(s: &str) -> Result<Self> {
        let pob = quick_xml::de::from_str(s).map_err(Error::ParseXml)?;

        Ok(Self { pob })
    }

    fn main_skill(&self) -> Option<&Skill> {
        let index = self.pob.build.main_socket_group;
        if index < 1 {
            return None;
        }
        self.pob.skills.skills.get(index as usize - 1)
    }
}

impl crate::PathOfBuilding for SerdePathOfBuilding {
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
            .player_stats
            .iter()
            .find(|x| stat == x.name)
            .map(|stat| stat.value.as_str())
    }

    fn minion_stat(&self, stat: Stat) -> Option<&str> {
        self.pob
            .build
            .minion_stats
            .iter()
            .find(|x| stat == x.name)
            .map(|stat| stat.value.as_str())
    }

    fn config(&self, config: Config) -> ConfigValue {
        self.pob
            .config
            .input
            .iter()
            .find(|x| config == x.name)
            .map(|stat| {
                if let Some(ref value) = stat.string {
                    ConfigValue::String(value)
                } else if let Some(value) = stat.number {
                    ConfigValue::Number(value)
                } else if let Some(value) = stat.boolean {
                    ConfigValue::Bool(value)
                } else {
                    ConfigValue::None
                }
            })
            .unwrap_or(ConfigValue::None)
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

    fn tree_specs(&self) -> Vec<crate::TreeSpec> {
        self.pob
            .tree
            .specs
            .iter()
            .map(|spec| crate::TreeSpec {
                title: spec.title.as_deref(),
                url: spec.url.as_deref(),
                nodes: &spec.nodes,
            })
            .collect()
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
        // TODO: test configs
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
        assert!(!pob.main_skill_supported_by(pob.main_skill_name().unwrap()));
        assert!(pob.main_skill_supported_by("Unbound Ailments"));
        assert!(!pob.main_skill_supported_by("Unbound Ailments 2.0"));
        assert!(pob.main_skill_supported_by("Lifetap")); // no gem_id
        assert!(pob.minion_stat(Stat::AverageDamage).is_none());
        assert_eq!(Some("1"), pob.minion_stat(Stat::EnduranceChargesMax));
        // TODO: test configs
    }
}
