use super::utils::{ts_rfc3339, ts_rfc3339_opt};
use serde::Serialize;
use std::borrow::Cow;
use std::fmt;
use std::io::Write;
use std::rc::Rc;

pub use serde_json::Value;
pub use std::collections::BTreeMap as Map;

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum EnvelopeItem<'a> {
    Event(Event<'a>),
    Transaction(Transaction<'a>),
    Attachment(&'a Attachment),
}

#[derive(Default, Debug)]
pub struct Envelope<'a> {
    event_id: Option<uuid::Uuid>,
    items: Vec<EnvelopeItem<'a>>,
}

impl<'a> Envelope<'a> {
    pub fn add_item(&mut self, item: EnvelopeItem<'a>) {
        if self.event_id.is_none() {
            if let EnvelopeItem::Event(ref event) = item {
                self.event_id = Some(event.event_id);
            } else if let EnvelopeItem::Transaction(ref transaction) = item {
                self.event_id = Some(transaction.event_id);
            }
        }
        self.items.push(item);
    }

    pub fn to_writer<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: Write,
    {
        let mut item_buf = Vec::new();

        // write the headers:
        let event_id = self.event_id.as_ref();
        match event_id {
            Some(uuid) => writeln!(writer, r#"{{"event_id":"{}"}}"#, uuid)?,
            _ => writeln!(writer, "{{}}")?,
        }

        // write each item:
        for item in &self.items {
            // we write them to a temporary buffer first, since we need their length
            match item {
                EnvelopeItem::Event(event) => serde_json::to_writer(&mut item_buf, event)?,
                EnvelopeItem::Transaction(transaction) => {
                    serde_json::to_writer(&mut item_buf, transaction)?
                }
                EnvelopeItem::Attachment(attachment) => {
                    attachment.to_writer(&mut writer)?;
                    writeln!(writer)?;
                    continue;
                }
            }
            let item_type = match item {
                EnvelopeItem::Event(_) => "event",
                EnvelopeItem::Transaction(_) => "transaction",
                EnvelopeItem::Attachment(_) => unreachable!(),
            };
            writeln!(
                writer,
                r#"{{"type":"{}","length":{}}}"#,
                item_type,
                item_buf.len()
            )?;
            writer.write_all(&item_buf)?;
            writeln!(writer)?;
            item_buf.clear();
        }

        Ok(())
    }
}

impl<'a> From<Event<'a>> for Envelope<'a> {
    fn from(event: Event<'a>) -> Self {
        let mut envelope = Self::default();
        envelope.add_item(EnvelopeItem::Event(event));
        envelope
    }
}

impl<'a> From<Transaction<'a>> for Envelope<'a> {
    fn from(transaction: Transaction<'a>) -> Self {
        let mut envelope = Self::default();
        envelope.add_item(EnvelopeItem::Transaction(transaction));
        envelope
    }
}

#[derive(Serialize)]
pub struct Store<'a> {
    pub logger: &'a str,
    pub platform: &'a str,
    pub level: &'a str,
    // extra: Option<HashMap<&'a str, >,
    // exception: &'a Exception,
    // request: &'a Request,
    pub user: &'a User,
    pub server_name: &'a str,
    pub release: &'a str,
    pub transaction: &'a str,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct Exceptions {
    values: Vec<Exception>,
}

#[derive(Serialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct Exception {
    /// The type of the exception.
    #[serde(rename = "type")]
    pub ty: String,
    /// The optional value of the exception.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// An optional module for this exception.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
}

#[derive(Default, Clone, Debug, Serialize, PartialEq, Eq)]
pub struct Request {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<url::Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub headers: Map<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct User {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct SpanId([u8; 8]);

impl Default for SpanId {
    fn default() -> Self {
        let val = crate::crypto::get_random_values().expect("SpanId random");

        Self(val)
    }
}

impl fmt::Display for SpanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", crate::utils::hex_lower(&self.0))
    }
}

impl fmt::Debug for SpanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SpanId({})", crate::utils::hex_lower(&self.0))
    }
}

impl serde::Serialize for SpanId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct TraceId([u8; 16]);

impl Default for TraceId {
    fn default() -> Self {
        let val = crate::crypto::get_random_values().expect("TraceId random");

        Self(val)
    }
}

impl fmt::Debug for TraceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TraceId({})", crate::utils::hex_lower(&self.0))
    }
}

impl fmt::Display for TraceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", crate::utils::hex_lower(&self.0))
    }
}

impl serde::Serialize for TraceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Timestamp(pub(crate) u64);

impl Timestamp {
    pub(crate) fn now() -> Timestamp {
        Self(js_sys::Date::new_0().get_time() as u64)
    }

    pub(crate) fn from_secs(secs: u64) -> Timestamp {
        Self(secs)
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self(js_sys::Date::new_0().get_time() as u64)
    }
}

impl serde::Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ((self.0 as f32) / 1000.0).serialize(serializer)
    }
}

#[derive(Serialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct Span {
    /// The ID of the span
    #[serde(default)]
    pub span_id: SpanId,
    /// Determines which trace the span belongs to.
    #[serde(default)]
    pub trace_id: TraceId,
    /// Determines the parent of this span, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<SpanId>,
    /// Determines whether this span is generated in the same process as its parent, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub same_process_as_parent: Option<bool>,
    /// Short code identifying the type of operation the span is measuring.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub op: Option<String>,
    /// Longer description of the span's operation, which uniquely identifies the span
    /// but is consistent across instances of the span.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The timestamp at the measuring of the span finished.
    #[serde(skip_serializing_if = "Option::is_none", with = "ts_rfc3339_opt")]
    pub timestamp: Option<Timestamp>,
    /// The timestamp at the measuring of the span started.
    #[serde(with = "ts_rfc3339")]
    pub start_timestamp: Timestamp,
    /// Describes the status of the span (e.g. `ok`, `cancelled`, etc.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<SpanStatus>,
    /// Optional tags to be attached to the span.
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub tags: Map<String, String>,
    /// Optional extra information to be sent with the span.
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub data: Map<String, Value>,
}

#[allow(dead_code)]
#[derive(Serialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SpanStatus {
    /// The operation completed successfully.
    #[serde(rename = "ok")]
    Ok,
    /// Deadline expired before operation could complete.
    #[serde(rename = "deadline_exceeded")]
    DeadlineExceeded,
    /// 401 Unauthorized (actually does mean unauthenticated according to RFC 7235)
    #[serde(rename = "unauthenticated")]
    Unauthenticated,
    /// 403 Forbidden
    #[serde(rename = "permission_denied")]
    PermissionDenied,
    /// 404 Not Found. Some requested entity (file or directory) was not found.
    #[serde(rename = "not_found")]
    NotFound,
    /// 429 Too Many Requests
    #[serde(rename = "resource_exhausted")]
    ResourceExhausted,
    /// Client specified an invalid argument. 4xx.
    #[serde(rename = "invalid_argument")]
    InvalidArgument,
    /// 501 Not Implemented
    #[serde(rename = "unimplemented")]
    Unimplemented,
    /// 503 Service Unavailable
    #[serde(rename = "unavailable")]
    Unavailable,
    /// Other/generic 5xx.
    #[serde(rename = "internal_error")]
    InternalError,
    /// Unknown. Any non-standard HTTP status code.
    #[serde(rename = "unknown_error")]
    UnknownError,
    /// The operation was cancelled (typically by the user).
    #[serde(rename = "cancelled")]
    Cancelled,
    /// Already exists (409)
    #[serde(rename = "already_exists")]
    AlreadyExists,
    /// Operation was rejected because the system is not in a state required for the operation's
    #[serde(rename = "failed_precondition")]
    FailedPrecondition,
    /// The operation was aborted, typically due to a concurrency issue.
    #[serde(rename = "aborted")]
    Aborted,
    /// Operation was attempted past the valid range.
    #[serde(rename = "out_of_range")]
    OutOfRange,
    /// Unrecoverable data loss or corruption
    #[serde(rename = "data_loss")]
    DataLoss,
}

impl From<u16> for SpanStatus {
    #[allow(clippy::match_overlapping_arm)]
    fn from(status: u16) -> Self {
        match status {
            100..=199 => Self::Ok,
            200..=299 => Self::Ok,
            300..=399 => Self::Ok,
            400 => Self::InvalidArgument,
            401 => Self::Unauthenticated,
            403 => Self::PermissionDenied,
            404 => Self::NotFound,
            409 => Self::AlreadyExists,
            429 => Self::ResourceExhausted,
            400..=499 => Self::InvalidArgument,
            501 => Self::Unimplemented,
            503 => Self::Unavailable,
            500..=599 => Self::InternalError,
            _ => Self::UnknownError,
        }
    }
}

/// Represents a tracing transaction.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Transaction<'a> {
    /// The ID of the event
    #[serde(serialize_with = "super::utils::serialize_id")]
    pub event_id: uuid::Uuid,
    /// The transaction name.
    #[serde(
        rename = "transaction",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub name: Option<String>,
    /// A release identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release: Option<Cow<'a, str>>,
    /// An optional environment identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<Cow<'a, str>>,
    /// Optional tags to be attached to the event.
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub tags: Map<String, String>,
    /// Optional extra information to be sent with the event.
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub extra: Map<String, Value>,
    /// SDK metadata
    // #[serde(default, skip_serializing_if = "Option::is_none")]
    // pub sdk: Option<Cow<'a, ClientSdkInfo>>,
    /// A platform identifier for this event.
    // #[serde(
    //     default = "event::default_platform",
    //     skip_serializing_if = "event::is_default_platform"
    // )]
    pub platform: Cow<'a, str>,
    /// The end time of the transaction.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "ts_rfc3339_opt"
    )]
    pub timestamp: Option<Timestamp>,
    /// The start time of the transaction.
    #[serde(with = "ts_rfc3339")]
    pub start_timestamp: Timestamp,
    /// The collection of finished spans part of this transaction.
    pub spans: Vec<Span>,
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub contexts: Map<String, Context>,
    /// Optionally HTTP request data to be sent along.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<Request>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub breadcrumbs: Vec<Breadcrumb>,
}

impl<'a> Default for Transaction<'a> {
    fn default() -> Self {
        Self {
            event_id: uuid::Builder::from_random_bytes(crate::crypto::get_random_values().unwrap())
                .into_uuid(),
            name: Default::default(),
            release: Default::default(),
            environment: Default::default(),
            tags: Default::default(),
            extra: Default::default(),
            platform: Default::default(),
            timestamp: Default::default(),
            start_timestamp: Default::default(),
            spans: Default::default(),
            contexts: Default::default(),
            request: Default::default(),
            user: Default::default(),
            breadcrumbs: Default::default(),
        }
    }
}

/// Represents a full event for Sentry.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Event<'a> {
    /// The ID of the event
    #[serde(serialize_with = "super::utils::serialize_id")]
    pub event_id: uuid::Uuid,
    /// The level of the event (defaults to error)
    pub level: Level,
    // /// An optional fingerprint configuration to override the default.
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub fingerprint: Cow<'a, [Cow<'a, str>]>,
    /// The culprit of the event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub culprit: Option<String>,
    /// The transaction name of the event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction: Option<String>,
    /// A message to be sent with the event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Optionally a log entry that can be used instead of the message for
    /// more complex cases.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logentry: Option<LogEntry>,
    /// Optionally the name of the logger that created this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logger: Option<String>,
    /// Optionally a name to version mapping of installed modules.
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub modules: Map<String, String>,
    /// A platform identifier for this event.
    // #[serde(
    //     default = "event::default_platform",
    //     skip_serializing_if = "event::is_default_platform"
    // )]
    pub platform: Cow<'a, str>,
    /// The timestamp of when the event was created.
    ///
    /// This can be set to `None` in which case the server will set a timestamp.
    #[serde(with = "ts_rfc3339")]
    pub timestamp: Timestamp,
    /// Optionally the server (or device) name of this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_name: Option<Cow<'a, str>>,
    /// A release identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release: Option<Cow<'a, str>>,
    /// An optional distribution identifer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dist: Option<Cow<'a, str>>,
    /// An optional environment identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<Cow<'a, str>>,
    /// Optionally user data to be sent along.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
    /// Optionally HTTP request data to be sent along.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request: Option<Request>,
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub contexts: Map<String, Context>,
    /// List of breadcrumbs to send along.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub breadcrumbs: Vec<Breadcrumb>,
    /// Exceptions to be attached (one or multiple if chained).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exception: Vec<Exception>,
    /// Optional tags to be attached to the event.
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub tags: Map<String, String>,
    /// Optional extra information to be sent with the event.
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub extra: Map<String, Value>,
}

impl<'a> Default for Event<'a> {
    fn default() -> Self {
        Self {
            event_id: uuid::Builder::from_random_bytes(crate::crypto::get_random_values().unwrap())
                .into_uuid(),
            level: Default::default(),
            culprit: Default::default(),
            transaction: Default::default(),
            message: Default::default(),
            logentry: Default::default(),
            logger: Default::default(),
            modules: Default::default(),
            platform: "other".into(),
            timestamp: Default::default(),
            server_name: Default::default(),
            release: Default::default(),
            dist: Default::default(),
            environment: Default::default(),
            user: Default::default(),
            request: Default::default(),
            contexts: Default::default(),
            breadcrumbs: Default::default(),
            exception: Default::default(),
            tags: Default::default(),
            extra: Default::default(),
        }
    }
}

#[derive(Serialize, Default, Clone, Debug, PartialEq, Eq)]
pub struct LogEntry {
    /// The log message with parameters replaced by `%s`
    pub message: String,
    /// Positional parameters to be inserted into the log entry.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<Value>,
}

/// Represents a single breadcrumb.
#[derive(Default, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Breadcrumb {
    pub timestamp: Timestamp,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub ty: Option<Cow<'static, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<Cow<'static, str>>,
    pub level: Level,
    /// An optional human readbale message for the breadcrumb.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Arbitrary breadcrumb data that should be send along.
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub data: Map<String, Value>,
}

#[derive(Default, Debug, Serialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    Debug,
    #[default]
    Info,
    Warning,
    Error,
    Fatal,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Level::Debug => write!(f, "debug"),
            Level::Info => write!(f, "info"),
            Level::Warning => write!(f, "warning"),
            Level::Error => write!(f, "error"),
            Level::Fatal => write!(f, "fatal"),
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "type")]
#[non_exhaustive]
pub enum Context {
    Trace(TraceContext),
    // #[serde(rename = "unknown")]
    // Other(Map<String, Value>),
}

#[derive(Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct TraceContext {
    /// The ID of the trace event
    pub span_id: SpanId,
    /// Determines which trace the transaction belongs to.
    pub trace_id: TraceId,
    /// Determines the parent of this transaction if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<SpanId>,
    /// Short code identifying the type of operation the transaction is measuring.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub op: Option<String>,
    /// Human readable detail description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Describes the status of the span (e.g. `ok`, `cancelled`, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<SpanStatus>,
}

/// The different types an attachment can have.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AttachmentType {
    Attachment,
}

impl Default for AttachmentType {
    fn default() -> Self {
        Self::Attachment
    }
}

impl AttachmentType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Attachment => "event.attachment",
        }
    }
}

#[derive(Clone)]
pub struct Attachment {
    pub buffer: Rc<[u8]>,
    pub filename: Cow<'static, str>,
    pub content_type: Option<Cow<'static, str>>,
    pub ty: Option<AttachmentType>,
}

impl Attachment {
    pub fn to_writer<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        serde_json::to_writer(
            &mut *writer,
            &serde_json::json!({
                "type": "attachment",
                "length": self.buffer.len(),
                "filename": self.filename,
                "attachment_type": self.ty.unwrap_or_default().as_str(),
                "content_type": self.content_type.as_deref().unwrap_or("application/octet-stream")
            }),
        )?;

        writeln!(writer)?;
        writer.write_all(&self.buffer)?;

        Ok(())
    }
}

impl fmt::Debug for Attachment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Attachment")
            .field("buffer", &self.buffer.len())
            .field("filename", &self.filename)
            .field("content_type", &self.content_type)
            .field("type", &self.ty)
            .finish()
    }
}
