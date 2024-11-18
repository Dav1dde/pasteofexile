use std::{
    borrow::Cow,
    fmt::{self, Write as _},
};

/// Custom character set which probably should adhere to some RFC or web spec,
/// but currently is just characters that need to be escaped in order for things to work.
/// Well at least for now.
static URL_CHARACTER_SET: percent_encoding::AsciiSet = percent_encoding::CONTROLS
    .add(b' ')
    .add(b'#')
    .add(b'%')
    .add(b'?')
    .add(b'@');

/// An URL safe string.
///
/// The string is correctly encoded to be used in an URL path.
pub struct UrlSafe<'a>(UrlSafeInner<'a>);

impl<'a> UrlSafe<'a> {
    pub const SLASH: UrlSafe<'static> = UrlSafe(UrlSafeInner::Static("/"));

    pub fn new(s: &'a str) -> Self {
        Self(UrlSafeInner::Encoded(
            percent_encoding::utf8_percent_encode(s, &URL_CHARACTER_SET),
        ))
    }

    /// Creates an [`UrlSafe`] from a string literal.
    ///
    /// The function requires the string to be already URL safe.
    pub fn from_static(s: &'static str) -> UrlSafe<'static> {
        UrlSafe(UrlSafeInner::Static(s))
    }

    pub fn join<'b>(self, other: impl Into<UrlSafe<'b>>) -> UrlSafe<'static> {
        let other = other.into();

        let mut s = self.0.into_string();
        let _ = match s.ends_with('/') {
            true => write!(&mut s, "{other}"),
            false => write!(&mut s, "/{other}"),
        };
        UrlSafe(UrlSafeInner::Owned(s))
    }

    pub fn push<'b>(self, other: impl Into<UrlSafe<'b>>) -> UrlSafe<'static> {
        let other = other.into();

        let mut s = self.0.into_string();
        let _ = write!(&mut s, "{other}");

        UrlSafe(UrlSafeInner::Owned(s))
    }

    pub fn into_cow(self) -> Cow<'a, str> {
        match self.0 {
            UrlSafeInner::Owned(s) => Cow::Owned(s),
            UrlSafeInner::Static(s) => Cow::Borrowed(s),
            UrlSafeInner::Encoded(s) => s.into(),
        }
    }
}

impl<'a> From<&'a str> for UrlSafe<'a> {
    fn from(value: &'a str) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for UrlSafe<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            UrlSafeInner::Owned(s) => f.write_str(s),
            UrlSafeInner::Static(s) => f.write_str(s),
            UrlSafeInner::Encoded(s) => s.fmt(f),
        }
    }
}

enum UrlSafeInner<'a> {
    Owned(String),
    Static(&'static str),
    Encoded(percent_encoding::PercentEncode<'a>),
}

impl UrlSafeInner<'_> {
    fn into_string(self) -> String {
        match self {
            Self::Owned(s) => s,
            Self::Static(s) => s.to_owned(),
            Self::Encoded(s) => s.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urlsafe_join() {
        assert_eq!(UrlSafe::SLASH.join("foo").to_string(), "/foo");
        assert_eq!(UrlSafe::new("foo").join("bar").to_string(), "foo/bar");
        assert_eq!(
            UrlSafe::new("foo").join("bar#baz").to_string(),
            "foo/bar%23baz"
        );
    }

    #[test]
    fn test_urlsafe_push() {
        assert_eq!(
            UrlSafe::new("foo").push(":").push("bar").to_string(),
            "foo:bar"
        );
    }

    #[test]
    fn test_urlsafe_static() {
        assert_eq!(
            UrlSafe::from_static("foobar://").join("bar").to_string(),
            "foobar://bar"
        );
    }
}
