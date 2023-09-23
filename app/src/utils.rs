use std::str::FromStr;

use serde::{de::DeserializeOwned, Serialize};
use sycamore::prelude::*;
use wasm_bindgen::{JsCast, UnwrapThrowExt};

pub mod hooks;
pub mod links;
mod route;
mod storage;

pub use route::PercentRoute;
pub use storage::LocalStorage;

macro_rules! memo_cond {
    ($cx:expr, $signal:ident, $if:expr, $else:expr) => {{
        create_memo($cx, move || if *$signal.get() { $if } else { $else })
    }};
}
pub(crate) use memo_cond;

macro_rules! view_cond {
    ($cx:expr, $if:expr, { $($token:tt)* }) => {
        if $if {
            let cx = $cx;
            view! { cx, $($token)* }
        } else {
            let cx = $cx;
            view! { cx, }
        }
    };
}
pub(crate) use view_cond;

macro_rules! view_if {
    ($cx:expr, $signal:expr, { $($token:tt)* }) => {
        create_memo($cx, move || crate::utils::view_cond!($cx, *$signal.get(), { $($token)* }))
    };
}
pub(crate) use view_if;

macro_rules! try_block {
    { $($token:tt)* } => {
        (move || { $($token)* })()
    }
}
pub(crate) use try_block;

macro_rules! try_block_async {
    { $($token:tt)* } => {
        (move || async move { $($token)* })().await
    }
}
pub(crate) use try_block_async;

macro_rules! async_callback {
    ($cx:expr, { $($token:tt)* }, $filter:expr) => {{
        move |_| {
            if !($filter) {
                return;
            }

            sycamore::futures::spawn_local_scoped($cx, async move {
                $($token)*
            })
        }
    }};
}
pub(crate) use async_callback;
use web_sys::HtmlAnchorElement;

/// Custom click handler that stops click propagation,
/// and handles a router navigation.
///
/// This is required if there are multiple nested anchor tags.
pub fn on_click_anchor(ev: web_sys::Event) {
    let anchor = ev
        .current_target()
        .unwrap_throw()
        .dyn_into::<HtmlAnchorElement>()
        .unwrap_throw();
    sycamore_router::navigate(&anchor.get_attribute("href").unwrap_throw());
    ev.stop_propagation();
    ev.prevent_default();
}

pub fn document<T: JsCast>() -> T {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .unchecked_into()
}

/// Checks if the current viewport size is at least `md:`.
pub fn is_at_least_medium_breakpoint() -> bool {
    web_sys::window()
        .and_then(|window| window.inner_width().ok())
        .and_then(|val| val.as_f64())
        .map(|width| width >= 768.0)
        .unwrap_or(true)
}

pub fn from_ref<T: JsCast>(node_ref: &NodeRef<impl GenericNode>) -> T {
    if let Some(node) = node_ref.try_get::<HydrateNode>() {
        node.unchecked_into()
    } else {
        node_ref.get::<DomNode>().unchecked_into()
    }
}

pub fn try_from_ref<T: JsCast>(node_ref: &NodeRef<impl GenericNode>) -> Option<T> {
    if let Some(node) = node_ref.try_get::<HydrateNode>() {
        return Some(node.unchecked_into());
    }

    node_ref
        .try_get::<DomNode>()
        .map(|node| node.unchecked_into())
}

pub fn find_text(element: &web_sys::Element, selector: &str) -> Option<String> {
    element
        .query_selector(selector)
        .ok()
        .flatten()
        .and_then(|e| e.text_content())
}

pub fn find_attribute<T: FromStr>(element: &web_sys::Element, attribute: &str) -> Option<T> {
    element
        .query_selector(&format!("[{attribute}]"))
        .ok()
        .flatten()
        .and_then(|e| e.get_attribute(attribute))
        .filter(|v| !v.is_empty())
        .and_then(|v| v.parse().ok())
}

pub fn deserialize_attribute<T: DeserializeOwned>(
    element: &web_sys::Element,
    attribute: &str,
) -> Option<T> {
    let attr = element
        .query_selector(&format!("[{attribute}]"))
        .ok()
        .flatten()
        .and_then(|e| e.get_attribute(attribute))?;

    deserialize_from_attribute(&attr)
}

pub fn deserialize_from_attribute<T: DeserializeOwned>(data: &str) -> T {
    // TODO: maybe custom encoding instead of base64, just swap " and @ (a different character)
    let data = base64::decode_config(data, base64::URL_SAFE_NO_PAD).expect("b64 decode");
    let data = String::from_utf8(data).expect("utf8");

    serde_json::from_str(&data).unwrap_or_else(|e| {
        let into = std::any::type_name::<T>();
        panic!("deserialize {data} into {into}: {e:?}")
    })
}

pub fn serialize_for_attribute<G: Html>(value: &(impl Serialize + ?Sized)) -> String {
    if G::IS_BROWSER {
        return String::new();
    }

    serialize_json_b64(value)
}

pub fn serialize_json_b64(value: &(impl Serialize + ?Sized)) -> String {
    base64::encode_config(
        serde_json::to_string(&value).expect("serialize in for attribute"),
        base64::URL_SAFE_NO_PAD,
    )
}

pub fn open_in_new_tab(url: &str) {
    web_sys::window()
        .unwrap_throw()
        .open_with_url_and_target(url, "_blank")
        .unwrap_throw();
}

pub fn wiki_url(page: &str) -> String {
    crate::consts::POE_WIKI.to_owned() + page
}

pub fn open_wiki_page(page: &str) {
    open_in_new_tab(&wiki_url(page));
}

pub trait IteratorExt: Iterator {
    fn collect_view<G: Html>(self) -> View<G>
    where
        Self: Sized,
        Self::Item: Into<View<G>>,
    {
        let views = self.map(Into::into).collect();
        View::new_fragment(views)
    }
}

impl<T: ?Sized> IteratorExt for T where T: Iterator {}

pub fn pretty_date_ts(ts: u64) -> String {
    let now = js_sys::Date::new_0().get_time();
    pretty_date(match ts > 0 {
        true => (now - ts as f64) as i64,
        false => -1,
    })
}

pub fn pretty_date(diff_in_ms: i64) -> String {
    if diff_in_ms < 0 {
        return String::new();
    }
    let diff_in_ms = diff_in_ms as u64;
    let seconds = diff_in_ms / 1000;
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let diff_days = hours / 24;

    let diff_seconds = seconds % 60;
    let diff_minutes = minutes % 60;
    let diff_hours = hours % 24;

    match (diff_days, diff_hours, diff_minutes, diff_seconds) {
        (0, 0, 0, s) => match s {
            0..=29 => "just now".into(),
            30.. => format!("{s} seconds ago"),
        },
        (0, 0, m, _) => match m {
            1 => "a minute ago".into(),
            _ => format!("{m} minutes ago"),
        },
        (0, h, _, _) => match h {
            1 => "an hour ago".into(),
            _ => format!("{h} hours ago"),
        },
        (d, _, _, _) => match d {
            1 => "a day ago".into(),
            0..=13 => format!("{d} days ago"),
            14..=61 => format!("{} weeks ago", d / 7),
            62..=729 => format!("{} months ago", d / 31),
            730.. => format!("{} years ago", d / 365),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECS: i64 = 1000;
    const MINS: i64 = SECS * 60;
    const HOURS: i64 = MINS * 60;
    const DAYS: i64 = HOURS * 24;
    const WEEKS: i64 = DAYS * 7;
    const MONTHS: i64 = DAYS * 31;
    const YEARS: i64 = DAYS * 365;

    #[test]
    fn test_pretty_date() {
        assert_eq!(pretty_date(-1), "");
        assert_eq!(pretty_date(0), "just now");
        assert_eq!(pretty_date(1), "just now");
        assert_eq!(pretty_date(30 * SECS), "30 seconds ago");
        assert_eq!(pretty_date(MINS), "a minute ago");
        assert_eq!(pretty_date(3 * MINS + 5 * SECS), "3 minutes ago");
        assert_eq!(pretty_date(HOURS + 10 * MINS), "an hour ago");
        assert_eq!(pretty_date(23 * HOURS), "23 hours ago");
        assert_eq!(pretty_date(DAYS), "a day ago");
        assert_eq!(pretty_date(13 * DAYS), "13 days ago");
        assert_eq!(pretty_date(2 * WEEKS), "2 weeks ago");
        assert_eq!(pretty_date(7 * WEEKS), "7 weeks ago");
        assert_eq!(pretty_date(2 * MONTHS), "2 months ago");
        assert_eq!(pretty_date(15 * MONTHS), "15 months ago");
        assert_eq!(pretty_date(3 * YEARS), "3 years ago");
    }
}
