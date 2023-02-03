use sycamore::prelude::*;

use crate::{utils::IteratorExt, Meta, Prefetch};

pub struct HeadArgs {
    pub meta: Meta,
    pub prefetch: Vec<Prefetch>,
    pub preload: Vec<Prefetch>,
}

#[component]
pub fn Head<G: Html>(cx: Scope, args: HeadArgs) -> View<G> {
    let meta = args.meta;
    let title = meta.title.clone();
    let image = match meta.image.is_empty() {
        true => crate::assets::logo().into(),
        false => meta.image,
    };

    let preload = args
        .preload
        .into_iter()
        .map(|preload| {
            let typ = preload.typ();
            let href = preload.into_url();
            view! { cx, link(rel="preload", href=href, as=typ) }
        })
        .collect_view();

    let prefetch = args
        .prefetch
        .into_iter()
        .map(|prefetch| {
            let href = prefetch.into_url();
            view! { cx,
                link(rel="prefetch", href=href)
            }
        })
        .collect_view();

    let meta_title = meta.title.clone();
    let meta_description = meta.description.clone();
    view! { cx,
        title { (title) }
        meta(name="title", content=meta_title)
        meta(name="description", content=meta_description)
        meta(property="og:type", content="website")
        meta(property="og:site_name", content="Paste of Exile - pobb.in")
        meta(property="og:title", content=meta.title)
        meta(property="og:description", content=meta.description)
        meta(property="og:image", content=image)
        meta(name="theme-color", content=meta.color)
        link(type="application/json+oembed", href=meta.oembed)
        (preload)
        (prefetch)
    }
}
