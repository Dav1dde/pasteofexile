use serde::de;
use serde::Deserializer;
use std::fmt;

macro_rules! or_nil_impl {
    ($name:ident, $t:ty) => {
        pub fn $name<'de, D>(deserializer: D) -> Result<$t, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct NumVisitor;

            impl<'de> de::Visitor<'de> for NumVisitor {
                type Value = $t;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("expected a number or nil")
                }

                fn visit_str<E: de::Error>(self, value: &str) -> Result<$t, E> {
                    if value == "nil" {
                        Ok(0)
                    } else {
                        value.parse().map_err(de::Error::custom)
                    }
                }
            }

            deserializer.deserialize_any(NumVisitor)
        }
    };
}

or_nil_impl!(u8_or_nil, u8);

pub(crate) const fn default_true() -> bool {
    true
}
