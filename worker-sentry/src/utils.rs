use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use super::protocol::Timestamp;

pub fn hex_lower(data: &[u8]) -> String {
    use std::fmt::Write;
    data.iter().fold(String::new(), |mut output, x| {
        let _ = write!(output, "{x:02x}");
        output
    })
}

pub fn serialize_id<S: serde::Serializer>(
    uuid: &uuid::Uuid,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_some(&uuid.as_simple().to_string())
}

pub mod ts_rfc3339 {
    use serde::ser;

    use super::*;

    pub fn serialize<S>(st: &Timestamp, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let d = time::Duration::milliseconds(st.as_msecs() as i64);
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
