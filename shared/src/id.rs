use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::UrlSafe;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(String);

impl Id {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl std::ops::Deref for Id {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InvalidId {
    #[error("Id too short")]
    TooShort,
    #[error("Id too long")]
    TooLong,
    #[error("Invalid Id, allowed characters: [0-9a-zA-Z_-]")]
    Invalid,
}

impl FromStr for Id {
    type Err = InvalidId;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        id.to_owned().try_into()
    }
}

impl TryFrom<String> for Id {
    type Error = InvalidId;

    fn try_from(id: String) -> Result<Self, Self::Error> {
        match id.len() {
            0..=4 => return Err(InvalidId::TooShort),
            5..=90 => (),
            _ => return Err(InvalidId::TooLong),
        };

        let valid = id
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'-'));

        match valid {
            true => Ok(Id(id)),
            false => Err(InvalidId::Invalid),
        }
    }
}

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserPasteId {
    pub user: crate::User,
    pub id: Id,
}

impl UserPasteId {
    pub fn to_user_api_url(&self) -> UrlSafe<'static> {
        self.user.to_api_url()
    }

    pub fn to_user_url(&self) -> UrlSafe<'static> {
        self.user.to_url()
    }

    pub fn to_paste_url(&self) -> UrlSafe<'static> {
        UrlSafe::SLASH.join("u").join(&*self.user).join(&*self.id)
    }

    pub fn to_paste_api_url(&self) -> UrlSafe<'static> {
        UrlSafe::SLASH
            .join("api")
            .join("internal")
            .join("paste")
            .join(UrlSafe::new(&self.user).push(":").push(&*self.id))
    }

    pub fn to_paste_edit_url(&self) -> UrlSafe<'static> {
        self.to_paste_url().join("edit")
    }

    pub fn to_raw_url(&self) -> UrlSafe<'static> {
        self.to_paste_url().join("raw")
    }

    pub fn to_json_url(&self) -> UrlSafe<'static> {
        self.to_paste_url().join("json")
    }

    pub fn to_pob_load_url(&self) -> UrlSafe<'static> {
        // TODO: maybe get rid of this format?
        UrlSafe::SLASH
            .join("pob")
            .join(UrlSafe::new(&self.user).push(":").push(&*self.id))
    }

    pub fn to_pob_long_load_url(&self) -> UrlSafe<'static> {
        UrlSafe::SLASH
            .join("pob")
            .join("u")
            .join(&*self.user)
            .join(&*self.id)
    }

    pub fn to_pob_open_url(&self) -> UrlSafe<'static> {
        // ("pob").join("u").join(&*self.user).join(&*self.id)
        // format!("pob://pobbin/{}:{}", self.user, self.id)
        UrlSafe::from_static("pob://pobbin/")
            .join(&*self.user)
            .push(":")
            .join(&*self.id)
    }
}

impl fmt::Display for UserPasteId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.user, self.id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PasteId {
    Paste(Id),
    UserPaste(UserPasteId),
}

impl PasteId {
    pub fn id(&self) -> &str {
        match self {
            Self::Paste(id) => id,
            Self::UserPaste(up) => &up.id,
        }
    }

    pub fn user(&self) -> Option<&crate::User> {
        match self {
            Self::Paste(_) => None,
            Self::UserPaste(up) => Some(&up.user),
        }
    }

    pub fn to_url(&self) -> UrlSafe<'static> {
        match self {
            Self::Paste(id) => UrlSafe::SLASH.join(id.as_str()),
            Self::UserPaste(up) => up.to_paste_url(),
        }
    }

    pub fn to_raw_url(&self) -> UrlSafe<'static> {
        match self {
            Self::Paste(id) => UrlSafe::SLASH.join(id.as_str()).join("raw"),
            Self::UserPaste(up) => up.to_raw_url(),
        }
    }

    pub fn to_json_url(&self) -> UrlSafe<'static> {
        match self {
            Self::Paste(id) => UrlSafe::SLASH.join(id.as_str()).join("json"),
            Self::UserPaste(up) => up.to_json_url(),
        }
    }

    pub fn to_pob_load_url(&self) -> UrlSafe<'static> {
        match self {
            Self::Paste(id) => UrlSafe::SLASH.join("pob").join(id.as_str()),
            Self::UserPaste(up) => up.to_pob_load_url(),
        }
    }

    pub fn to_pob_open_url(&self) -> UrlSafe<'static> {
        match self {
            Self::Paste(id) => UrlSafe::from_static("pob://pobbin/").join(id.as_str()),
            Self::UserPaste(up) => up.to_pob_open_url(),
        }
    }

    // TODO get rid of unwraps_*
    // Should be easily possible with a trait to convert to urls for ids
    pub fn unwrap_paste(self) -> Id {
        match self {
            Self::Paste(id) => id,
            _ => panic!("unwrap_paste but not a paste"),
        }
    }

    pub fn unwrap_user(self) -> UserPasteId {
        match self {
            Self::UserPaste(id) => id,
            _ => panic!("unwrap_user but not a user paste id"),
        }
    }
}

impl From<Id> for PasteId {
    fn from(id: Id) -> Self {
        Self::Paste(id)
    }
}

impl From<UserPasteId> for PasteId {
    fn from(id: UserPasteId) -> Self {
        Self::UserPaste(id)
    }
}

impl From<PasteId> for String {
    fn from(id: PasteId) -> Self {
        match id {
            PasteId::Paste(id) => id.to_string(),
            PasteId::UserPaste(up) => format!("{}:{}", up.user, up.id),
        }
    }
}

impl From<&PasteId> for PasteId {
    fn from(id: &PasteId) -> Self {
        id.clone()
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

#[derive(Debug, thiserror::Error)]
pub enum InvalidPasteId {
    #[error(transparent)]
    InvalidId(#[from] InvalidId),
    #[error(transparent)]
    InvalidUser(#[from] crate::InvalidUser),
}

impl FromStr for PasteId {
    // TODO: better error
    type Err = InvalidPasteId;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let r = match s.split_once(':') {
            Some((user, id)) => {
                let user = user.parse()?;
                Self::UserPaste(UserPasteId {
                    user,
                    id: id.parse()?,
                })
            }
            None => Self::Paste(s.parse()?),
        };

        Ok(r)
    }
}

impl<'de> Deserialize<'de> for PasteId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

impl Serialize for PasteId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
