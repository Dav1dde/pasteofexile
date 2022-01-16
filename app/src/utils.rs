#[macro_export]
macro_rules! memo {
    ($signal:ident, $x:expr) => {
        create_memo(cloned!($signal => move || {
            $x
        }))
    };
}

#[macro_export]
macro_rules! effect {
    ($signal:ident, $x:expr) => {
        create_effect(cloned!($signal => move || {
            $x
        }))
    };
}
