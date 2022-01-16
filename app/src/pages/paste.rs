use crate::memo;
use cfg_if::cfg_if;
use pob::SerdePathOfBuilding;
use std::rc::Rc;
use sycamore::prelude::*;

#[derive(Clone)]
struct Data {
    content: Rc<String>,
    pob: Rc<SerdePathOfBuilding>,
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use sycamore::context::use_context;

        fn get_content<G: 'static>(_id: String) -> Signal<Option<Data>> {
            let ctx = use_context::<crate::Context>();
            let content = ctx.get_paste().map(|paste| paste.content().to_owned()).unwrap();
            let pob = ctx.get_paste().map(|paste| paste.path_of_building().unwrap()).unwrap();
            Signal::new(Some(Data {
                content: Rc::new(content),
                pob,
            }))
        }
    } else {
        use pob::PathOfBuilding;
        use wasm_bindgen_futures::spawn_local;
        use futures::FutureExt;

        fn get_content<G: 'static>(id: String) -> Signal<Option<Data>> {
            if let Some(hk) = sycamore::utils::hydrate::get_current_id() {
                let content = web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .query_selector(&format!("[data-hk=\"{}.{}\"] > textarea", hk.0, hk.1))
                    .unwrap()
                    .unwrap()
                    .inner_html(); // inner_text would be better, but inner_html is good enough

                let pob = Rc::new(SerdePathOfBuilding::from_export(&content).unwrap());
                Signal::new(Some(Data {
                    content: Rc::new(content),
                    pob
                }))
            } else {
                let result = Signal::new(None);

                let result2 = result.clone();
                let future = crate::api::get_paste(id).map(move |response| {
                    // TODO: error handling
                    let content = response.unwrap();

                    let pob = Rc::new(SerdePathOfBuilding::from_export(&content).unwrap());
                    result2.set(Some(Data {
                        content: Rc::new(content),
                        pob,
                    }))
                });
                spawn_local(future);
                result
            }
        }
    }
}

#[component(PastePage<G>)]
pub fn paste_page(id: String) -> View<G> {
    // TODO: fetching this dogwater should be done one level above this page, to properly handle
    // 404 as well
    let data = get_content::<G>(id);

    // This is absolutely dogshit
    let title = memo!(data, {
        if data.get().is_none() {
            return "None".to_owned();
        }

        (*data.get())
            .as_ref()
            .map(|data| crate::pob::title(&*data.pob))
            .unwrap()
    });
    let content = memo!(data, {
        if data.get().is_none() {
            return Rc::new("None".to_owned());
        }

        (*data.get())
            .as_ref()
            .map(|data| data.content.clone())
            .unwrap()
    });

    view! {
        div {
            h1 { (*title.get()) }
            textarea(readonly=true) {
                (*content.get())
            }
        }
    }

    // view! {
    //     div {
    //         h1 { (*title.get()) }
    //         textarea(readonly=true) {
    //             (*content.get())
    //         }
    //     }
    // }
}
