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
    Internal,
    Server,
    Client,
    Producer,
    Consumer,
}

/// One span in a trace, normalized from any ingest dialect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub name: String,
    pub kind: SpanKind,
    pub start_unix_nano: i64,
    pub end_unix_nano: i64,
    pub is_error: bool,
    pub service: String,
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
