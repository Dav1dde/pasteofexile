use std::{borrow::Cow, marker::PhantomData};

use sycamore::prelude::*;
use thousands::Separable;

use crate::components::{PobColoredText, StaticPopup};

pub struct Element<'a> {
    name: &'static str,
    title: Option<&'static str>,
    color: Option<&'static str>,
    stat: Option<Cow<'a, str>>,
    percent: Option<Cow<'a, str>>,
    hover: Option<Cow<'a, str>>,
    values: Option<Vec<(&'static str, Cow<'a, str>)>>,
}

impl<'a> Element<'a> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            title: None,
            color: None,
            stat: None,
            percent: None,
            hover: None,
            values: None,
        }
    }

    pub fn title(mut self, value: &'static str) -> Self {
        self.title = Some(value);
        self
    }

    pub fn color(mut self, value: &'static str) -> Self {
        self.color = Some(value);
        self
    }

    pub fn hover<T>(mut self, value: Option<T>) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.hover = value.map(Into::into);
        self
    }

    pub fn stat_str<T>(mut self, value: Option<T>) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.stat = value.map(Into::into);
        self
    }

    pub fn stat_int(mut self, value: Option<f32>) -> Self {
        self.stat = value
            .map(|value| (value as i64).separate_with_commas())
            .map(Cow::Owned);
        self
    }

    pub fn stat_float(mut self, value: Option<f32>) -> Self {
        self.stat = value
            .map(|value| format!("{value:0.2}").separate_with_commas())
            .map(Cow::Owned);
        self
    }

    pub fn stat_percent<T>(mut self, value: Option<T>) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.percent = value.map(Into::into);
        self
    }

    pub fn stat_percent_int(mut self, value: Option<f32>) -> Self {
        self.percent = value
            .map(|value| format!("{}", value as i64))
            .map(Cow::Owned);
        self
    }

    pub fn stat_percent_float(mut self, value: Option<f32>) -> Self {
        self.percent = value.map(|value| format!("{value:.2}")).map(Cow::Owned);
        self
    }

    pub fn stat_percent_if<T>(mut self, ifv: bool, value: Option<T>) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        if ifv {
            self.percent = value.map(Into::into);
        }
        self
    }

    pub fn push_percent(mut self, color: &'static str, value: f32) -> Self {
        self.values
            .get_or_insert_with(Vec::new)
            .push((color, Cow::Owned(format!("{}%", value as i32))));
        self
    }

    pub fn add_to(self, v: &mut Vec<Self>) {
        v.push(self);
    }

    pub fn render_to_string(self) -> Option<String> {
        self.render_priv(StringRenderer::new())
    }

    pub fn render_to_view<G: Html>(self, cx: Scope) -> Option<View<G>> {
        self.render_priv(ViewRenderer::new(cx))
    }

    fn render_priv<R: Renderer>(self, renderer: R) -> Option<<R as Renderer>::Output> {
        if self.stat.is_some() || self.percent.is_some() {
            self.render_stat(renderer)
        } else if self.values.is_some() {
            self.render_values(renderer)
        } else {
            None
        }
    }

    fn render_stat<R: Renderer>(self, mut renderer: R) -> Option<<R as Renderer>::Output> {
        let (stat, percent) = match (self.stat, self.percent) {
            (Some(stat), percent) => {
                let percent =
                    percent.map(|sup| Fragment::with_type(FragmentType::Super, format!("{sup}%")));
                (stat.into_owned(), percent)
            }
            (None, Some(percent)) => (format!("{percent}%"), None),
            _ => return None,
        };

        renderer.push(Fragment::with_formatting(
            Formatting::default().with_title(self.title),
            self.name,
        ));
        renderer.push(": ");

        let mut sub = renderer.sub(
            Formatting::default()
                .with_class(self.color)
                .with_hover(self.hover),
        );
        sub.push(stat);
        if let Some(percent) = percent {
            sub.push(percent);
        }
        renderer.push_sub(sub);

        Some(renderer.finish())
    }

    fn render_values<R: Renderer>(self, mut renderer: R) -> Option<<R as Renderer>::Output> {
        renderer.push(Fragment::with_formatting(
            Formatting::default().with_title(self.title),
            self.name,
        ));
        renderer.push(": ");

        let values = self.values?;
        for i in 0..values.len() {
            let (color, value) = &values[i];
            let is_last = i == values.len() - 1;

            renderer.push(Fragment::with_formatting(
                Formatting::default().with_class(Some(color)),
                value.clone(),
            ));

            if !is_last {
                renderer.push("/");
            }
        }

        Some(renderer.finish())
    }
}

trait Renderer {
    type Output;

    fn push<T>(&mut self, element: T)
    where
        T: Into<Fragment>;
    fn push_sub(&mut self, element: Self);
    fn sub(&mut self, formatting: Formatting) -> Self;
    fn finish(self) -> Self::Output;
}

#[derive(Default)]
struct Formatting {
    class: Option<&'static str>,
    title: Option<&'static str>,
    hover: Option<String>,
}

impl Formatting {
    fn with_class(mut self, class: Option<&'static str>) -> Self {
        self.class = class;
        self
    }

    fn with_title(mut self, title: Option<&'static str>) -> Self {
        self.title = title;
        self
    }

    fn with_hover<T>(mut self, hover: Option<T>) -> Self
    where
        T: Into<String>,
    {
        self.hover = hover.map(Into::into);
        self
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum FragmentType {
    Text,
    Super,
}

struct Fragment {
    formatting: Formatting,
    value: String,
    typ: FragmentType,
}

impl Fragment {
    fn with_formatting<T>(formatting: Formatting, value: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            formatting,
            value: value.into(),
            typ: FragmentType::Text,
        }
    }

    fn with_type<T>(typ: FragmentType, value: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            formatting: Formatting::default(),
            value: value.into(),
            typ,
        }
    }
}

impl From<&str> for Fragment {
    fn from(value: &str) -> Self {
        Self {
            formatting: Formatting::default(),
            value: value.to_owned(),
            typ: FragmentType::Text,
        }
    }
}

impl From<String> for Fragment {
    fn from(value: String) -> Self {
        Self {
            formatting: Formatting::default(),
            value,
            typ: FragmentType::Text,
        }
    }
}

struct StringRenderer {
    views: Vec<String>,
}

impl StringRenderer {
    fn new() -> Self {
        Self { views: Vec::new() }
    }
}

impl Renderer for StringRenderer {
    type Output = String;

    fn push<T>(&mut self, fragment: T)
    where
        T: Into<Fragment>,
    {
        let fragment = fragment.into();
        let s = match fragment.typ {
            FragmentType::Text => fragment.value,
            FragmentType::Super => format!(" [{}]", fragment.value),
        };
        self.views.push(s);
    }

    fn push_sub(&mut self, element: Self) {
        self.views.extend(element.views);
    }

    fn sub(&mut self, _: Formatting) -> Self {
        StringRenderer::new()
    }

    fn finish(self) -> Self::Output {
        self.views.join("")
    }
}

struct ViewRenderer<'a, G: GenericNode> {
    cx: Scope<'a>,
    formatting: Formatting,
    views: Vec<View<G>>,
    _g: PhantomData<G>,
}

impl<'a, G: Html> ViewRenderer<'a, G> {
    fn new(cx: Scope<'a>) -> Self {
        Self {
            cx,
            formatting: Formatting::default(),
            views: Vec::new(),
            _g: PhantomData,
        }
    }

    fn with_formatting(cx: Scope<'a>, formatting: Formatting) -> Self {
        Self {
            cx,
            formatting,
            views: Vec::new(),
            _g: PhantomData,
        }
    }
}

impl<G: Html> Renderer for ViewRenderer<'_, G> {
    type Output = View<G>;

    fn push<T>(&mut self, fragment: T)
    where
        T: Into<Fragment>,
    {
        let fragment = fragment.into();

        let class = fragment.formatting.class.unwrap_or("");
        let title = fragment.formatting.title.unwrap_or("");
        let hover = fragment.formatting.hover;
        let cx = self.cx;

        let class = if hover.is_some() {
            Cow::Owned(format!("{class} underline decoration-dotted"))
        } else {
            Cow::Borrowed(class)
        };

        let mut view = match fragment.typ {
            FragmentType::Text => view! { cx, span(class=class, title=title) { (fragment.value) } },
            FragmentType::Super => view! { cx, sup(class=class, title=title) { (fragment.value) } },
        };
        if let Some(hover) = hover {
            let content = render_hover(cx, &hover);
            view = view! { cx, StaticPopup(content=content) { (view) } }
        };

        self.views.push(view);
    }

    fn push_sub(&mut self, element: Self) {
        let inner = View::new_fragment(element.views);
        let class = element.formatting.class.unwrap_or("");
        let title = element.formatting.title.unwrap_or("");
        let hover = element.formatting.hover;
        let cx = self.cx;

        let class = if hover.is_some() {
            Cow::Owned(format!("{class} underline decoration-dotted"))
        } else {
            Cow::Borrowed(class)
        };

        let mut element = view! { cx,
            span(class=class, title=title) {
                (inner)
            }
        };
        if let Some(hover) = hover {
            let content = render_hover(cx, &hover);
            element = view! { cx, StaticPopup(content=content) { (element) } }
        };

        self.views.push(element);
    }

    fn sub(&mut self, formatting: Formatting) -> Self {
        ViewRenderer::with_formatting(self.cx, formatting)
    }

    fn finish(self) -> Self::Output {
        let inner = View::new_fragment(self.views);
        let cx = self.cx;

        view! { cx, div(class="inline-block ml-3") { (inner) } }
    }
}

fn render_hover<G: Html>(cx: Scope<'_>, s: &str) -> View<G> {
    let s = s.trim();
    view! { cx,
        div(class="bg-black/[0.8] font-['FontinSmallCaps'] py-2 px-4 text-sm whitespace-pre-line") {
            PobColoredText(text=s, links=false)
        }
    }
}
