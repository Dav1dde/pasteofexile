use shared::model::{Nodes, PasteMetadata, PasteSummary, UserPasteId};

use crate::Route;

pub struct Context {
    route: Result<Route, crate::Error>,
    inner: Inner,
}

impl Context {
    pub fn error(err: crate::Error) -> Self {
        Self {
            route: Err(err),
            inner: Inner::None,
        }
    }

    pub fn index() -> Self {
        Self {
            route: Ok(Route::Index),
            inner: Inner::None,
        }
    }

    pub fn not_found() -> Self {
        Self {
            route: Ok(Route::NotFound),
            inner: Inner::None,
        }
    }

    pub fn paste(name: String, paste: shared::model::Paste) -> Self {
        Self {
            route: Ok(Route::Paste(name)),
            inner: paste.into(),
        }
    }

    pub fn user(name: shared::User, pastes: Vec<PasteSummary>) -> Self {
        Self {
            route: Ok(Route::User(name)),
            inner: Inner::User(pastes),
        }
    }

    pub fn user_paste(up: UserPasteId, paste: shared::model::Paste) -> Self {
        Self {
            route: Ok(Route::UserPaste(up.user, up.id)),
            inner: paste.into(),
        }
    }

    pub fn user_paste_edit(up: UserPasteId, paste: shared::model::Paste) -> Self {
        Self {
            route: Ok(Route::UserEditPaste(up.user, up.id)),
            inner: paste.into(),
        }
    }

    pub fn route(&self) -> Result<&Route, &crate::Error> {
        self.route.as_ref()
    }

    // TODO: I dont like this get_* naming
    pub fn get_paste(&self) -> Option<&Paste> {
        match self.inner {
            Inner::Paste(ref paste) => Some(paste),
            _ => None,
        }
    }

    pub fn into_paste(self) -> Option<Paste> {
        match self.inner {
            Inner::Paste(paste) => Some(paste),
            _ => None,
        }
    }

    pub fn get_user(&self) -> Option<&Vec<PasteSummary>> {
        match self.inner {
            Inner::User(ref pastes) => Some(pastes),
            _ => None,
        }
    }
}

pub struct Paste {
    pub metadata: Option<PasteMetadata>,
    pub last_modified: u64,
    pub content: String,
    pub nodes: Vec<Nodes>,
}

enum Inner {
    None,
    Paste(Paste),
    User(Vec<PasteSummary>),
}

impl From<shared::model::Paste> for Inner {
    fn from(p: shared::model::Paste) -> Self {
        Self::Paste(Paste {
            metadata: p.metadata,
            last_modified: p.last_modified,
            content: p.content,
            nodes: p.nodes,
        })
    }
}
