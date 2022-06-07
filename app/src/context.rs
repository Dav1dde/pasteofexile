use lazycell::LazyCell;
use pob::SerdePathOfBuilding;
use std::rc::Rc;

use crate::{model::PasteSummary, Route};

struct ContextInner {
    route: Option<Route>,
    host: String,
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
                host: "".to_owned(),
                inner: Inner::None,
            }),
        }
    }

    pub fn index(host: String) -> Self {
        Self {
            inner: Rc::new(ContextInner {
                route: Some(Route::Index),
                host,
                inner: Inner::None,
            }),
        }
    }

    pub fn not_found(host: String) -> Self {
        Self {
            inner: Rc::new(ContextInner {
                route: Some(Route::NotFound),
                host,
                inner: Inner::None,
            }),
        }
    }

    pub fn paste(host: String, name: String, content: String) -> Self {
        Self {
            inner: Rc::new(ContextInner {
                route: Some(Route::Paste(name)),
                host,
                inner: Inner::Paste(Paste {
                    content,
                    pob: LazyCell::new(),
                }),
            }),
        }
    }

    pub fn user(host: String, name: String, pastes: Vec<PasteSummary>) -> Self {
        Self {
            inner: Rc::new(ContextInner {
                route: Some(Route::User(name)),
                host,
                inner: Inner::User(pastes),
            }),
        }
    }

    pub fn user_paste(host: String, user: String, name: String, content: String) -> Self {
        Self {
            inner: Rc::new(ContextInner {
                route: Some(Route::UserPaste(user, name)),
                host,
                inner: Inner::Paste(Paste {
                    content,
                    pob: LazyCell::new(),
                }),
            }),
        }
    }

    pub fn route(&self) -> Option<&Route> {
        self.inner.route.as_ref()
    }

    pub fn host(&self) -> &str {
        &self.inner.host
    }

    // TODO: I dont like this get_* naming
    pub fn get_paste(&self) -> Option<&Paste> {
        match self.inner.inner {
            Inner::Paste(ref paste) => Some(paste),
            _ => None,
        }
    }

    pub fn get_user(&self) -> Option<&Vec<PasteSummary>> {
        match self.inner.inner {
            Inner::User(ref pastes) => Some(pastes),
            _ => None,
        }
    }
}

pub struct Paste {
    content: String,
    pob: LazyCell<Rc<SerdePathOfBuilding>>,
}

impl Paste {
    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn path_of_building(&self) -> anyhow::Result<Rc<SerdePathOfBuilding>> {
        self.pob
            .try_borrow_with(|| Ok(Rc::new(SerdePathOfBuilding::from_export(&self.content)?)))
            .map(|x| x.clone())
    }
}

enum Inner {
    None,
    Paste(Paste),
    User(Vec<PasteSummary>),
}
