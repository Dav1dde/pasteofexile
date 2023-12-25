use core::fmt;
use std::{borrow::Cow, collections::BTreeMap, io::Write};

use super::Timestamp;

#[derive(Debug, Clone)]
pub struct Metric {
    pub name: &'static str,
    pub unit: MetricUnit,
    pub tags: BTreeMap<&'static str, Cow<'static, str>>,
    pub value: MetricValue,
    pub timestamp: Option<Timestamp>,
}

impl Metric {
    pub(crate) fn to_statsd(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(50);
        let _ = write!(&mut result, "{}", self.name);
        if !matches!(self.unit, MetricUnit::None) {
            let _ = write!(&mut result, "@{}", self.unit.as_str());
        }
        let _ = write!(&mut result, ":{}|{}|", self.value, self.value.as_type());
        for (name, value) in self.tags.iter() {
            let _ = write!(&mut result, "#{name}:{value}|");
        }
        if let Some(timestamp) = self.timestamp {
            let _ = write!(&mut result, "T{}", timestamp.as_secs());
        }

        result
    }
}

#[derive(Debug, Clone, Copy)]
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
    Set(u32),
    Gauge(f64),
}

impl MetricValue {
    pub fn as_type(&self) -> &'static str {
        match self {
            MetricValue::Counter(_) => "c",
            MetricValue::Distribution(_) => "d",
            MetricValue::Set(_) => "s",
            MetricValue::Gauge(_) => "g",
        }
    }
}

impl fmt::Display for MetricValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetricValue::Counter(value) => write!(f, "{}", value),
            MetricValue::Distribution(value) => write!(f, "{}", value),
            MetricValue::Set(value) => write!(f, "{}", value),
            MetricValue::Gauge(value) => write!(f, "{}", value),
        }
    }
}
