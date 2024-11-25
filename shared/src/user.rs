use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::UrlSafe;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct User(String);

impl User {
    /// Creates a new [`Self`] from a user name.
    ///
    /// The user name is normalized.
    pub fn new(user: &str) -> Self {
        // Normalize to lowercase, usernames generally are accepted case insensitive.
        Self(user.to_lowercase())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Returns the URL to the user API.
    pub fn to_api_url(&self) -> UrlSafe<'static> {
        UrlSafe::SLASH
            .join("api")
            .join("internal")
            .join("user")
            .join(&*self.0)
    }

    /// Returns the URL to the frontend user page.
    pub fn to_url(&self) -> UrlSafe<'static> {
        UrlSafe::SLASH.join("u").join(&*self.0)
    }
}

impl std::ops::Deref for User {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl AsRef<str> for User {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<User> for String {
    fn from(user: User) -> Self {
        user.0
    }
}

impl From<&User> for User {
    fn from(user: &User) -> Self {
        user.clone()
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InvalidUser {
    #[error("Username too long")]
    TooLong,
    #[error("Invalid Username")]
    Invalid,
}

impl FromStr for User {
    type Err = InvalidUser;

    fn from_str(username: &str) -> Result<Self, Self::Err> {
        let mut count = 0usize;
        for c in username.chars() {
            if matches!(c, '/' | ':') {
                return Err(Self::Err::Invalid);
            }
            count += 1;

            if count > 30 {
                return Err(Self::Err::TooLong);
            }
        }

        Ok(Self::new(username))
    }
}

impl TryFrom<String> for User {
    type Error = InvalidUser;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}
