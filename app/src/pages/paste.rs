use crate::{async_callback, future::LocalBoxFuture, memo, pob, router::RoutedComponent, Result};
use ::pob::{Config, Keystone, PathOfBuilding, PathOfBuildingExt, SerdePathOfBuilding, Stat};
use std::borrow::Cow;
use std::rc::Rc;
use sycamore::prelude::*;
use thousands::Separable;
use wasm_bindgen::JsCast;
use web_sys::HtmlTextAreaElement;

pub struct Data {
    content: String,
    pob: Rc<SerdePathOfBuilding>,
}

impl<G: Html> RoutedComponent<G> for PastePage<G> {
    type RouteArg = String;

    fn from_context(ctx: crate::Context) -> Result<Data> {
        let paste = ctx.get_paste().unwrap();
        Ok(Data {
            content: paste.content().to_owned(),
            pob: paste.path_of_building()?,
        })
    }

    fn from_hydration(element: web_sys::Element) -> Result<Data> {
        let content = element
            .query_selector("textarea")
            .unwrap()
            .unwrap()
            .inner_html();

        let pob = Rc::new(SerdePathOfBuilding::from_export(&content)?);
        Ok(Data { content, pob })
    }

    fn from_dynamic<'a>(id: Self::RouteArg) -> LocalBoxFuture<'a, Result<Data>> {
        Box::pin(async move {
            let content = crate::api::get_paste(id).await?;
            let pob = Rc::new(SerdePathOfBuilding::from_export(&content)?);
            Ok(Data { content, pob })
        })
    }
}

#[allow(dead_code)]
#[derive(PartialEq, Eq)]
enum CopyState {
    Ready,
    Progress,
    Done,
    Failed,
}

impl CopyState {
    fn name(&self) -> &'static str {
        match self {
            Self::Ready => "Copy",
            Self::Progress => "Copy",
            Self::Done => "Copied",
            Self::Failed => "Failed",
        }
    }
}

#[component(PastePage<G>)]
pub fn paste_page(Data { content, pob }: Data) -> View<G> {
    let title = crate::pob::title(&*pob);
    let notes = pob.notes().to_owned();

    let select_all = |event: web_sys::Event| {
        let s: HtmlTextAreaElement = event.target().unwrap().unchecked_into();
        let _ = s.focus();
        s.select();
    };

    let content_ref = NodeRef::new();
    let copy_state = Signal::new(CopyState::Ready);

    // TODO: figure out Signal clones and scopes
    let copy_to_clipboard = async_callback!(
        copy_state,
        content_ref,
        {
            use crate::utils::{document, from_ref};

            copy_state.set(CopyState::Progress);

            from_ref::<_, web_sys::HtmlTextAreaElement>(content_ref).select();

            let document: web_sys::HtmlDocument = document();
            let state = if document.exec_command("copy").is_ok() {
                CopyState::Done
            } else {
                CopyState::Failed
            };

            let _ = document
                .get_selection()
                .unwrap()
                .unwrap()
                .remove_all_ranges();

            copy_state.set(state);
            gloo_timers::future::TimeoutFuture::new(1_000).await;
            copy_state.set(CopyState::Ready);
        },
        *copy_state.get() == CopyState::Ready
    );

    let btn_copy_name = memo!(copy_state, copy_state.get().name());
    let btn_copy_disabled = memo!(copy_state, *copy_state.get() != CopyState::Ready);

    let core_stats = core_stats(&pob);
    let defense = defense(&pob);
    let offense = offense(&pob);
    let config = config(&pob);

    let summary = vec![core_stats, defense, offense, config]
        .into_iter()
        .map(|stat| view! { div(class="flex-row gap-x-5") { (stat) } })
        .collect();
    let summary = View::new_fragment(summary);

    view! {
        div(class="flex flex-col md:flex-row gap-y-5 md:gap-x-3 mb-10") {
            div(class="flex-auto flex flex-col gap-y-2") {
                h1(class="text-xl mb-1 dark:text-slate-100 text-slate-900") { (title) }
                (summary)
            }
            div(class="flex flex-col flex-initial gap-y-3 md:w-96") {
                textarea(
                    ref=content_ref,
                    on:click=select_all,
                    class="flex-auto resize-none text-sm break-all outline-none max-h-40 min-h-[5rem] dark:text-slate-400 rounded-sm shadow-sm pl-1",
                    readonly=true
                ) {
                    (content)
                }
                div(class="text-right") {
                    button(
                        on:click=copy_to_clipboard,
                        disabled=*btn_copy_disabled.get(),
                        class="bg-sky-500 hover:bg-sky-700 hover:cursor-pointer px-6 py-2 text-sm rounded-lg font-semibold text-white disabled:opacity-50 disabled:cursor-not-allowed inline-flex"
                    ) { (btn_copy_name.get()) }
                }
            }
        }
        div {
            h3(class="text-lg dark:text-slate-100 text-slate-900") { "Notes" }
            pre(class="text-xs break-words whitespace-pre-line font-mono") { (notes) }
        }
    }
}

// TODO: use these stats for meta tags, return Element with multiple render funcs
fn core_stats<G: GenericNode>(pob: &SerdePathOfBuilding) -> View<G> {
    let mut elements = Vec::with_capacity(5);

    Element::new("Life")
        .color("text-rose-500")
        .stat_int(pob.stat_parse(Stat::LifeUnreserved))
        .stat_percent(pob.stat(Stat::LifeInc))
        .add_to(&mut elements);

    if pob.stat_at_least(Stat::EnergyShield, 10.0) {
        Element::new("ES")
            .color("text-cyan-200")
            .stat_int(pob.stat_parse(Stat::EnergyShield))
            .stat_percent_if(pob::is_hybrid(pob), pob.stat(Stat::EnergyShieldInc))
            .add_to(&mut elements);
    }

    Element::new("Mana")
        .color("text-blue-400")
        .stat_int(pob.stat_parse(Stat::ManaUnreserved))
        .stat_percent_if(
            pob.has_keystone(Keystone::MindOverMatter),
            pob.stat(Stat::ManaInc),
        )
        .add_to(&mut elements);

    Element::new("eHP")
        .color("text-amber-50")
        .stat_int(Some(pob::ehp(pob) as f32))
        .add_to(&mut elements);

    let elements = elements
        .into_iter()
        .filter_map(|element| element.render())
        .collect();

    View::new_fragment(elements)
}

fn defense<G: GenericNode>(pob: &SerdePathOfBuilding) -> View<G> {
    let mut elements = Vec::with_capacity(5);

    Element::new("Resistances")
        .push_percent(
            "text-orange-400",
            pob.stat(Stat::FireResistance).unwrap_or("-60"),
        )
        .push_percent(
            "text-blue-400",
            pob.stat(Stat::ColdResistance).unwrap_or("-60"),
        )
        .push_percent(
            "text-yellow-300",
            pob.stat(Stat::LightningResistance).unwrap_or("-60"),
        )
        .push_percent(
            "text-fuchsia-500",
            pob.stat(Stat::ChaosResistance).unwrap_or("-60"),
        )
        .add_to(&mut elements);

    if pob.stat_at_least(Stat::MeleeEvadeChance, 20.0) {
        Element::new("Evade")
            .color("text-amber-50")
            .stat_percent(pob.stat(Stat::MeleeEvadeChance))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::PhysicalDamageReduction, 10.0) {
        Element::new("Phys DR")
            .color("text-amber-50")
            .stat_percent(pob.stat(Stat::PhysicalDamageReduction))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::SpellSuppressionChance, 30.0) {
        Element::new("Supp")
            .color("text-amber-50")
            .stat_percent(pob.stat(Stat::SpellSuppressionChance))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::AttackDodgeChance, 20.0) {
        Element::new("Dodge")
            .color("text-amber-50")
            .stat_percent(pob.stat(Stat::AttackDodgeChance))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::SpellDodgeChance, 10.0) {
        Element::new("Spell Dodge")
            .color("text-amber-50")
            .stat_percent(pob.stat(Stat::SpellDodgeChance))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::BlockChance, 30.0) {
        Element::new("Block")
            .color("text-amber-50")
            .stat_percent(pob.stat(Stat::BlockChance))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::SpellBlockChance, 10.0) {
        Element::new("Spell Block")
            .color("text-amber-50")
            .stat_percent(pob.stat(Stat::SpellBlockChance))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::Armour, 5000.0) {
        Element::new("Armour")
            .color("text-amber-50")
            .stat_int(pob.stat_parse(Stat::Armour))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::Evasion, 5000.0) {
        Element::new("Evasion")
            .color("text-amber-50")
            .stat_int(pob.stat_parse(Stat::Evasion))
            .add_to(&mut elements);
    }

    let elements = elements
        .into_iter()
        .filter_map(|element| element.render())
        .collect();

    View::new_fragment(elements)
}

fn offense<G: GenericNode>(pob: &SerdePathOfBuilding) -> View<G> {
    let mut elements = Vec::with_capacity(5);

    Element::new("DPS")
        .color("text-amber-50")
        .stat_float(pob.stat_parse(Stat::CombinedDps))
        .add_to(&mut elements);

    // TODO: this is cast rate for spells
    Element::new("Speed")
        .color("text-amber-50")
        .stat_float(pob.stat_parse(Stat::Speed))
        .add_to(&mut elements);

    Element::new("Hit Rate")
        .color("text-amber-50")
        .stat_float(pob.stat_parse(Stat::HitRate))
        .add_to(&mut elements);

    Element::new("Hit Chance")
        .color("text-amber-50")
        .stat_percent(pob.stat(Stat::HitChance))
        .add_to(&mut elements);

    if pob::is_crit(pob) {
        Element::new("Crit")
            .color("text-amber-50")
            .stat_percent(pob.stat(Stat::CritChance))
            .add_to(&mut elements);

        if pob.stat_at_least(Stat::CritMultiplier, 1.0) {
            Element::new("Crit Multi")
                .color("text-amber-50")
                .stat_percent(pob.stat(Stat::CritMultiplier))
                .add_to(&mut elements);
        }
    }

    let elements = elements
        .into_iter()
        .filter_map(|element| element.render())
        .collect();

    View::new_fragment(elements)
}

fn config<G: GenericNode>(pob: &SerdePathOfBuilding) -> View<G> {
    let mut configs = Vec::with_capacity(5);

    let boss = pob.config(Config::Boss);
    if boss.is_true() {
        configs.push("Boss".to_owned());
    } else if let Some(boss) = boss.string() {
        configs.push(boss.to_owned());
    }

    if pob.config(Config::Focused).is_true() {
        configs.push("Focused".to_owned());
    }

    if pob.config(Config::EnemyShocked).is_true() {
        let effect = pob.config(Config::ShockEffect).number().unwrap_or(50.0) as i32;
        configs.push(format!("{}% Shock", effect));
    }

    if configs.is_empty() {
        configs.push("None".to_owned());
    }

    Element::new("Config")
        .color("text-amber-50")
        .stat_str(Some(&configs.join(", ")))
        .render()
        .unwrap_or_else(View::empty)
}

struct Element<'a> {
    name: &'static str,
    color: Option<&'static str>,
    stat: Option<Cow<'a, str>>,
    percent: Option<Cow<'a, str>>,
    values: Option<Vec<(&'static str, Cow<'a, str>)>>,
}

impl<'a> Element<'a> {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            color: None,
            stat: None,
            percent: None,
            values: None,
        }
    }

    fn color(mut self, value: &'static str) -> Self {
        self.color = Some(value);
        self
    }

    fn stat_str(mut self, value: Option<&'a str>) -> Self {
        self.stat = value.map(Cow::Borrowed);
        self
    }

    fn stat_int(mut self, value: Option<f32>) -> Self {
        self.stat = value
            .map(|value| (value as i64).separate_with_commas())
            .map(Cow::Owned);
        self
    }

    fn stat_float(mut self, value: Option<f32>) -> Self {
        self.stat = value
            .map(|value| format!("{:0.2}", value).separate_with_commas())
            .map(Cow::Owned);
        self
    }

    fn stat_percent(mut self, value: Option<&'a str>) -> Self {
        self.percent = value.map(Cow::Borrowed);
        self
    }

    fn stat_percent_if(mut self, ifv: bool, value: Option<&'a str>) -> Self {
        if ifv {
            self.percent = value.map(Cow::Borrowed);
        }
        self
    }

    fn push_percent(mut self, color: &'static str, value: &'a str) -> Self {
        self.values
            .get_or_insert_with(Vec::new)
            .push((color, Cow::Owned(format!("{}%", value))));
        self
    }

    fn add_to(self, v: &mut Vec<Self>) {
        v.push(self);
    }

    fn render<G: GenericNode>(self) -> Option<View<G>> {
        if self.stat.is_some() || self.percent.is_some() {
            self.render_stat()
        } else if self.values.is_some() {
            self.render_values()
        } else {
            None
        }
    }

    fn render_stat<G: GenericNode>(self) -> Option<View<G>> {
        let (stat, percent) = match (self.stat, self.percent) {
            (Some(stat), percent) => {
                let percent = percent
                    .map(|sup| format!("{}%", sup))
                    .map(|sup| view! { sup { (sup) } })
                    .unwrap_or_else(View::empty);
                (stat.into_owned(), percent)
            }
            (None, Some(percent)) => (format!("{}%", percent), View::empty()),
            _ => return None,
        };

        let color = self.color.unwrap_or("");

        Some(view! {
            div(class="inline-block ml-3") {
                span { (self.name) }
                span { ": " }
                span(class=color) {
                    span() { (stat) }
                    span() { (percent) }
                }
            }
        })
    }

    fn render_values<G: GenericNode>(self) -> Option<View<G>> {
        let mut fragments = Vec::with_capacity(10);

        for (color, value) in self.values? {
            let value = value.into_owned();
            fragments.push(view! { span(class=color) { (value) } });
            // TODO: maybe separators with CSS
            fragments.push(view! { span { "/" } });
        }
        fragments.pop();
        let fragments = View::new_fragment(fragments);

        Some(view! {
            div(class="inline-block ml-3") {
                span { (self.name) }
                span { ": " }
                (fragments)
            }
        })
    }
}
