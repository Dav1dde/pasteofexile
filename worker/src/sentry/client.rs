use super::protocol;
use crate::{Error, Result};
use git_version::git_version;
use worker::{Fetch, Headers, Method, RequestInit};

#[derive(Clone)]
pub struct Sentry {
    ctx: worker::Context,
    project: String,
    token: String,
    breadcrumbs: Vec<protocol::Breadcrumb>,
    trace_context: Vec<protocol::TraceContext>,
    pub(crate) trace_id: protocol::TraceId,
    user: Option<protocol::User>,
    request: Option<protocol::Request>,
    transaction: Option<(protocol::Transaction<'static>, protocol::TraceContext)>,
}

impl Sentry {
    pub fn new(ctx: worker::Context, options: super::Options) -> Self {
        Self {
            ctx,
            project: options.project,
            token: options.token,
            breadcrumbs: Vec::new(),
            trace_context: Vec::new(),
            trace_id: protocol::TraceId::default(),
            user: None,
            request: None,
            transaction: None,
        }
    }

    pub fn set_user(&mut self, user: impl Into<Option<protocol::User>>) {
        self.user = user.into();
    }

    pub fn set_request(&mut self, request: impl Into<Option<protocol::Request>>) {
        self.request = request.into();
    }

    pub fn capture_err(&self, err: &Error) {
        self.do_capture_err(err, err.level());
    }

    pub fn capture_err_level(&self, err: &Error, level: protocol::Level) {
        self.do_capture_err(err, level);
    }

    pub(crate) fn push_trace_context(&mut self, trace_context: protocol::TraceContext) {
        self.trace_context.push(trace_context);
    }

    pub(crate) fn pop_trace_context(&mut self) {
        self.trace_context.pop();
    }

    pub(crate) fn add_span(&mut self, span: protocol::Span) {
        // TODO: start a new transaction here if there is no transaction open?
        if let Some(transaction) = self.transaction.as_mut() {
            transaction.0.spans.push(span);
        }
    }

    pub(crate) fn add_breadcrumb(&mut self, breadcrumb: protocol::Breadcrumb) {
        self.breadcrumbs.push(breadcrumb);
    }

    pub(crate) fn start_transaction(&mut self, ctx: super::TransactionContext) {
        let transaction = protocol::Transaction {
            name: Some(ctx.name),
            ..Default::default()
        };

        let trace_context = protocol::TraceContext {
            trace_id: self.trace_id,
            op: Some(ctx.op),
            ..Default::default()
        };

        self.transaction = Some((transaction, trace_context));
    }

    pub(crate) fn update_transaction(&mut self, status: protocol::SpanStatus) {
        if let Some((_, trace_context)) = self.transaction.as_mut() {
            trace_context.status = Some(status);
        }
    }

    pub(crate) fn finish_transaction(&mut self) {
        let (mut transaction, trace_context) = match self.transaction.take() {
            Some(transaction) => transaction,
            None => return,
        };

        transaction.timestamp = Some(protocol::Timestamp::now());
        transaction.release = Some(git_version!().into());
        transaction.request = self.request.clone();
        transaction.user = self.user.clone();
        transaction
            .contexts
            .insert("trace".into(), protocol::Context::Trace(trace_context));

        let _ = self.send_envelope(transaction.into());
    }

    pub(crate) fn capture_event(&self, mut event: protocol::Event<'static>) {
        let server_name = self
            .request
            .as_ref()
            .and_then(|request| request.url.as_ref())
            .and_then(|url| url.host_str())
            .map(|s| s.to_owned().into());

        event.transaction = self
            .transaction
            .as_ref()
            .and_then(|t| t.0.name.to_owned())
            .map(Into::into);
        event.breadcrumbs = self.breadcrumbs.clone();
        event.release = Some(git_version!().into());
        event.server_name = server_name;
        event.request = self.request.clone();
        event.user = self.user.clone();

        let tc = self
            .trace_context
            .last()
            .or_else(|| self.transaction.as_ref().map(|t| &t.1));
        if let Some(tc) = tc {
            event
                .contexts
                .insert("trace".into(), protocol::Context::Trace(tc.clone()));
        }

        if let Err(err) = self.send_envelope(event.into()) {
            worker::console_log!("failed to caputre error with sentry: {:?}", err);
        }
    }

    fn do_capture_err(&self, err: &Error, level: protocol::Level) {
        let event = protocol::Event {
            message: Some(err.to_string()),
            level,
            ..Default::default()
        };

        self.capture_event(event);
    }

    fn send_envelope(&self, envelope: protocol::Envelope) -> Result<()> {
        let mut headers = Headers::new();
        headers.set("Content-Type", "application/x-sentry-envelope")?;
        headers.set("User-Agent", "pobb.bin/1.0")?;
        headers.set(
            "X-Sentry-Auth",
            &format!(
                "Sentry sentry_version=7, sentry_client=pobb.in/1.0, sentry_key={}",
                self.token,
            ),
        )?;

        let mut body = Vec::new();
        envelope.to_writer(&mut body)?;

        let request = worker::Request::new_with_init(
            &format!("https://sentry.io/api/{}/envelope/", self.project),
            &RequestInit {
                method: Method::Post,
                headers,
                body: Some(unsafe { js_sys::Uint8Array::view(&body) }.into()),
                ..Default::default()
            },
        )?;

        self.ctx.wait_until(async move {
            let r = Fetch::Request(request).send().await;
            match r {
                Err(err) => worker::console_log!("failed to send envelope: {:?}", err),
                Ok(r) => {
                    if r.status_code() >= 300 {
                        worker::console_log!("failed to send envelop: {:?}", r.status_code());
                    }
                }
            }
        });

        Ok(())
    }
}
