use std::cell::RefCell;

thread_local! {
    static RESPONSE_CONTEXT: RefCell<Option<ResponseContext>> = RefCell::new(None);
}

macro_rules! with_ctx {
    ($ctx:ident, $block:expr) => {{
        if let Some($ctx) = $ctx.borrow_mut().as_mut() {
            $block
        }
    }};
}

pub struct ResponseContext {
    status_code: u16,
}

impl ResponseContext {
    fn new() -> Self {
        Self { status_code: 200 }
    }

    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    pub(crate) fn with<F, R>(f: F) -> (R, ResponseContext)
    where
        F: FnOnce() -> R,
    {
        RESPONSE_CONTEXT.with(|ctx| {
            assert!(ctx.borrow().is_none());
            *ctx.borrow_mut() = Some(Self::new());
            let r = f();
            let ctx = ctx.borrow_mut().take().unwrap();
            (r, ctx)
        })
    }

    pub(crate) fn set_status_code(status_code: u16) {
        RESPONSE_CONTEXT.with(|ctx| {
            with_ctx!(ctx, {
                ctx.status_code = status_code;
            })
        });
    }
}

impl Default for ResponseContext {
    fn default() -> Self {
        Self::new()
    }
}
