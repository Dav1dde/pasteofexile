use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PasteSummary {
    pub id: String,
    pub user: Option<String>,
    pub title: String,
    pub ascendancy: String,
    pub version: String,
    pub main_skill_name: String,
    pub last_modified: u64,
}

impl PasteSummary {
    pub(crate) fn to_url(&self) -> String {
        if let Some(ref user) = self.user {
            format!("/u/{user}/{}", self.id)
        } else {
            format!("/{}", self.id)
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct UserPasteId {
    pub user: String,
    pub id: String,
}

impl UserPasteId {
    pub fn to_user_url(&self) -> String {
        format!("/u/{}", self.user)
    }

    pub fn to_paste_url(&self) -> String {
        format!("/u/{}/{}", self.user, self.id)
    }

    pub fn to_paste_edit_url(&self) -> String {
        format!("/u/{}/{}/edit", self.user, self.id)
    }

    pub fn to_raw_url(&self) -> String {
        format!("/u/{}/{}/raw", self.user, self.id)
    }

    pub fn to_pob_open_url(&self) -> String {
        // TODO: implement this in pob
        format!("pob://pobbin/{}:{}", self.user, self.id)
    }
}

#[derive(Clone, Serialize)]
#[serde(untagged)]
pub enum PasteId {
    // TODO: newtype for this?
    Paste(String),
    UserPaste(UserPasteId),
}

impl PasteId {
    pub fn id(id: String) -> Self {
        Self::Paste(id)
    }

    pub fn user(user: String, id: String) -> Self {
        Self::UserPaste(UserPasteId { user, id })
    }

    pub fn to_raw_url(&self) -> String {
        match self {
            // TODO: use Display here?
            Self::Paste(id) => format!("/{id}/raw"),
            Self::UserPaste(up) => up.to_raw_url(),
        }
    }

    pub fn to_pob_open_url(&self) -> String {
        match self {
            // TODO: use Display here?
            Self::Paste(id) => format!("pob://pobbin/{id}"),
            Self::UserPaste(up) => up.to_pob_open_url(),
        }
    }

    pub fn unwrap_user(self) -> UserPasteId {
        match self {
            Self::UserPaste(id) => id,
            _ => panic!("unwrap_user but not a user paste id"),
        }
    }
}

impl From<UserPasteId> for PasteId {
    fn from(id: UserPasteId) -> Self {
        Self::UserPaste(id)
    }
}

impl fmt::Display for PasteId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Paste(id) => write!(f, "{id}"),
            Self::UserPaste(up) => write!(f, "{}:{}", up.user, up.id),
        }
    }
}

impl FromStr for PasteId {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let r = s
            .split_once(':')
            .map(|(user, id)| Self::user(user.to_owned(), id.to_owned()))
            .unwrap_or_else(|| Self::Paste(s.to_owned()));
        Ok(r)
    }
}
