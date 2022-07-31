use std::cell::RefCell;

mod client;
mod converter;
mod layer;
mod protocol;
mod utils;

pub use self::client::Sentry;
pub use self::layer::Layer;
pub use self::protocol::{Level, Request, User};

thread_local!(pub(crate) static SENTRY: RefCell<Option<Sentry>> = RefCell::new(None));

pub struct Options {
    pub project: String,
    pub token: String,
}

pub fn init(ctx: &worker::Context, options: impl Into<Option<Options>>) -> SentryToken {
    if let Some(options) = options.into() {
        let sentry = Sentry::new(ctx.clone(), options);
        SENTRY.with(move |cell| cell.borrow_mut().replace(sentry));
        SentryToken(true)
    } else {
        SentryToken(false)
    }
}

pub struct SentryToken(bool);

impl SentryToken {
    pub fn initialized(&self) -> bool {
        self.0
    }
}

impl Drop for SentryToken {
    fn drop(&mut self) {
        if self.initialized() {
            SENTRY.with(|cell| {
                let sentry = cell.borrow_mut().take();
                if let Some(mut sentry) = sentry {
                    sentry.finish_transaction();
                }
            })
        }
    }
}

pub struct TransactionContext {
    pub name: String,
    pub op: String,
}

pub fn start_transaction(ctx: TransactionContext) {
    with_sentry_mut(|sentry| sentry.start_transaction(ctx));
}

pub fn set_user(user: User) {
    with_sentry_mut(|sentry| sentry.set_user(user));
}

pub fn set_request(request: Request) {
    with_sentry_mut(|sentry| sentry.set_request(request));
}

pub fn with_sentry<F, T>(f: F) -> Option<T>
where
    F: FnOnce(&Sentry) -> T,
{
    SENTRY.with(|sentry| sentry.borrow().as_ref().map(f))
}

pub(crate) fn with_sentry_mut<F, T>(f: F) -> Option<T>
where
    F: FnOnce(&mut Sentry) -> T,
{
    SENTRY.with(|sentry| sentry.borrow_mut().as_mut().map(f))
}
