use crate::components::PobColoredText;
use crate::{Prefetch, ResponseContext};
use pob::{PathOfBuilding, SerdePathOfBuilding, TreeSpec};
use std::{any::TypeId, rc::Rc};
use sycamore::prelude::*;

struct Tree {
    name: String,
    image_url: String,
}

#[component(PobTreePreview<G>)]
pub fn pob_tree_preview(pob: Rc<SerdePathOfBuilding>) -> View<G> {
    let trees = pob
        .tree_specs()
        .into_iter()
        .map(|spec| {
            let name = spec.title.unwrap_or("<Default>").to_owned();
            let image_url = get_tree_url(&spec).unwrap();
            Tree { name, image_url }
        })
        .collect::<Vec<_>>();

    if trees.is_empty() {
        return view! {};
    }

    if TypeId::of::<G>() == TypeId::of::<SsrNode>() {
        for tree in &trees {
            ResponseContext::preload(Prefetch::Image(tree.image_url.clone()));
        }
    }

    let value = trees.get(0).unwrap().image_url.clone();
    let value = Signal::new(value);

    let select = trees.into_iter()
        .enumerate()
        .map(|(i, t)| {
            let selected = i == 0;
            view! { option(value=t.image_url, selected=selected) { span { PobColoredText(t.name) } } }
        })
        .collect::<Vec<_>>();
    let select = View::new_fragment(select);

    view! {
        select(class="sm:ml-3 mt-1 mb-2", bind:value=value.clone()) { (select) }
        object(data=value.get(), type="image/svg+xml", class="w-full h-full h-[370px] md:h-[500px]") {
        }
    }
}

fn get_tree_url(spec: &TreeSpec) -> Option<String> {
    spec.url
        .and_then(|url| url.rsplit_once('/'))
        .map(|(_, data)| data)
        .zip(spec.version)
        // .map(|(data, ver)| format!("https://tree.pobb.in/{ver}/{data}"))
        .map(|(data, ver)| format!("http://192.168.33.6:8786/{ver}/{data}"))
}
