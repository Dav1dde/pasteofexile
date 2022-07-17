use sycamore::prelude::*;
use wasm_bindgen::JsCast;

// TODO: move these `macro_export`'s to `use`

#[macro_export]
macro_rules! memo {
    ($signal:ident, $x:expr) => {
        create_memo(cloned!($signal => move || {
            $x
        }))
    };
    ($signal1:ident, $signal2:ident, $x:expr) => {
        create_memo(cloned!(($signal1, $signal2) => move || {
            $x
        }))
    };
}

#[macro_export]
macro_rules! memo_cond {
    ($signal:ident, $if:expr, $else:expr) => {{
        create_memo(cloned!($signal => move || {
            if *$signal.get() {
                $if
            } else {
                $else
            }
        }))
    }};
}

#[macro_export]
macro_rules! effect {
    ($signal:ident, $x:expr) => {
        create_effect(cloned!($signal => move || {
            $x
        }))
    };
    ($signal1:ident, $signal2:ident, $x:expr) => {
        create_effect(cloned!(($signal1, $signal2) => move || {
            $x
        }))
    };
}

#[macro_export]
macro_rules! try_block {
    { $($token:tt)* } => {
        (move || { $($token)* })()
    }
}

#[macro_export]
macro_rules! try_block_async {
    { $($token:tt)* } => {
        (move || async move { $($token)* })().await
    }
}

#[allow(unused)]
macro_rules! spawn_local {
    ($($id:ident),+, { $($token:tt)* }) => {
        wasm_bindgen_futures::spawn_local(cloned!($($id),+ => async move {
            $($token)*
        }))
    };
}
#[allow(unused)]
pub(crate) use spawn_local;

#[macro_export]
macro_rules! async_callback {
    ($($id:ident),+, { $($token:tt)* }, $filter:expr) => {{
        #[cfg(not(feature = "ssr"))]
        let ret = cloned!($($id),+ => move |_| {
            if !($filter) {
                return;
            }

            wasm_bindgen_futures::spawn_local(cloned!($($id),+ => async move {
                $($token)*
            }))
        });
        #[cfg(feature = "ssr")]
        let ret = |_| {};
        ret
    }};
}

macro_rules! if_browser {
    ({ $($browser:tt)* }, { $($server:tt)* }) => {{
        #[cfg(not(feature = "ssr"))] { $($browser)* }
        #[cfg(feature = "ssr")] { $($server)* }
    }};
    { $($browser:tt)* } => {{
        #[cfg(not(feature = "ssr"))] { $($browser)* }
    }};
}
pub(crate) use if_browser;

pub fn is_hydrating() -> bool {
    sycamore::utils::hydrate::get_current_id().is_some()
}

#[cfg(not(feature = "ssr"))]
pub fn document<T: JsCast>() -> T {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .unchecked_into()
}

pub fn from_ref<G: GenericNode, T: JsCast>(node_ref: &NodeRef<G>) -> T {
    if let Some(node) = node_ref.try_get::<HydrateNode>() {
        node.unchecked_into()
    } else {
        node_ref.get::<DomNode>().unchecked_into()
    }
}

pub fn find_text(element: &web_sys::Element, selector: &str) -> Option<String> {
    element
        .query_selector(selector)
        .ok()
        .flatten()
        .and_then(|e| e.text_content())
}

pub fn find_attribute(element: &web_sys::Element, attribute: &str) -> Option<String> {
    element
        .query_selector(&format!("[{attribute}]"))
        .ok()
        .flatten()
        .and_then(|e| e.get_attribute(attribute))
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
        assert_eq!(pretty_date(1 * MINS), "a minute ago");
        assert_eq!(pretty_date(3 * MINS + 5 * SECS), "3 minutes ago");
        assert_eq!(pretty_date(1 * HOURS + 10 * MINS), "an hour ago");
        assert_eq!(pretty_date(23 * HOURS), "23 hours ago");
        assert_eq!(pretty_date(1 * DAYS), "a day ago");
        assert_eq!(pretty_date(13 * DAYS), "13 days ago");
        assert_eq!(pretty_date(2 * WEEKS), "2 weeks ago");
        assert_eq!(pretty_date(7 * WEEKS), "7 weeks ago");
        assert_eq!(pretty_date(2 * MONTHS), "2 months ago");
        assert_eq!(pretty_date(15 * MONTHS), "15 months ago");
        assert_eq!(pretty_date(3 * YEARS), "3 years ago");
    }
}
