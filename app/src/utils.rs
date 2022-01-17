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
        create_effect(cloned!($signal => move || {
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

pub fn is_hydrating() -> bool {
    sycamore::utils::hydrate::get_current_id().is_some()
}
