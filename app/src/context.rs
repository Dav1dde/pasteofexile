use crate::Route;
use lazycell::LazyCell;
use pob::SerdePathOfBuilding;
use shared::model::{PasteMetadata, PasteSummary, UserPasteId};
use std::rc::Rc;

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

    pub fn paste(host: String, name: String, paste: shared::model::Paste) -> Self {
        Self {
            inner: Rc::new(ContextInner {
                route: Some(Route::Paste(name)),
                host,
                inner: Inner::Paste(Paste {
                    metadata: paste.metadata,
                    content: paste.content.into(),
                    pob: LazyCell::new(),
                }),
            }),
        }
    }

    pub fn user(host: String, name: shared::User, pastes: Vec<PasteSummary>) -> Self {
        Self {
            inner: Rc::new(ContextInner {
                route: Some(Route::User(name)),
                host,
                inner: Inner::User(pastes),
            }),
        }
    }

    pub fn user_paste(host: String, up: UserPasteId, paste: shared::model::Paste) -> Self {
        Self {
            inner: Rc::new(ContextInner {
                route: Some(Route::UserPaste(up.user, up.id)),
                host,
                inner: Inner::Paste(Paste {
                    metadata: paste.metadata,
                    content: paste.content.into(),
                    pob: LazyCell::new(),
                }),
            }),
        }
    }

    pub fn user_paste_edit(host: String, up: UserPasteId, paste: shared::model::Paste) -> Self {
        Self {
            inner: Rc::new(ContextInner {
                route: Some(Route::UserEditPaste(up.user, up.id)),
                host,
                inner: Inner::Paste(Paste {
                    metadata: paste.metadata,
                    content: paste.content.into(),
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
    metadata: Option<PasteMetadata>,
    content: Rc<str>,
    pob: LazyCell<Rc<SerdePathOfBuilding>>,
}

impl Paste {
    pub fn content(&self) -> &Rc<str> {
        &self.content
    }

    pub fn metadata(&self) -> Option<&PasteMetadata> {
        self.metadata.as_ref()
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
