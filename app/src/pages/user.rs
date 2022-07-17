use crate::{
    components::{PasteToolbox, PasteToolboxProps},
    future::LocalBoxFuture,
    memo_cond,
    router::RoutedComponent,
    utils::{find_attribute, if_browser, pretty_date},
    Meta, Result,
};
use shared::model::{PasteSummary, UserPasteId};
use std::rc::Rc;
use sycamore::prelude::*;

pub struct Data {
    name: String,
    pastes: Vec<PasteSummary>,
}

impl<G: Html> RoutedComponent<G> for UserPage<G> {
    type RouteArg = String;

    fn from_context(name: Self::RouteArg, ctx: crate::Context) -> Result<Data> {
        Ok(Data {
            name,
            pastes: ctx.get_user().unwrap().to_vec(),
        })
    }

    fn from_hydration(name: Self::RouteArg, element: web_sys::Element) -> Result<Data> {
        let ssr = find_attribute(&element, "data-ssr").unwrap_or_default();
        // TODO: maybe custom encoding instead of base64, just swap " and @ (a different character)
        let ssr = base64::decode_config(ssr, base64::URL_SAFE_NO_PAD).expect("b64 decode");
        let ssr = String::from_utf8(ssr).expect("utf8");

        let pastes = serde_json::from_str(&ssr).expect("deserialize");

        Ok(Data { name, pastes })
    }

    fn from_dynamic<'a>(name: Self::RouteArg) -> LocalBoxFuture<'a, Result<Data>> {
        Box::pin(async move {
            let pastes = crate::api::get_user(&name).await?;
            Ok(Data { name, pastes })
        })
    }

    fn meta(Data { name, .. }: &Data) -> Result<Meta> {
        let title = format!("{name}'s builds").into();
        let description = format!("{name}'s builds").into();
        // TODO: pobbin logo
        let image = "".into();
        let color = "";

        Ok(Meta {
            title,
            description,
            image,
            color,
        })
    }
}

#[component(UserPage<G>)]
pub fn user_page(Data { name, pastes }: Data) -> View<G> {
    let data_ssr = if_browser!({ String::new() }, {
        base64::encode_config(
            serde_json::to_string(&pastes).unwrap(),
            base64::URL_SAFE_NO_PAD,
        )
    });

    let p = pastes
        .into_iter()
        .map(Rc::new)
        .map(summary_to_view)
        .collect();
    let p = View::new_fragment(p);

    view! {
        h1(class="text-amber-50 text-xl mb-4") {
            span { (name) }
            span { "'s builds" }
        }
        div(data-ssr=data_ssr,
            class="flex flex-col gap-2") {
            (p)
        }
    }
}

fn summary_to_view<G: GenericNode + Html>(summary: Rc<PasteSummary>) -> View<G> {
    let deleted = Signal::new(false);

    let url = summary.to_url();
    let asc = crate::assets::ascendancy_image(&summary.ascendancy).unwrap_or("");

    let id = UserPasteId {
        id: summary.id.clone(),
        user: summary.user.clone().unwrap(),
    };
    let open_in_pob_url = id.to_pob_open_url();

    let now = js_sys::Date::new_0().get_time();
    let diff_in_ms = if summary.last_modified > 0 {
        (now - summary.last_modified as f64) as i64
    } else {
        -1
    };

    let summary2 = summary.clone();
    let summary3 = summary.clone();

    let toolbox = PasteToolboxProps {
        id,
        on_delete: deleted.clone(),
    };

    // TODO: don't just hide the element, remove it
    let css = memo_cond!(deleted, "hidden", "p-3 even:bg-slate-700");

    view! {
        div(class=*css.get()) {
            div(class="flex flex-wrap gap-4 items-center") {
                img(src=asc,
                    width=50, height=50,
                    class="rounded-full h-min",
                    onerror="this.style.visibility='hidden'") {}
                a(href=url, class="flex-auto basis-52 text-slate-200 flex flex-col gap-3") {
                    span(class="text-amber-50") { (summary.title) sup(class="ml-1") { (summary2.version) } }
                    span() { (summary3.main_skill_name) }
                }
                div(class="flex-1 flex flex-col items-end justify-between gap-2 whitespace-nowrap") {
                    a(
                        href=open_in_pob_url,
                        title="Open build in Path of Building",
                        class="bg-sky-500 hover:bg-sky-700 hover:cursor-pointer w-fit px-6 py-2 text-sm rounded-lg font-semibold text-white disabled:opacity-50 disabled:cursor-not-allowed inline-flex hidden sm:block"
                     ) { "Open in PoB" }

                    PasteToolbox(toolbox)

                    div(class="text-right text-sm text-slate-400") {
                        (pretty_date(diff_in_ms))
                    }
                }
            }
        }
    }
}
