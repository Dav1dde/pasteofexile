use std::collections::BTreeMap;

use pob::{PathOfBuilding, SerdePathOfBuilding};
use shared::{
    model::{Node, Nodes, Paste, PasteSummary},
    PasteId, User, UserPasteId,
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
            nodes: extract_node_info(&pob),
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
                let is_private = item.metadata.as_ref().map_or(false, |m| m.private);
                !is_private || session.map(|u| &u.name) == Some(user)
            })
            .map(|item| {
                let metadata = item.metadata.unwrap_or_default();
                let id = item.name.parse().expect("only valid ids are stored");

                PasteSummary {
                    id: UserPasteId {
                        user: user.clone(),
                        id,
                    }
                    .into(),
                    title: metadata.title,
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

fn extract_node_info(pob: &impl PathOfBuilding) -> Vec<Nodes> {
    let mut data = Vec::new();
    for spec in pob.tree_specs() {
        let version = spec
            .version
            .and_then(|v| v.parse::<poe_tree::Version>().ok())
            .unwrap_or_else(poe_tree::Version::latest);

        let mut keystones = spec
            .nodes
            .iter()
            .filter_map(|&node| poe_tree::get_node(version, node))
            .filter(|node| node.kind.is_keystone())
            .map(|node| Node {
                name: node.name.to_owned(),
                stats: stats_to_owned(node.stats),
            })
            .collect::<Vec<_>>();
        keystones.sort_unstable_by(|a, b| a.name.cmp(&b.name));

        let mut masteries = BTreeMap::<&'static str, Vec<String>>::new();
        for &(node, effect) in spec.mastery_effects {
            let Some(node) = poe_tree::get_node(version, node) else {
                continue;
            };
            let Some(mastery) = node.mastery_effects.iter().find(|m| m.effect == effect) else {
                continue;
            };

            let stats: &mut Vec<_> = masteries.entry(node.name).or_default();
            stats.extend(mastery.stats.iter().map(|&s| s.to_owned()));
        }
        let masteries = masteries
            .into_iter()
            .map(|(name, stats)| Node {
                name: name.to_owned(),
                stats,
            })
            .collect();

        data.push(Nodes {
            keystones,
            masteries,
        });
    }

    data
}

fn stats_to_owned(stats: &[&str]) -> Vec<String> {
    stats.iter().map(|&s| s.to_owned()).collect()
}
