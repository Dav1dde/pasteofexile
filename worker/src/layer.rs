use tracing::Subscriber;
use tracing_subscriber::layer;

#[cfg(feature = "debug")]
thread_local!(static LAST_LOG_MSG: std::cell::Cell<u64> = std::cell::Cell::new(0));

pub struct Layer {}

impl<S: Subscriber> layer::Layer<S> for Layer {
    #[cfg(feature = "debug")]
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: layer::Context<'_, S>) {
        let (message, _) = crate::sentry::converter::extract_event_data(event);
        let message = message.unwrap_or_default();

        let now = worker::Date::now().as_millis();
        let mut last = LAST_LOG_MSG.with(|last| last.replace(now));
        if last == 0 {
            last = now;
        }

        let target = event
            .metadata()
            .file()
            .unwrap_or_else(|| event.metadata().target());
        let line = event.metadata().line().unwrap_or(0);
        let level = event.metadata().level();

        worker::console_log!(
            "[+ {:>5}] <{}> {:>5}: {}",
            now - last,
            format_args!("{}:{}", target, line),
            level,
            message
        );
    }
}
