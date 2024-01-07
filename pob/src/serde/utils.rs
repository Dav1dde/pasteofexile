use std::{fmt, marker::PhantomData, str::FromStr};

use serde::{de, Deserialize, Deserializer};

macro_rules! or_nil_impl {
    ($name:ident, $t:ty) => {
        pub fn $name<'de, D>(deserializer: D) -> Result<Option<$t>, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct NumVisitor;

            impl<'de> de::Visitor<'de> for NumVisitor {
                type Value = Option<$t>;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a number or nil")
                }

                fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                    if value == "nil" {
                        Ok(None)
                    } else {
                        value.parse().map_err(de::Error::custom).map(Some)
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

pub fn lenient<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
    T: Default,
{
    Ok(T::deserialize(deserializer).unwrap_or_else(|_| Default::default()))
}

pub fn comma_separated<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    use serde::de::Error;

    struct Visitor<T>(PhantomData<T>);

    impl<'de, T> de::Visitor<'de> for Visitor<T>
    where
        T: FromStr,
        T::Err: fmt::Display,
    {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a comma separated list")
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
            value
                .split(',')
                .map(|item| item.trim())
                .map(FromStr::from_str)
                .collect::<Result<_, _>>()
                .map_err(Error::custom)
        }
    }

    deserializer.deserialize_str(Visitor(Default::default()))
}

pub fn lua_table<'de, D, K, V>(deserializer: D) -> Result<Vec<(K, V)>, D::Error>
where
    D: Deserializer<'de>,
    K: FromStr,
    K::Err: std::error::Error,
    V: FromStr,
    V::Err: std::error::Error,
{
    use serde::de::{Error, Unexpected};

    struct Visitor<K, V> {
        _phantom: PhantomData<(K, V)>,
    }

    impl<'de, K, V> de::Visitor<'de> for Visitor<K, V>
    where
        K: FromStr,
        K::Err: std::error::Error,
        V: FromStr,
        V::Err: std::error::Error,
    {
        type Value = Vec<(K, V)>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a lua table")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if v.is_empty() {
                return Ok(Vec::new());
            }

            let mut result = Vec::new();
            for part in v.trim_end_matches('}').split("},") {
                let (k, v) = part
                    .strip_prefix('{')
                    .and_then(|part| part.split_once(','))
                    .ok_or_else(|| Error::invalid_value(Unexpected::Str(v), &self))?;

                let k = k.parse().map_err(|e| Error::custom(e))?;
                let v = v.parse().map_err(|e| Error::custom(e))?;

                result.push((k, v))
            }

            Ok(result)
        }
    }

    deserializer.deserialize_str(Visitor {
        _phantom: PhantomData,
    })
}
