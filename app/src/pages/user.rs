use crate::{
    components::{PasteToolbox, PasteToolboxProps},
    future::LocalBoxFuture,
    memo_cond,
    router::RoutedComponent,
    utils::{find_attribute, if_browser, pretty_date},
    Meta, Result,
};
use shared::{
    model::{PasteSummary, UserPasteId},
    User,
};
use std::rc::Rc;
use sycamore::prelude::*;

pub struct Data {
    name: User,
    pastes: Vec<PasteSummary>,
}

impl<G: Html> RoutedComponent<G> for UserPage<G> {
    type RouteArg = User;

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

    fn meta(Data { name, pastes }: &Data) -> Result<Meta> {
        let title = format!("{name}'s builds").into();

        let mut summary = pastes
            .iter()
            .take(3)
            .map(|paste| format!("\u{27A4} {}", paste.title))
            .collect::<Vec<_>>();
        if pastes.len() > 3 {
            summary.push(format!("\u{27A4} .. {} more builds", pastes.len() - 3));
        }
        if summary.is_empty() {
            summary.push("\u{27A4} there aren't any builds yet".to_owned());
        }

        let description = summary.join("\n").into();
        let image = crate::assets::logo().into();

        Ok(Meta {
            title,
            description,
            image,
            ..Default::default()
        })
    }
}

#[component(UserPage<G>)]
pub fn user_page(Data { name, pastes }: Data) -> View<G> {
    let data_ssr = if_browser!(
        String::new(),
        base64::encode_config(
            serde_json::to_string(&pastes).unwrap(),
            base64::URL_SAFE_NO_PAD,
        )
    );

    let p = pastes
        .into_iter()
        .map(Rc::new)
        .map(|summary| {
            let deleted = Signal::new(false);
            let content = memo_cond!(
                deleted,
                view! {},
                summary_to_view(summary.clone(), deleted.clone())
            );
            view! { (&*content.get()) }
        })
        .collect::<Vec<_>>();

    let p = if !p.is_empty() {
        View::new_fragment(p)
    } else {
        view! {
            span(class="text-center") { "There is nothing here .." }
        }
    };

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

fn summary_to_view<G: GenericNode + Html>(
    summary: Rc<PasteSummary>,
    on_delete: Signal<bool>,
) -> View<G> {
    let url = summary.to_url();
    let image = crate::assets::ascendancy_image(&summary.ascendancy_or_class).unwrap_or("");
    let color = crate::meta::get_color(&summary.ascendancy_or_class);

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

    let toolbox = PasteToolboxProps { id, on_delete };

    view! {
        div(class="p-3 even:bg-slate-700 border-solid border-[color:var(--col)]
                hover:border-l-4 hover:bg-[color:var(--bg-col)]",
            style=format!("--col: {color}; --bg-col: {color}66")
        ) {
            div(class="flex flex-wrap gap-4 items-center") {
                img(src=image,
                    class="asc-image rounded-full",
                    onerror="this.style.visibility='hidden'") {}
                a(href=url, class="flex-auto basis-52 text-slate-200 flex flex-col gap-3") {
                    span(class="text-amber-50") { (summary.title) sup(class="ml-1") { (summary2.version) } }
                    span() { (summary3.main_skill_name) }
                }
                div(class="
                    flex-1 sm:flex-initial flex flex-col items-end justify-between gap-2
                    whitespace-nowrap") {
                    a(
                        href=open_in_pob_url,
                        title="Open build in Path of Building",
                        class="btn btn-primary"
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
