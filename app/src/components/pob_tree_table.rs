use pob::{PathOfBuilding, TreeSpec};
use sycamore::prelude::*;

use crate::{build::Build, components::PobColoredText};

#[component(PobTreeTable<G>)]
pub fn pob_tree_table(build: Build) -> View<G> {
    let rows = build
        .tree_specs()
        .into_iter()
        // TODO: reject pastes that do not go to this domain
        .filter(filter_valid_url)
        .map(|spec| {
            if spec.nodes.len() == 2 {
                // Empty tree: assume this is just a separator
                return view! { div(dangerously_set_inner_html="&nbsp;") {} div() {} };
            }

            let title = spec.title.unwrap_or("<Default>").to_owned();

            let title = match spec.url {
                Some(url) => {
                    let url = url.to_owned();
                    view! { a(
                        href=url,
                        rel="external",
                        target="_blank",
                        class="text-sky-500 dark:text-sky-400 hover:underline"
                    ) { PobColoredText(title) } }
                }
                None => {
                    view! { span(class="dark:text-amber-50 text-slate-800") { PobColoredText(title) } }
                }
            };

            let (nodes, level) = resolve_level(spec.nodes.len());
            let description = format!("Level {level} ({nodes} passives)");
            view! {
                div(class=if spec.active { "font-bold" } else { "" }) { (title) }
                div(class="mb-3 sm:mb-0") { (description) }
            }
        })
        .collect();

    let rows = View::new_fragment(rows);

    // TODO: try flexbox with 50% 50%
    view! {
        div(class="grid grid-cols-1 overflow-x-auto sm:grid-cols-[minmax(max-content,_350px)_max-content] gap-x-8 sm:gap-y-1 sm:ml-3") {
            (rows)
        }
    }
}

fn filter_valid_url(spec: &TreeSpec) -> bool {
    !spec
        .url
        .map(|url| url.starts_with("https://pathofexile.com/"))
        .unwrap_or(false)
}

// TODO: needs auto-generated node information for ascendancies
fn resolve_level(allocated: usize) -> (usize, usize) {
    if allocated == 0 {
        return (0, 0);
    }

    // character start node
    let allocated = allocated - 1;

    // points count towards allocated but aren't available skill tree points
    let asc = match allocated {
        0..=38 => 0,
        39..=69 => 3, // 2 points + ascendancy start node
        70..=90 => 5,
        91..=98 => 7,
        _ => 9,
    };

    // TODO: check for bandits
    let bandits = match allocated {
        0..=21 => 0,
        _ => 2,
    };

    let quests = match allocated - asc - bandits {
        0..=11 => 0,
        12..=23 => 2,
        24..=34 => 3,
        35..=44 => 5,
        45..=49 => 6,
        50..=57 => 8,
        58..=64 => 11,
        65..=73 => 14,
        74..=80 => 17,
        81..=85 => 19,
        _ => 22,
    };

    (allocated - asc, 1 + allocated - asc - bandits - quests)
}
