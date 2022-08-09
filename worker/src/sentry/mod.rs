use std::borrow::Cow;
use std::{cell::RefCell, rc::Rc};

mod client;
pub(crate) mod converter;
mod layer;
mod protocol;
mod utils;

pub use self::client::Sentry;
pub use self::layer::Layer;
pub use self::protocol::{Breadcrumb, Level, Map, Request, SpanStatus as Status, User, Value};

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

pub fn update_transaction(status: Status) {
    with_sentry_mut(|sentry| sentry.update_transaction(status));
}

pub fn add_breadcrumb(breadcrumb: Breadcrumb) {
    with_sentry_mut(move |sentry| {
        sentry.add_breadcrumb(breadcrumb);
    });
}

pub fn add_attachment_plain(data: Rc<[u8]>, filename: impl Into<Cow<'static, str>>) {
    with_sentry_mut(move |sentry| {
        sentry.add_attachment(protocol::Attachment {
            buffer: data,
            filename: filename.into(),
            content_type: Some("text/plain".into()),
            ty: Some(protocol::AttachmentType::Attachment),
        })
    });
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
