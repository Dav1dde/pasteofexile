#[cfg(not(feature = "ssr"))]
use sycamore::prelude::*;
#[cfg(not(feature = "ssr"))]
use wasm_bindgen::JsCast;

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

#[cfg(not(feature = "ssr"))]
pub fn from_ref<G: GenericNode, T: JsCast>(node_ref: NodeRef<G>) -> T {
    if let Some(node) = node_ref.try_get::<HydrateNode>() {
        node.unchecked_into()
    } else {
        node_ref.get::<DomNode>().unchecked_into()
    }
}
