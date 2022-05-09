use crate::{Meta, Prefetch};
use sycamore::prelude::*;

pub struct HeadArgs {
    pub meta: Meta,
    pub prefetch: Vec<Prefetch>,
}

#[component(Head<G>)]
pub fn head(args: HeadArgs) -> View<G> {
    let meta = args.meta;
    let title = meta.title.clone();

    let preload = args
        .prefetch
        .into_iter()
        .map(|preload| {
            let typ = preload.typ();
            let href = preload.into_url();
            view! { link(rel="prefetch", href=href, as=typ) }
        })
        .collect::<Vec<_>>();
    let preload = View::new_fragment(preload);

    view! {
        title { (title) }
        meta(property="og:title", content=meta.title)
        meta(property="og:description", content=meta.description)
        meta(property="og:image", content=meta.image)
        meta(name="theme-color", content=meta.color)
        link(type="application/json+oembed", href="/oembed.json")
        (preload)
    }
}
