use std::collections::HashMap;

use crate::serde::model::*;
use crate::{Config, ConfigValue, Error, Keystone, Result, Stat};

#[derive(Debug)]
pub struct SerdePathOfBuilding {
    pob: PathOfBuilding,
    // TODO: quick access list (indices) for active items (?)
}

impl SerdePathOfBuilding {
    pub fn from_xml(s: &str) -> Result<Self> {
        let mut xd = quick_xml::de::Deserializer::from_reader(s.as_bytes());

        #[cfg(feature = "better-errors")]
        let pob = match serde_path_to_error::deserialize(&mut xd) {
            Ok(pob) => pob,
            Err(err) => {
                let path = err.path().to_string();
                return Err(Error::ParseXml(path, err.into_inner()));
            }
        };

        #[cfg(not(feature = "better-errors"))]
        let pob = serde::Deserialize::deserialize(&mut xd)
            .map_err(|e| Error::ParseXml("Unknown".to_owned(), e))?;

        Ok(Self { pob })
    }

    pub fn from_export(data: &str) -> Result<Self> {
        let data = crate::utils::decompress(data)?;
        Self::from_xml(&data)
    }

    fn main_skill(&self) -> Option<&Skill> {
        let mut index = self.pob.build.main_socket_group as usize;
        if index < 1 {
            // find a fallback main skill with at least some links
            let (i, _) = self
                .pob
                .skills
                .active_skills()
                .iter()
                .enumerate()
                .filter(|(_, s)| s.gems.len() >= 4)
                .filter(|(_, s)| active_skill_names(&s.gems).next().is_some())
                // max_by_key returns the last item, but we actually want the first -> rev
                .rev()
                .max_by_key(|(_, s)| s.gems.len())?;
            index = i + 1;
        }
        self.pob.skills.active_skills().get(index - 1)
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
        let skill = self.main_skill()?;

        let index = skill.main_active_skill.checked_sub(1)? as usize;
        active_skill_names(&skill.gems).nth(index)
    }

    fn main_skill_supported_by(&self, skill: &str) -> bool {
        self.main_skill()
            .iter()
            .flat_map(|x| x.support_gems())
            .any(|gem| gem.name == skill)
    }

    fn skill_sets(&self) -> Vec<crate::SkillSet<'_>> {
        let main_socket_group = self.pob.build.main_socket_group as usize; // starts at 1

        // Old PoB, emulate skill sets (all skills in one fake skill set)
        if !self.pob.skills.skills.is_empty() {
            let skills = to_skills(&self.pob.skills.skills, main_socket_group);
            if skills.is_empty() {
                return vec![];
            }

            return vec![crate::SkillSet {
                id: 1,
                title: None,
                skills,
                is_selected: true,
            }];
        }

        self.pob
            .skills
            .skill_sets
            .iter()
            .map(|ss| crate::SkillSet {
                id: ss.id,
                title: ss.title.as_deref(),
                skills: to_skills(&ss.skills, main_socket_group),
                is_selected: self.pob.skills.active_skill_set == Some(ss.id),
            })
            .filter(|ss| !ss.skills.is_empty())
            .collect()
    }

    fn item_by_id(&self, id: u16) -> Option<&str> {
        // TODO: maybe this lookup should be done with a hashmap and combined with item_sets
        self.pob
            .items
            .items
            .iter()
            .find(|item| item.id == id)
            .map(|item| item.content.content.as_str())
    }

    fn item_sets(&self) -> Vec<crate::ItemSet> {
        let items = self
            .pob
            .items
            .items
            .iter()
            .map(|item| (item.id, item.content.content.as_str()))
            .collect::<HashMap<_, _>>();

        self.pob
            .items
            .item_sets
            .iter()
            .map(|set| {
                let gear = &set.gear;
                let gear = crate::Gear {
                    weapon1: gear.weapon1.and_then(|id| items.get(&id)).copied(),
                    weapon2: gear.weapon2.and_then(|id| items.get(&id)).copied(),
                    helmet: gear.helmet.and_then(|id| items.get(&id)).copied(),
                    body_armour: gear.body_armour.and_then(|id| items.get(&id)).copied(),
                    gloves: gear.gloves.and_then(|id| items.get(&id)).copied(),
                    boots: gear.boots.and_then(|id| items.get(&id)).copied(),
                    amulet: gear.amulet.and_then(|id| items.get(&id)).copied(),
                    ring1: gear.ring1.and_then(|id| items.get(&id)).copied(),
                    ring2: gear.ring2.and_then(|id| items.get(&id)).copied(),
                    belt: gear.belt.and_then(|id| items.get(&id)).copied(),
                    flask1: gear.flask1.and_then(|id| items.get(&id)).copied(),
                    flask2: gear.flask2.and_then(|id| items.get(&id)).copied(),
                    flask3: gear.flask3.and_then(|id| items.get(&id)).copied(),
                    flask4: gear.flask4.and_then(|id| items.get(&id)).copied(),
                    flask5: gear.flask5.and_then(|id| items.get(&id)).copied(),
                    sockets: gear
                        .sockets
                        .iter()
                        .filter_map(|id| items.get(id))
                        .copied()
                        .collect(),
                };

                crate::ItemSet {
                    id: set.id,
                    title: set.title.as_deref(),
                    gear,
                    is_selected: Some(set.id) == self.pob.items.active_item_set,
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
                sockets: spec
                    .sockets
                    .sockets
                    .iter()
                    .map(|s| crate::Socket {
                        node_id: s.node_id,
                        item_id: s.item_id,
                    })
                    .collect(),
                mastery_effects: &spec.mastery_effects,
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

/// Returns an iterator of active skills as PoB sees it.
fn active_skill_names(gems: &[Gem]) -> impl Iterator<Item = &str> {
    gems.iter().flat_map(|gem| {
        let active = gem.is_active().then_some(gem.name.as_str());
        // all vaal gems are implicitly also active
        let vaal = gem.is_vaal().then(|| gem.non_vaal_name());
        // granted skills by gems (e.g. `Impending Doom` grantes `Doom Blast`.
        let granted = gem
            .skill_id
            .iter()
            .flat_map(|sid| crate::gems::granted_active_skills(sid))
            .copied();

        // Order is important here.
        // A vaal skill incldes the vall skill name in `gem.name`,
        // this needs to go first, then this adds the non vaal version as second.
        //
        // Currently only Impending Doom grants a skill,
        // if there is an active gem that grants a skill, this order may need to be re-evaluted.

        active.into_iter().chain(vaal).chain(granted)
    })
}

fn to_skills(skills: &[Skill], main_socket_group: usize) -> Vec<crate::Skill> {
    skills
        .iter()
        .enumerate()
        .map(|(index, s)| {
            let is_selected = main_socket_group == index + 1;
            to_skill(s, is_selected)
        })
        .collect()
}

fn to_skill(skill: &Skill, is_selected: bool) -> crate::Skill {
    let mut actives = 0;
    let gems = skill
        .gems
        .iter()
        .map(|g| {
            let is_selected = if g.is_active() {
                actives += 1;
                is_selected && skill.main_active_skill == actives
            } else {
                false
            };
            crate::Gem {
                name: &g.name,
                skill_id: g.skill_id.as_deref(),
                level: g.level,
                quality: g.quality,
                is_enabled: g.enabled,
                is_active: g.is_active(),
                is_support: g.is_support(),
                is_selected,
            }
        })
        .collect();

    crate::Skill {
        gems,
        label: skill.label.as_deref(),
        slot: skill.slot.as_deref(),
        is_selected,
        is_enabled: skill.enabled,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PathOfBuilding, PathOfBuildingExt};

    static V316_EMPTY: &str = include_str!("../../test/316_empty.xml");
    static V316_POISON_OCC: &str = include_str!("../../test/316_poison_occ.xml");
    static V318_SKILLSET: &str = include_str!("../../test/318_skillset.xml");
    static V319_MASTERY_EFFECTS: &str = include_str!("../../test/319_mastery_effects.xml");
    static V320_IMPENDING_DOOM: &str = include_str!("../../test/320_impending_doom.xml");

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

        // Level and quality overflow the u8, they should fall back to Default::default()
        assert_eq!(0, pob.skill_sets()[0].skills[0].gems[0].level);
        assert_eq!(0, pob.skill_sets()[0].skills[0].gems[0].quality);
        // No overflows
        assert_eq!(20, pob.skill_sets()[0].skills[0].gems[1].level);
        assert_eq!(20, pob.skill_sets()[0].skills[0].gems[1].quality);

        assert_eq!(2, pob.item_sets().len());
        assert_eq!(None, pob.item_sets()[0].title);
        assert_eq!(Some("Perfect Gear"), pob.item_sets()[1].title);

        // TODO: test configs
    }

    #[test]
    fn parse_v318_skillset() {
        let pob = SerdePathOfBuilding::from_xml(V318_SKILLSET).unwrap();

        assert_eq!(Some("Arc"), pob.main_skill_name());
        assert_eq!(3, pob.skill_sets().len());
        assert!(pob.skill_sets()[0].is_selected);
        assert_eq!(Some("Arc SS"), pob.skill_sets()[0].title);

        // TODO: assert skill sets, expose skill sets
    }

    #[test]
    fn parse_v319_mastery_effects() {
        let pob = SerdePathOfBuilding::from_xml(V319_MASTERY_EFFECTS).unwrap();

        let spec = pob.tree_specs().pop().unwrap();
        assert_eq!(spec.mastery_effects, &[(12382, 47642), (8732, 12119)]);
    }

    #[test]
    fn parse_v320_impending_doom() {
        let pob = SerdePathOfBuilding::from_xml(V320_IMPENDING_DOOM).unwrap();
        // Impending Doom is a support which grants an active skill,
        // the main active skill should be `Doom Blast`.
        assert_eq!(pob.main_skill_name(), Some("Doom Blast"));

        // The skill's `nameSpec` is empty and needs to be supplied from `crate::gems`.
        assert_eq!(pob.skill_sets()[0].skills[1].gems[0].name, "Tornado");
    }
}
