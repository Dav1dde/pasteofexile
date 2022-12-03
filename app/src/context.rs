use crate::Route;
use shared::model::{Nodes, PasteMetadata, PasteSummary, UserPasteId};

pub struct Context {
    route: Option<Route>,
    host: String,
    inner: Inner,
}

impl Context {
    pub fn empty() -> Self {
        Self {
            route: None,
            host: "".to_owned(),
            inner: Inner::None,
        }
    }

    pub fn index(host: String) -> Self {
        Self {
            route: Some(Route::Index),
            host,
            inner: Inner::None,
        }
    }

    pub fn not_found(host: String) -> Self {
        Self {
            route: Some(Route::NotFound),
            host,
            inner: Inner::None,
        }
    }

    pub fn paste(host: String, name: String, paste: shared::model::Paste) -> Self {
        Self {
            route: Some(Route::Paste(name)),
            host,
            inner: paste.into(),
        }
    }

    pub fn user(host: String, name: shared::User, pastes: Vec<PasteSummary>) -> Self {
        Self {
            route: Some(Route::User(name)),
            host,
            inner: Inner::User(pastes),
        }
    }

    pub fn user_paste(host: String, up: UserPasteId, paste: shared::model::Paste) -> Self {
        Self {
            route: Some(Route::UserPaste(up.user, up.id)),
            host,
            inner: paste.into(),
        }
    }

    pub fn user_paste_edit(host: String, up: UserPasteId, paste: shared::model::Paste) -> Self {
        Self {
            route: Some(Route::UserEditPaste(up.user, up.id)),
            host,
            inner: paste.into(),
        }
    }

    pub fn route(&self) -> Option<&Route> {
        self.route.as_ref()
    }

    pub fn host(&self) -> &str {
        &self.host
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
