use pob::{PathOfBuilding, SerdePathOfBuilding, TreeSpec};
use std::rc::Rc;
use sycamore::prelude::*;

#[component(PobTreeTable<G>)]
pub fn pob_tree_table(pob: Rc<SerdePathOfBuilding>) -> View<G> {
    let rows = pob
        .tree_specs()
        .into_iter()
        // TODO: reject pastes that do not go to this domain
        .filter(filter_valid_url)
        .map(|spec| {
            let title = spec.title.unwrap_or("<Unnamed>").to_owned();
            let title = match spec.url {
                Some(url) => {
                    let url = url.to_owned();
                    view! { a(
                        href=url,
                        rel="external",
                        class="text-sky-500 dark:text-sky-400 hover:underline"
                    ) { (title) } }
                }
                None => {
                    view! { span(class="dark:text-amber-50 text-slate-800") { (title) } }
                }
            };

            // TODO: read proper amount of nodes
            let nodes = spec.nodes.len().saturating_sub(10); // remove 10 points for ascendancy etc.
            let level = nodes.saturating_sub(23); // TODO: bandits, level progression
            let description = format!("Required Level {} ({} passive points)", level, nodes);
            view! {
                div() { (title) }
                div(class="mb-3 sm:mb-0") { (description) }
            }
        })
        .collect();

    let rows = View::new_fragment(rows);

    view! {
        div(class="grid grid-cols-1 sm:grid-cols-[auto_1fr] gap-x-8 sm:gap-y-1 sm:ml-3") {
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
