use super::protocol;
use tracing::{span, Subscriber};
use tracing_subscriber::{layer, registry::LookupSpan};

pub struct Layer {}

#[allow(clippy::significant_drop_in_scrutinee)]
impl<S> layer::Layer<S> for Layer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: layer::Context<'_, S>) {
        let span = match ctx.span(id) {
            Some(span) => span,
            None => return,
        };

        let op = span.name();
        let target = span.metadata().target();

        let (description, data) = super::converter::extract_span_data(attrs);
        let description = description.unwrap_or_else(|| {
            if target.is_empty() {
                op.to_string()
            } else {
                format!("{}::{}", target, op)
            }
        });

        let parent_span_id = span.parent().and_then(|parent| {
            let mut extensions = parent.extensions_mut();
            let span = extensions.get_mut::<protocol::Span>()?;
            Some(span.span_id)
        });

        let sentry_span = protocol::Span {
            trace_id: super::with_sentry(|sentry| sentry.trace_id).unwrap_or_default(),
            parent_span_id,
            op: Some(span.name().to_owned()),
            description: Some(description),
            data,
            ..Default::default()
        };

        let trace_context = protocol::TraceContext {
            span_id: sentry_span.span_id,
            trace_id: sentry_span.trace_id,
            parent_span_id: sentry_span.parent_span_id,
            op: sentry_span.op.clone(),
            description: sentry_span.description.clone(),
            ..Default::default()
        };

        super::with_sentry_mut(|sentry| sentry.push_trace_context(trace_context));

        let mut extensions = span.extensions_mut();
        extensions.insert(sentry_span);
    }

    fn on_close(&self, id: span::Id, ctx: layer::Context<'_, S>) {
        let span = match ctx.span(&id) {
            Some(span) => span,
            None => return,
        };

        let mut extensions = span.extensions_mut();
        let mut sentry_span = match extensions.remove::<protocol::Span>() {
            Some(span) => span,
            None => return,
        };

        sentry_span.timestamp = Some(protocol::Timestamp::now());

        super::with_sentry_mut(|sentry| {
            sentry.add_span(sentry_span);
            sentry.pop_trace_context();
        });
    }

    fn on_record(&self, span: &span::Id, values: &span::Record<'_>, ctx: layer::Context<'_, S>) {
        let span = match ctx.span(span) {
            Some(s) => s,
            _ => return,
        };

        let mut extensions = span.extensions_mut();
        let span = match extensions.get_mut::<protocol::Span>() {
            Some(span) => span,
            None => return,
        };

        let mut data = super::converter::FieldVisitor::default();
        values.record(&mut data);

        for (key, value) in data.json_values {
            span.data.insert(key, value);
        }
    }

    fn on_event(&self, event: &tracing::Event<'_>, ctx: layer::Context<'_, S>) {
        use tracing::Level;

        match event.metadata().level() {
            &Level::ERROR => {
                let event = super::converter::exception_from_event(event, ctx);
                super::with_sentry(|sentry| sentry.capture_event(event));
            }
            &Level::WARN | &Level::INFO => {
                let breadcrumb = super::converter::breadcrumb_from_event(event);
                super::with_sentry_mut(|sentry| sentry.add_breadcrumb(breadcrumb));
            }
            &Level::DEBUG | &Level::TRACE => (),
        }
    }
}
