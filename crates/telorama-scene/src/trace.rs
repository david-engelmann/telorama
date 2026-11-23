//! Canonical trace model shared by ingest, derivations, and renderers.
//!
//! A `Span` is the single normalized shape every dialect adapter targets.
//! M1 ships the service-telemetry model only; the agent-run model lands
//! with the dialect adapters in M2.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The kind of work a span represents, mirroring the OpenTelemetry span
/// kinds. Edge formation rules in later milestones depend on this staying
/// faithful to the source data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpanKind {
    /// Work inside a single process, neither a remote call nor
    /// a message handoff.
    Internal,
    /// Handling of a remote request, the receiving end of a
    /// Client span's call.
    Server,
    /// A remote call out, answered by a Server span on the
    /// other side.
    Client,
    /// Creation of a message handed to a broker or queue for
    /// later delivery.
    Producer,
    /// Handling of a message a Producer created, possibly long
    /// after it was sent.
    Consumer,
}

/// One span in a trace, normalized from any ingest dialect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    /// Identifier of the trace this span belongs to, shared by
    /// every span in that trace.
    pub trace_id: String,
    /// This span's own identifier, unique within its trace.
    pub span_id: String,
    /// The parent span's identifier, or None when this span is
    /// a root.
    pub parent_span_id: Option<String>,
    /// The operation name the source instrumentation recorded.
    pub name: String,
    /// The span's kind. Edge and staging derivations read it.
    pub kind: SpanKind,
    /// Span start in Unix nanoseconds.
    pub start_unix_nano: i64,
    /// Span end in Unix nanoseconds.
    pub end_unix_nano: i64,
    /// True when the source marked the span as failed.
    pub is_error: bool,
    /// The service that produced the span, from the source's
    /// resource.
    pub service: String,
    /// Source attributes normalized to strings. Ingest keeps
    /// raw values here so malformed data stays visible for
    /// diagnosis.
    #[serde(default)]
    pub attributes: BTreeMap<String, String>,
}

impl Span {
    /// Wall-clock duration of the span in nanoseconds. Never negative in
    /// output: a malformed span with end before start reports zero, and
    /// ingest keeps the raw values visible in attributes for diagnosis.
    pub fn duration_nanos(&self) -> i64 {
        (self.end_unix_nano - self.start_unix_nano).max(0)
    }
}
