use crate::components::{CreatePaste, CreatePasteProps};
use sycamore::prelude::*;

#[component(IndexPage<G>)]
pub fn index_page() -> View<G> {
    view! {
        CreatePaste(CreatePasteProps::default())
    }
}
