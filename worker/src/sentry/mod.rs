use std::cell::RefCell;

mod client;
mod converter;
mod layer;
mod protocol;
mod utils;

pub use self::client::Sentry;
pub use self::layer::Layer;


thread_local!(pub(crate) static SENTRY: RefCell<Option<Sentry>> = RefCell::new(None));


pub fn init(env: &worker::Env, ctx: &worker::Context, req: &worker::Request) {
    if let Some(sentry) = Sentry::from_env(env, ctx, req) {
        SENTRY.with(move |cell| cell.borrow_mut().replace(sentry));
    }
}

pub fn finish() {
    with_sentry(|sentry| sentry.finish_transaction());
}

pub fn with_sentry<F, T>(f: F) -> Option<T> where F: FnOnce(&Sentry) -> T {
    SENTRY.with(|sentry| {
        if let Some(sentry) = sentry.borrow().as_ref() {
            Some(f(sentry))
        } else {
            None
        }
    })
}

pub(crate) fn with_sentry_mut<F, T>(f: F) -> Option<T> where F: FnOnce(&mut Sentry) -> T {
    SENTRY.with(|sentry| {
        if let Some(sentry) = sentry.borrow_mut().as_mut() {
            Some(f(sentry))
        } else {
            None
        }
    })
}
