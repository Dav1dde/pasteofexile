use std::rc::Rc;

use cfg_if::cfg_if;
use pob::SerdePathOfBuilding;
use sycamore::prelude::*;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use sycamore::context::use_context;

        fn get_content<G: 'static>() -> (String, Rc<SerdePathOfBuilding>) {
            let ctx = use_context::<crate::Context>();
            let content = ctx.get_paste().map(|paste| paste.content().to_owned()).unwrap();
            let pob = ctx.get_paste().map(|paste| paste.path_of_building().unwrap()).unwrap();
            (content, pob)
        }
    } else {
        use pob::PathOfBuilding;
        fn get_content<G: 'static>() -> (String, Rc<SerdePathOfBuilding>) {
            if let Some(hk) = sycamore::utils::hydrate::get_current_id() {
                let content = web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .query_selector(&format!("[data-hk=\"{}.{}\"] > textarea", hk.0, hk.1))
                    .unwrap()
                    .unwrap()
                    .inner_html(); // inner_text would be better, but inner_html is good enough
                let pob = SerdePathOfBuilding::from_export(&content).unwrap();
                (content, Rc::new(pob))
            } else {
                panic!("dynamic not implemented")
            }
        }
    }
}

#[component(PastePage<G>)]
pub fn paste_page(_content: String) -> View<G> {
    // TODO: invalid ID -> this does not work because we should be on a 404 site not here
    let (content, pob) = get_content::<G>();

    let title = crate::pob::title(&*pob);

    view! {
        div {
            h1 { (title) }
            textarea(readonly=true) {
                (content)
            }
        }
    }
}
