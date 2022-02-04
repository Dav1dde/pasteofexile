use crate::serde::model::*;
use crate::{Config, ConfigValue, Error, Keystone, Result, Stat};

#[derive(Debug)]
pub struct SerdePathOfBuilding {
    pob: PathOfBuilding,
    // TODO: quick access list (indices) for active items (?)
}

impl SerdePathOfBuilding {
    pub fn from_xml(s: &str) -> Result<Self> {
        let pob = quick_xml::de::from_str(s).map_err(Error::ParseXml)?;
        Ok(Self { pob })
    }

    pub fn from_export(data: &str) -> Result<Self> {
        let data = crate::utils::decompress(data)?;
        Self::from_xml(&data)
    }

    fn main_skill(&self) -> Option<&Skill> {
        let index = self.pob.build.main_socket_group;
        if index < 1 {
            return None;
        }
        self.pob.skills.skills.get(index as usize - 1)
    }

    fn has_keystone_on_gear(&self, keystone: Keystone) -> bool {
        let keystone = match keystone.as_item_stat() {
            Some(keystone) => keystone,
            None => return false,
        };

        self.pob
            .items
            .items
            .iter()
            .filter(|item| {
                self.pob
                    .items
                    .slots
                    .iter()
                    .any(|slot| item.id == slot.item_id)
            })
            .flat_map(|item| item.content.content.lines())
            .any(|stat| stat == keystone)
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
            .stats
            .iter()
            .filter_map(|stat| stat.player())
            .find(|x| stat == x.name)
            .map(|stat| stat.value.as_str())
    }

    fn minion_stat(&self, stat: Stat) -> Option<&str> {
        self.pob
            .build
            .stats
            .iter()
            .filter_map(|stat| stat.minion())
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

    fn skills(&self) -> Vec<crate::Skill> {
        // starts at 1
        let main_socket_group = self.pob.build.main_socket_group as usize;

        self.pob
            .skills
            .skills
            .iter()
            .enumerate()
            .map(|(index, s)| {
                let is_selected = main_socket_group == index + 1;

                let mut actives = 0;
                let gems = s
                    .gems
                    .iter()
                    .map(|g| {
                        let is_selected = if g.is_active() {
                            actives += 1;
                            is_selected && s.main_active_skill == actives
                        } else {
                            false
                        };
                        crate::Gem {
                            name: &g.name,
                            skill_id: g.skill_id.as_deref(),
                            level: g.level,
                            quality: g.quality,
                            is_active: g.is_active(),
                            is_support: g.is_support(),
                            is_selected,
                        }
                    })
                    .collect();

                crate::Skill {
                    gems,
                    label: s.label.as_deref(),
                    slot: s.slot.as_deref(),
                    is_selected,
                    is_enabled: s.enabled,
                }
            })
            .collect()
    }

    fn tree_specs(&self) -> Vec<crate::TreeSpec> {
        self.pob
            .tree
            .specs
            .iter()
            .enumerate()
            .map(|(i, spec)| crate::TreeSpec {
                title: spec.title.as_deref(),
                url: spec.url.as_deref(),
                version: spec.version.as_deref(),
                nodes: &spec.nodes,
                active: self.pob.tree.active_spec as usize == i + 1,
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

    fn has_keystone(&self, keystone: Keystone) -> bool {
        self.has_tree_node(keystone.node()) || self.has_keystone_on_gear(keystone)
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
        assert_eq!(Some("3.16".to_owned()), pob.max_tree_version());
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

        // EB: is in a non-active item
        assert!(!pob.has_keystone(Keystone::EldritchBattery));
        // MoM is on an active item
        assert!(pob.has_keystone(Keystone::MindOverMatter));

        assert_eq!(Some("3.19".to_owned()), pob.max_tree_version());

        // TODO: test configs
    }
}
