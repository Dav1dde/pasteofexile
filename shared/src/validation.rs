// TODO this should probably be Validation<T> e.g. Validation<InvalidId>
pub enum Validation {
    Valid,
    Invalid(&'static str),
}

impl Validation {
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }

    pub fn ok(&self) -> Result<(), &'static str> {
        match self {
            Self::Valid => Ok(()),
            Self::Invalid(msg) => Err(msg),
        }
    }
}

/// Validates an Id for internal usage, this constraint needs to be
/// less or equally restrictive then `user::is_valid_custom_id`.
pub use user::is_valid_custom_id as is_valid_id; // currently all Id's share the same validation

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_id() {
        // Make sure the minimum length constraint is upheld
        // some functions depend on that.
        assert!(!is_valid_id("").is_valid());
        assert!(!is_valid_id("a").is_valid());
        assert!(!is_valid_id("aa").is_valid());
        assert!(!is_valid_id("aaa").is_valid());
        assert!(!is_valid_id("aaaa").is_valid());
        assert!(is_valid_id("aaaaa").is_valid());

        // Make sure at least one special character is not allowed
        assert!(!is_valid_id("aaaaaä").is_valid());

        assert!(!is_valid_id(
            "abcdefghijklmnopqrstuvwxyz123456789012345678901234567890\
            abcdefghijklmnopqrstuvwxyz123456789012345678901234567890"
        )
        .is_valid());
        assert!(is_valid_id("abcde").is_valid());
        assert!(is_valid_id("AZ09az-_").is_valid());
        assert!(is_valid_id("-AZ09az-_").is_valid());
    }
}

/// User facing validation
pub mod user {
    use super::Validation::{self, *};

    #[must_use]
    pub fn is_valid_custom_title(title: &str) -> Validation {
        // TODO: maybe validate chars not length
        match title.len() {
            0..=4 => Invalid("Title too short"),
            5..=90 => Valid,
            _ => Invalid("Title too long"),
        }
    }

    #[must_use]
    pub fn is_valid_custom_id(id: &str) -> Validation {
        match id.len() {
            0..=4 => return Invalid("Id too short"),
            5..=90 => (),
            _ => return Invalid("Id too long"),
        };

        let valid = id
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'-'));

        match valid {
            true => Valid,
            false => Invalid("Invalid Id, allowed characters: [0-9a-zA-Z_-]"),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_title_length() {
            for i in 0..200 {
                let s = std::iter::repeat('a').take(i).collect::<String>();
                let v = is_valid_custom_title(&s);
                assert!(if i >= 5 && i <= 90 {
                    v.is_valid()
                } else {
                    !v.is_valid()
                });
            }
        }

        #[test]
        fn test_title_chars() {
            assert!(is_valid_custom_title("aks jda;klsdäö").is_valid());
        }

        #[test]
        fn test_id_length() {
            for i in 0..200 {
                let s = std::iter::repeat('a').take(i).collect::<String>();
                let v = is_valid_custom_id(&s);
                assert!(if i >= 5 && i <= 90 {
                    v.is_valid()
                } else {
                    !v.is_valid()
                });
            }
        }

        #[test]
        fn test_id_chars() {
            assert!(!is_valid_custom_id("aAzZ09aaaa bb").is_valid());
            assert!(!is_valid_custom_id("aAzZ09aaaa;bb").is_valid());
            assert!(!is_valid_custom_id("aAzZ09aaaaäbb").is_valid());
            assert!(!is_valid_custom_id("aAzZ09aaaa/bb").is_valid());
            assert!(is_valid_custom_id("aAzZ09aaaa_bb").is_valid());
            assert!(is_valid_custom_id("aAzZ09aaaa-bb").is_valid());
        }
    }
}
