use std::collections::{BTreeMap, HashMap};

use pob::{PathOfBuilding, SerdePathOfBuilding};
use shared::{
    model::{
        data::{self, NodeStat},
        Paste, PasteSummary,
    },
    GameVersion, PasteId, User, UserPasteId,
};

use crate::request_context::{Env, FromEnv, Session};

pub struct Meta {
    pub etag: String,
}

pub struct Pastes {
    pub(crate) storage: crate::storage::Storage,
}

impl FromEnv for Pastes {
    fn from_env(env: &Env) -> Option<Self> {
        Some(Self {
            storage: crate::storage::Storage::from_env(env)?,
        })
    }
}

impl Pastes {
    pub async fn get_paste(&self, id: &PasteId) -> crate::Result<Option<(Meta, Paste)>> {
        let Some(stored) = self.storage.get(id).await? else {
            return Ok(None);
        };

        let pob = SerdePathOfBuilding::from_export(&stored.content)
            .map_err(|e| crate::Error::InvalidPoB(e, String::new()))?;

        let paste = Paste {
            metadata: stored.metadata,
            last_modified: stored.last_modified,
            content: stored.content,
            data: data::Data {
                nodes: extract_node_info(&pob),
                gems: extract_gem_info(&pob),
            },
        };

        let meta = Meta {
            etag: stored.entity_id,
        };

        Ok(Some((meta, paste)))
    }

    pub async fn list_pastes(
        &self,
        session: Session<'_>,
        user: &User,
    ) -> crate::Result<(Meta, Vec<PasteSummary>)> {
        let mut pastes = self
            .storage
            .list(user)
            .await?
            .into_iter()
            .filter(|item| {
                let is_private = item.metadata.private;
                !is_private || session.map(|u| &u.name) == Some(user)
            })
            .map(|item| {
                let metadata = item.metadata;
                let id = item.name.parse().expect("only valid ids are stored");

                PasteSummary {
                    id: UserPasteId {
                        user: user.clone(),
                        id,
                    }
                    .into(),
                    title: metadata.title,
                    game_version: metadata.game_version,
                    ascendancy_or_class: metadata.ascendancy_or_class,
                    version: metadata.version,
                    main_skill_name: metadata.main_skill_name,
                    last_modified: item.last_modified,
                    rank: metadata.rank,
                    private: metadata.private,
                }
            })
            .collect::<Vec<_>>();

        pastes.sort_unstable_by(|a, b| {
            b.rank
                .cmp(&a.rank)
                .then(b.last_modified.cmp(&a.last_modified))
        });

        let etag = pastes
            .first()
            .map(|f| format!("{}-{}", pastes.len(), f.last_modified))
            .unwrap_or_else(|| "empty".to_owned());
        let meta = Meta { etag };

        Ok((meta, pastes))
    }
}

fn extract_node_info(pob: &impl PathOfBuilding) -> Vec<data::Nodes> {
    let mut data = Vec::new();
    for spec in pob.tree_specs() {
        let version = spec
            .version
            .and_then(|v| v.parse::<poe_tree::Version>().ok())
            .unwrap_or_else(poe_tree::Version::latest);

        let mut keystones = spec
            .nodes
            .iter()
            .filter_map(|&id| Some((id, poe_tree::get_node(version, id)?)))
            .filter(|(_, node)| {
                node.kind.is_keystone() || node.kind.is_alternate_ascendancy_notable()
            })
            .map(|(id, node)| data::Node {
                name: node.name.to_owned(),
                icon: node.icon.map(|icon| icon.to_owned()),
                stats: stats_to_owned(id, node.stats),
                sort: node.kind.is_keystone().then_some(0),
            })
            .collect::<Vec<_>>();
        keystones.sort_unstable_by(|a, b| (a.sort, &a.name).cmp(&(b.sort, &b.name)));

        struct MasteryNode {
            icon: Option<String>,
            stats: Vec<NodeStat>,
        }
        let mut masteries = BTreeMap::<&'static str, MasteryNode>::new();
        for &(node_id, effect) in spec.mastery_effects {
            let Some(node) = poe_tree::get_node(version, node_id) else {
                continue;
            };
            let Some(mastery) = node.mastery_effects.iter().find(|m| m.effect == effect) else {
                continue;
            };

            let v = masteries.entry(node.name).or_insert_with(|| MasteryNode {
                icon: node.icon.map(|icon| icon.to_owned()),
                stats: Vec::new(),
            });
            v.stats.extend(stats_to_owned(node_id, mastery.stats));
        }
        let masteries = masteries
            .into_iter()
            .map(|(name, node)| data::Node {
                name: name.to_owned(),
                icon: node.icon,
                stats: node.stats,
                sort: None,
            })
            .collect();

        data.push(data::Nodes {
            keystones,
            masteries,
        });
    }

    data
}

fn stats_to_owned(id: u32, stats: &[&str]) -> Vec<NodeStat> {
    stats
        .iter()
        .map(|&s| NodeStat {
            id,
            text: s.to_owned(),
        })
        .collect()
}

fn extract_gem_info(pob: &impl PathOfBuilding) -> HashMap<String, data::Gem> {
    let gems = pob
        .skill_sets()
        .into_iter()
        .flat_map(|ss| ss.skills)
        .flat_map(|skill| skill.gems);

    let gem_fn = match pob.game_version() {
        GameVersion::One => poe_data::gems::by_id_poe1,
        GameVersion::Two => poe_data::gems::by_id_poe2,
    };

    let mut result = HashMap::new();
    for gem in gems {
        let Some(gem_id) = gem.gem_id else {
            continue;
        };

        if result.contains_key(gem_id) {
            continue;
        }

        let Some(gem_data) = gem_fn(gem_id) else {
            tracing::info!("no gem data {gem:?}");
            continue;
        };

        let vendors = gem_data
            .vendors(pob.class())
            .map(|vendor| data::Vendor {
                act: vendor.act,
                npc: vendor.npc.to_owned(),
                quest: vendor.quest.to_owned(),
            })
            .collect();

        result.insert(
            gem_id.to_owned(),
            data::Gem {
                name: gem_data.name.to_owned(),
                level: gem_data.level,
                color: gem_data.color,
                vendors,
            },
        );
    }
    result
}
