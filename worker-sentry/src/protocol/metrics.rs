use core::fmt;

use serde::ser::SerializeMap;

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub enum MetricUnit {
    // There are a bunch more here .. https://getsentry.github.io/relay/relay_metrics/enum.MetricUnit.html
    MilliSecond,
    Second,
    Byte,
    None,
}

impl MetricUnit {
    pub fn as_str(&self) -> &'static str {
        match self {
            MetricUnit::MilliSecond => "millisecond",
            MetricUnit::Second => "second",
            MetricUnit::Byte => "byte",
            MetricUnit::None => "none",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MetricValue {
    Counter(i64),
    Distribution(f64),
}

impl serde::Serialize for MetricValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_map(Some(2))?;
        match self {
            MetricValue::Counter(value) => {
                s.serialize_entry("type", "counter")?;
                s.serialize_entry("value", value)?;
            }
            MetricValue::Distribution(value) => {
                s.serialize_entry("type", "distribution")?;
                s.serialize_entry("value", value)?;
            }
        }
        s.end()
    }
}

impl fmt::Display for MetricValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetricValue::Counter(value) => write!(f, "{value}"),
            MetricValue::Distribution(value) => write!(f, "{value}"),
        }
    }
}
