use sycamore::prelude::*;

#[component(PastePage<G>)]
pub fn paste_page(content: String) -> View<G> {
    view! {
        div {
            h1 { "Paste" }
            textarea(readonly=true) {
                (content)
            }
        }
    }
}
