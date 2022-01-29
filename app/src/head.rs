use crate::Meta;
use sycamore::prelude::*;

#[component(Head<G>)]
pub fn head(meta: Meta) -> View<G> {
    let title = meta.title.clone();
    view! {
        title { (title) }
        meta(property="og:title", content=meta.title)
        meta(property="og:description", content=meta.description)
        meta(property="og:image", content=meta.image)
        meta(name="theme-color", content=meta.color)
        link(type="application/json+oembed", href="/oembed.json")
    }
}
