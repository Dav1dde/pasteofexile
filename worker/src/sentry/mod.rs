use std::borrow::Cow;
use std::{cell::RefCell, rc::Rc};

mod client;
pub(crate) mod converter;
mod layer;
mod protocol;
mod utils;

pub use self::client::Sentry;
pub use self::layer::Layer;
pub use self::protocol::{
    Breadcrumb, Level, Map, Request, SpanStatus as Status, TraceId, User, Value,
};
use crate::consts;
use crate::request_context::{Env, FromEnv};

type SentryCell = Rc<RefCell<Sentry>>;
thread_local!(static SENTRY: RefCell<Vec<SentryCell>> = RefCell::new(Vec::new()));

pub struct Options {
    pub project: String,
    pub token: String,
}

impl FromEnv for Options {
    fn from_env(env: &Env) -> Option<Self> {
        let project = env.var(consts::ENV_SENTRY_PROJECT)?;
        let token = env.var(consts::ENV_SENTRY_TOKEN)?;
        Some(Self { project, token })
    }
}

pub fn new(ctx: &worker::Context, options: impl Into<Option<Options>>) -> SentryToken {
    if let Some(options) = options.into() {
        let sentry = Rc::new(RefCell::new(Sentry::new(ctx.clone(), options)));
        SentryToken(Some(sentry))
    } else {
        SentryToken(None)
    }
}

pub struct SentryToken(Option<SentryCell>);

impl SentryToken {
    pub fn set_trace_id(&self, trace_id: TraceId) -> &Self {
        if let Some(ref cell) = self.0 {
            cell.borrow_mut().set_trace_id(trace_id);
        }
        self
    }

    pub fn set_user(&self, user: User) -> &Self {
        if let Some(ref cell) = self.0 {
            cell.borrow_mut().set_user(user);
        }
        self
    }

    pub fn set_request(&self, request: Request) -> &Self {
        if let Some(ref cell) = self.0 {
            cell.borrow_mut().set_request(request);
        }
        self
    }

    pub fn start_transaction(&self, tctx: TransactionContext) -> &Self {
        if let Some(ref cell) = self.0 {
            cell.borrow_mut().start_transaction(tctx);
        }
        self
    }

    pub fn update_transaction(&self, status: Status) -> &Self {
        if let Some(ref cell) = self.0 {
            cell.borrow_mut().update_transaction(status);
        }
        self
    }
}

impl Drop for SentryToken {
    fn drop(&mut self) {
        if let Some(sentry) = self.0.take() {
            sentry.borrow_mut().finish_transaction();
        }
    }
}

pin_project_lite::pin_project! {
    pub struct SentryFuture<T> {
        #[pin]
        inner: T,
        sentry: Option<SentryCell>,
    }
}

impl<T: std::future::Future> std::future::Future for SentryFuture<T> {
    type Output = T::Output;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        let _entered = this.sentry.clone().map(enter);
        this.inner.poll(cx)
    }
}

fn enter(sentry: SentryCell) -> impl Drop {
    struct Entered;
    impl Drop for Entered {
        fn drop(&mut self) {
            SENTRY.with(|cell| cell.borrow_mut().pop());
        }
    }

    SENTRY.with(|cell| cell.borrow_mut().push(sentry));

    Entered
}

pub trait WithSentry: Sized {
    fn with_sentry(self, sentry: &SentryToken) -> SentryFuture<Self> {
        SentryFuture {
            inner: self,
            sentry: sentry.0.clone(),
        }
    }

    fn with_current_sentry(self) -> SentryFuture<Self> {
        let sentry = SENTRY.with(|cell| cell.borrow().last().cloned());
        SentryFuture {
            inner: self,
            sentry,
        }
    }
}

impl<T: Sized> WithSentry for T {}

pub struct TransactionContext {
    pub name: String,
    pub op: String,
}

pub fn capture_err(err: &crate::Error) {
    with_sentry(|sentry| sentry.capture_err(err));
}

pub fn add_breadcrumb(breadcrumb: Breadcrumb) {
    with_sentry_mut(move |sentry| {
        sentry.add_breadcrumb(breadcrumb);
    });
}

pub fn add_attachment_plain(data: Rc<[u8]>, filename: impl Into<Cow<'static, str>>) {
    add_attachment(data, Some("text/plain".into()), filename)
}

pub fn add_attachment(
    data: Rc<[u8]>,
    content_type: Option<Cow<'static, str>>,
    filename: impl Into<Cow<'static, str>>,
) {
    with_sentry_mut(move |sentry| {
        sentry.add_attachment(protocol::Attachment {
            buffer: data,
            filename: filename.into(),
            content_type,
            ty: Some(protocol::AttachmentType::Attachment),
        })
    });
}

pub fn update_username(name: impl Into<String>) {
    with_sentry_mut(|sentry| {
        if let Some(user) = sentry.user_mut() {
            user.username = Some(name.into());
        }
    });
}

pub(crate) fn with_sentry<F, T>(f: F) -> Option<T>
where
    F: FnOnce(&Sentry) -> T,
{
    SENTRY.with(|sentry| sentry.borrow().last().map(|s| f(&s.borrow())))
}

pub(crate) fn with_sentry_mut<F, T>(f: F) -> Option<T>
where
    F: FnOnce(&mut Sentry) -> T,
{
    SENTRY.with(|sentry| sentry.borrow().last().map(|s| f(&mut s.borrow_mut())))
}
