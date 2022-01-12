use std::rc::Rc;

use crate::Route;

struct ContextInner {
    route: Option<Route>,
    inner: Inner,
}

#[derive(Clone)]
pub struct Context {
    inner: Rc<ContextInner>,
}

impl Context {
    pub fn empty() -> Self {
        Self {
            inner: Rc::new(ContextInner {
                route: None,
                inner: Inner::None,
            }),
        }
    }

    pub fn index() -> Self {
        Self {
            inner: Rc::new(ContextInner {
                route: Some(Route::Index),
                inner: Inner::None,
            }),
        }
    }

    pub fn not_found() -> Self {
        Self {
            inner: Rc::new(ContextInner {
                route: Some(Route::NotFound),
                inner: Inner::None,
            }),
        }
    }

    pub fn paste(name: String, content: String) -> Self {
        Self {
            inner: Rc::new(ContextInner {
                route: Some(Route::Paste(name)),
                inner: Inner::Paste(Paste { content }),
            }),
        }
    }

    pub fn route(&self) -> Option<&Route> {
        self.inner.route.as_ref()
    }

    // TODO: I dont like this get_* naming
    pub fn get_paste(&self) -> Option<&Paste> {
        match self.inner.inner {
            Inner::Paste(ref paste) => Some(paste),
            _ => None,
        }
    }
}

pub struct Paste {
    content: String,
}

impl Paste {
    pub fn content(&self) -> &str {
        &self.content
    }
}

enum Inner {
    None,
    Paste(Paste),
}
