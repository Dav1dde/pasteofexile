use std::{borrow::Cow, marker::PhantomData};
use sycamore::prelude::*;
use thousands::Separable;

pub struct Element<'a> {
    name: &'static str,
    color: Option<&'static str>,
    stat: Option<Cow<'a, str>>,
    percent: Option<Cow<'a, str>>,
    values: Option<Vec<(&'static str, Cow<'a, str>)>>,
}

impl<'a> Element<'a> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            color: None,
            stat: None,
            percent: None,
            values: None,
        }
    }

    pub fn color(mut self, value: &'static str) -> Self {
        self.color = Some(value);
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
            .map(|value| format!("{:0.2}", value).separate_with_commas())
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
        self.percent = value.map(|value| format!("{:.2}", value)).map(Cow::Owned);
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

    pub fn push_percent(mut self, color: &'static str, value: &'a str) -> Self {
        self.values
            .get_or_insert_with(Vec::new)
            .push((color, Cow::Owned(format!("{}%", value))));
        self
    }

    pub fn add_to(self, v: &mut Vec<Self>) {
        v.push(self);
    }

    pub fn render_to_string(self) -> Option<String> {
        self.render_priv(StringRenderer::new())
    }

    pub fn render_to_view<G: GenericNode>(self) -> Option<View<G>> {
        self.render_priv(ViewRenderer::new())
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
                let percent = percent
                    .map(|sup| Fragment::with_type(FragmentType::Super, format!("{}%", sup)));
                (stat.into_owned(), percent)
            }
            (None, Some(percent)) => (format!("{}%", percent), None),
            _ => return None,
        };

        let color = self.color.unwrap_or("");

        renderer.push(self.name);
        renderer.push(": ");

        let mut sub = renderer.sub(color);
        sub.push(stat);
        if let Some(percent) = percent {
            sub.push(percent);
        }
        renderer.push_sub(sub);

        Some(renderer.finish())
    }

    fn render_values<R: Renderer>(self, mut renderer: R) -> Option<<R as Renderer>::Output> {
        renderer.push(self.name);
        renderer.push(": ");

        let values = self.values?;
        for i in 0..values.len() {
            let (color, value) = &values[i];
            let is_last = i == values.len() - 1;

            renderer.push(Fragment::with_formatting(*color, value.to_owned()));

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
    fn sub(&mut self, formatting: &'static str) -> Self;
    fn finish(self) -> Self::Output;
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum FragmentType {
    Text,
    Super,
}

struct Fragment {
    formatting: &'static str,
    value: String,
    typ: FragmentType,
}

impl Fragment {
    fn with_formatting<T>(formatting: &'static str, value: T) -> Self
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
            formatting: "",
            value: value.into(),
            typ,
        }
    }
}

impl From<&str> for Fragment {
    fn from(value: &str) -> Self {
        Self {
            formatting: "",
            value: value.to_owned(),
            typ: FragmentType::Text,
        }
    }
}

impl From<String> for Fragment {
    fn from(value: String) -> Self {
        Self {
            formatting: "",
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

    fn sub(&mut self, _: &'static str) -> Self {
        StringRenderer::new()
    }

    fn finish(self) -> Self::Output {
        self.views.join("")
    }
}

struct ViewRenderer<G: GenericNode> {
    formatting: &'static str,
    views: Vec<View<G>>,
    _g: PhantomData<G>,
}

impl<G: GenericNode> ViewRenderer<G> {
    fn new() -> Self {
        Self {
            formatting: "",
            views: Vec::new(),
            _g: PhantomData,
        }
    }

    fn with_formatting(formatting: &'static str) -> Self {
        Self {
            formatting,
            views: Vec::new(),
            _g: PhantomData,
        }
    }
}

impl<G: GenericNode> Renderer for ViewRenderer<G> {
    type Output = View<G>;

    fn push<T>(&mut self, fragment: T)
    where
        T: Into<Fragment>,
    {
        let fragment = fragment.into();
        let view = match fragment.typ {
            FragmentType::Text => view! { span(class=fragment.formatting) { (fragment.value) } },
            FragmentType::Super => view! { sup(class=fragment.formatting) { (fragment.value) } },
        };
        self.views.push(view);
    }

    fn push_sub(&mut self, element: Self) {
        let inner = View::new_fragment(element.views);
        let element = view! {
            span(class=element.formatting) {
                (inner)
            }
        };
        self.views.push(element);
    }

    fn sub(&mut self, formatting: &'static str) -> Self {
        ViewRenderer::with_formatting(formatting)
    }

    fn finish(self) -> Self::Output {
        let inner = View::new_fragment(self.views);
        view! {
            div(class="inline-block ml-3") {
                (inner)
            }
        }
    }
}
