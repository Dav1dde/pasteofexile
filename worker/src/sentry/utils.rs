use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use super::protocol::Timestamp;

pub fn serialize_id<S: serde::Serializer>(
    uuid: &uuid::Uuid,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_some(&uuid.as_simple().to_string())
}

pub mod ts_rfc3339 {
    use std::fmt;

    use serde::{de, ser};

    use super::*;

    pub fn serialize<S>(st: &Timestamp, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let d = time::Duration::milliseconds(st.0 as i64);
        match OffsetDateTime::UNIX_EPOCH
            .checked_add(d)
            .and_then(|dt| dt.format(&Rfc3339).ok())
        {
            Some(formatted) => serializer.serialize_str(&formatted),
            None => Err(ser::Error::custom(format!(
                "invalid `Timestamp` instance: {st:?}"
            ))),
        }
    }

    pub(super) struct Rfc3339Deserializer;

    impl<'de> de::Visitor<'de> for Rfc3339Deserializer {
        type Value = Timestamp;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "an RFC3339 timestamp")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let dt = OffsetDateTime::parse(v, &Rfc3339).map_err(|e| E::custom(format!("{e}")))?;
            let secs = u64::try_from(dt.unix_timestamp()).map_err(|e| E::custom(format!("{e}")))?;
            Ok(Timestamp::from_secs(secs))
        }
    }
}

pub mod ts_rfc3339_opt {
    use serde::ser;

    use super::*;

    pub fn serialize<S>(st: &Option<Timestamp>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match st {
            Some(st) => ts_rfc3339::serialize(st, serializer),
            None => serializer.serialize_none(),
        }
    }
}
