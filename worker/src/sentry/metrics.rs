use std::{borrow::Cow, collections::BTreeMap};

use super::protocol::{Metric, MetricUnit, MetricValue, Timestamp};

pub trait MetricName {
    fn name(&self) -> &'static str;
}

impl MetricName for &'static str {
    fn name(&self) -> &'static str {
        self
    }
}

pub trait MetricTagValue {
    fn to_value(self) -> Cow<'static, str>;
}

impl MetricTagValue for &'static str {
    fn to_value(self) -> Cow<'static, str> {
        self.into()
    }
}

impl MetricTagValue for String {
    fn to_value(self) -> Cow<'static, str> {
        self.into()
    }
}

macro_rules! metric_tag_value_display {
    ($ty:ty) => {
        impl MetricTagValue for $ty {
            fn to_value(self) -> Cow<'static, str> {
                self.to_string().into()
            }
        }
    };
}

metric_tag_value_display!(u8);
metric_tag_value_display!(i8);
metric_tag_value_display!(u16);
metric_tag_value_display!(i16);
metric_tag_value_display!(u32);
metric_tag_value_display!(i32);
metric_tag_value_display!(u64);
metric_tag_value_display!(i64);
metric_tag_value_display!(usize);
metric_tag_value_display!(isize);
metric_tag_value_display!(f32);
metric_tag_value_display!(f64);

pub struct Counter {
    name: &'static str,
    unit: MetricUnit,
    value: i64,
    tags: BTreeMap<&'static str, Cow<'static, str>>,
    timestamp: Option<Timestamp>,
}

impl Counter {
    pub fn unit(mut self, unit: MetricUnit) -> Self {
        self.unit = unit;
        self
    }

    pub fn inc(mut self, count: i64) -> Self {
        self.value += count;
        self
    }

    pub fn tag(mut self, name: &'static str, value: impl MetricTagValue) -> Self {
        self.tags.insert(name, value.to_value());
        self
    }

    pub fn tag_opt(mut self, name: &'static str, value: Option<impl MetricTagValue>) -> Self {
        if let Some(value) = value {
            self.tags.insert(name, value.to_value());
        }
        self
    }
}

impl Drop for Counter {
    fn drop(&mut self) {
        if self.value == 0 {
            return;
        }

        let metric = Metric {
            name: self.name,
            unit: self.unit,
            tags: std::mem::take(&mut self.tags),
            value: MetricValue::Counter(self.value),
            timestamp: self.timestamp,
        };

        super::with_sentry_mut(move |sentry| {
            sentry.add_metric(metric);
        });
    }
}

pub fn counter(metric: impl MetricName) -> Counter {
    Counter {
        name: metric.name(),
        unit: MetricUnit::None,
        tags: BTreeMap::new(),
        value: 0,
        timestamp: None,
    }
}

pub struct Distribution {
    name: &'static str,
    unit: MetricUnit,
    value: f64,
    tags: BTreeMap<&'static str, Cow<'static, str>>,
    timestamp: Option<Timestamp>,
}

impl Distribution {
    pub fn unit(mut self, unit: MetricUnit) -> Self {
        self.unit = unit;
        self
    }

    pub fn tag(mut self, name: &'static str, value: impl MetricTagValue) -> Self {
        self.tags.insert(name, value.to_value());
        self
    }

    pub fn tag_opt(mut self, name: &'static str, value: Option<impl MetricTagValue>) -> Self {
        if let Some(value) = value {
            self.tags.insert(name, value.to_value());
        }
        self
    }
}

impl Drop for Distribution {
    fn drop(&mut self) {
        let metric = Metric {
            name: self.name,
            unit: self.unit,
            tags: std::mem::take(&mut self.tags),
            value: MetricValue::Distribution(self.value),
            timestamp: self.timestamp,
        };

        super::with_sentry_mut(move |sentry| {
            sentry.add_metric(metric);
        });
    }
}

pub fn distribution(metric: impl MetricName, value: impl Into<f64>) -> Distribution {
    Distribution {
        name: metric.name(),
        unit: MetricUnit::None,
        value: value.into(),
        tags: BTreeMap::new(),
        timestamp: None,
    }
}
