use std::collections::BTreeMap;
use std::error::Error;

use super::protocol::{Breadcrumb, Event, Exception, Level, Value};
use tracing::field::{Field, Visit};
use tracing::{span, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;

fn convert_tracing_level(level: &tracing::Level) -> Level {
    match level {
        &tracing::Level::TRACE | &tracing::Level::DEBUG => Level::Debug,
        &tracing::Level::INFO => Level::Info,
        &tracing::Level::WARN => Level::Warning,
        &tracing::Level::ERROR => Level::Error,
    }
}

/// Extracts the message and metadata from an event
pub(crate) fn extract_event_data(event: &tracing::Event) -> (Option<String>, FieldVisitor) {
    // Find message of the event, if any
    let mut visitor = FieldVisitor::default();
    event.record(&mut visitor);
    let message = visitor
        .json_values
        .remove("message")
        // When #[instrument(err)] is used the event does not have a message attached to it.
        // the error message is attached to the field "error".
        .or_else(|| visitor.json_values.remove("error"))
        .and_then(|v| v.as_str().map(|s| s.to_owned()));

    (message, visitor)
}

/// Extracts the message and metadata from a span
pub(crate) fn extract_span_data(
    attrs: &span::Attributes,
) -> (Option<String>, BTreeMap<String, Value>) {
    let mut data = FieldVisitor::default();
    attrs.record(&mut data);

    // Find message of the span, if any
    let message = data
        .json_values
        .remove("message")
        .and_then(|v| v.as_str().map(|s| s.to_owned()));

    (message, data.json_values)
}

/// Records all fields of [`tracing_core::Event`] for easy access
#[derive(Default)]
pub(crate) struct FieldVisitor {
    pub json_values: BTreeMap<String, Value>,
    pub exceptions: Vec<super::protocol::Exception>,
}

impl FieldVisitor {
    fn record<T: Into<Value>>(&mut self, field: &Field, value: T) {
        self.json_values
            .insert(field.name().to_owned(), value.into());
    }
}

impl Visit for FieldVisitor {
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.record(field, value);
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.record(field, value);
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.record(field, value);
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.record(field, value);
    }

    fn record_error(&mut self, _field: &Field, value: &(dyn Error + 'static)) {
        let event = event_from_error(value);
        for exception in event.exception {
            self.exceptions.push(exception);
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.record(field, format!("{:?}", value));
    }
}

/// Creates a [`Breadcrumb`] from a given [`tracing_core::Event`]
pub fn breadcrumb_from_event(event: &tracing::Event) -> Breadcrumb {
    let (message, visitor) = extract_event_data(event);
    Breadcrumb {
        category: Some(event.metadata().target().into()),
        ty: Some("log".into()),
        level: convert_tracing_level(event.metadata().level()),
        message,
        data: visitor.json_values,
        ..Default::default()
    }
}

pub fn event_from_error<E: Error + ?Sized>(err: &E) -> Event<'static> {
    let mut exceptions = vec![exception_from_error(err)];

    let mut source = err.source();
    while let Some(err) = source {
        exceptions.push(exception_from_error(err));
        source = err.source();
    }

    exceptions.reverse();
    Event {
        exception: exceptions,
        level: Level::Error,
        ..Default::default()
    }
}

fn exception_from_error<E: Error + ?Sized>(err: &E) -> Exception {
    let dbg = format!("{:?}", err);
    let value = err.to_string();

    // A generic `anyhow::msg` will just `Debug::fmt` the `String` that you feed
    // it. Trying to parse the type name from that will result in a leading quote
    // and the first word, so quite useless.
    // To work around this, we check if the `Debug::fmt` of the complete error
    // matches its `Display::fmt`, in which case there is no type to parse and
    // we will just be using `Error`.
    let ty = if dbg == format!("{:?}", value) {
        String::from("Error")
    } else {
        parse_type_from_debug(&dbg).to_owned()
    };
    Exception {
        ty,
        value: Some(err.to_string()),
        ..Default::default()
    }
}

pub fn parse_type_from_debug(d: &str) -> &str {
    d.split(&[' ', '(', '{', '\r', '\n'][..])
        .next()
        .unwrap()
        .trim()
}

/// Creates an exception [`Event`] from a given [`tracing_core::Event`]
pub fn exception_from_event<S>(event: &tracing::Event, _ctx: Context<S>) -> Event<'static>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    let (message, visitor) = extract_event_data(event);
    Event {
        logger: Some(event.metadata().target().to_owned()),
        level: convert_tracing_level(event.metadata().level()),
        message,
        exception: visitor.exceptions,
        ..Default::default()
    }
}
