use crate::components::{CreatePaste, CreatePasteProps, ImportPastebin};
use sycamore::prelude::*;

#[component(IndexPage<G>)]
pub fn index_page() -> View<G> {
    view! {
        div(class="flex flex-col gap-12") {
            CreatePaste(CreatePasteProps::default())
            ImportPastebin()
        }
    }
}
