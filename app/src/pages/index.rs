use sycamore::prelude::*;

use crate::components::{CreatePaste, CreatePasteProps, ImportPastebin};

#[component(IndexPage<G>)]
pub fn index_page() -> View<G> {
    view! {
        div(class="flex flex-col gap-12") {
            CreatePaste(CreatePasteProps::default())
            ImportPastebin()
        }
    }
}
