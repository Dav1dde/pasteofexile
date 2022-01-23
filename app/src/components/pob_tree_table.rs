use pob::SerdePathOfBuilding;
use std::rc::Rc;
use sycamore::prelude::*;

#[component(PobTreeTable<G>)]
pub fn pob_tree_table(_pob: Rc<SerdePathOfBuilding>) -> View<G> {
    view! { "Tree" }
}
