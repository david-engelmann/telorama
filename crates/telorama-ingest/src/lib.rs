//! telorama-ingest: OpenTelemetry data in, canonical spans out.
//!
//! M1 reads OTLP trace exports in JSON form, either a single
//! `ExportTraceServiceRequest` document or JSON Lines with one document
//! per line, from a file's text. Live backend adapters and the agent
//! dialect adapters arrive in later milestones behind this same
//! canonical output.

pub mod claude_code;
pub mod codex;
pub(crate) mod events;
mod pricing;
pub mod span_dialects;

use serde::Deserialize;
use std::collections::BTreeMap;
use std::fmt;
use telorama_scene::{Span, SpanKind, StackSample};

pub use claude_code::parse_claude_code_session;
pub use codex::parse_codex_session;
pub use span_dialects::{parse_genai_session, parse_openinference_session};

/// An ingest failure with a message specific enough to act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestError {
    pub message: String,
}

impl fmt::Display for IngestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for IngestError {}

fn fail(message: impl Into<String>) -> IngestError {
    IngestError {
        message: message.into(),
    }
}

#[derive(Debug, Deserialize)]
struct ExportRequest {
    #[serde(default, rename = "resourceSpans")]
    resource_spans: Vec<ResourceSpans>,
}

#[derive(Debug, Deserialize)]
struct ResourceSpans {
    #[serde(default)]
    resource: Option<Resource>,
    #[serde(default, rename = "scopeSpans")]
    scope_spans: Vec<ScopeSpans>,
}

#[derive(Debug, Deserialize)]
struct Resource {
    #[serde(default)]
    attributes: Vec<KeyValue>,
}

#[derive(Debug, Deserialize)]
struct ScopeSpans {
    #[serde(default)]
    spans: Vec<OtlpSpan>,
}

#[derive(Debug, Deserialize)]
struct KeyValue {
    #[serde(default)]
    key: String,
    #[serde(default)]
    value: Option<AnyValue>,
}

#[derive(Debug, Deserialize)]
struct AnyValue {
    #[serde(default, rename = "stringValue")]
    string_value: Option<String>,
    #[serde(default, rename = "intValue")]
    int_value: Option<serde_json::Value>,
    #[serde(default, rename = "boolValue")]
    bool_value: Option<bool>,
    #[serde(default, rename = "doubleValue")]
    double_value: Option<f64>,
}

impl AnyValue {
    fn to_display(&self) -> String {
        if let Some(s) = &self.string_value {
            return s.clone();
        }
        if let Some(i) = &self.int_value {
            // OTLP JSON encodes intValue as a string; a JSON string's
            // own text is its contents, not its quoted form.
            return match i {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
        }
        if let Some(b) = self.bool_value {
            return b.to_string();
        }
        if let Some(d) = self.double_value {
            return format!("{d}");
        }
        String::new()
    }
}

#[derive(Debug, Deserialize)]
struct OtlpSpan {
    #[serde(default, rename = "traceId")]
    trace_id: String,
    #[serde(default, rename = "spanId")]
    span_id: String,
    #[serde(default, rename = "parentSpanId")]
    parent_span_id: Option<String>,
    #[serde(default)]
    name: String,
    #[serde(default)]
    kind: i32,
    #[serde(default, rename = "startTimeUnixNano")]
    start: Option<serde_json::Value>,
    #[serde(default, rename = "endTimeUnixNano")]
    end: Option<serde_json::Value>,
    #[serde(default)]
    attributes: Vec<KeyValue>,
    #[serde(default)]
    status: Option<OtlpStatus>,
}

#[derive(Debug, Deserialize)]
struct OtlpStatus {
    #[serde(default)]
    code: i32,
}

fn nano_value(v: &Option<serde_json::Value>) -> Option<i64> {
    match v {
        Some(serde_json::Value::String(s)) => s.parse::<i64>().ok(),
        Some(serde_json::Value::Number(n)) => n.as_i64(),
        _ => None,
    }
}

fn kind_from_i32(kind: i32) -> SpanKind {
    match kind {
        2 => SpanKind::Server,
        3 => SpanKind::Client,
        4 => SpanKind::Producer,
        5 => SpanKind::Consumer,
        _ => SpanKind::Internal,
    }
}

fn attributes_of(list: &[KeyValue]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for kv in list {
        if let Some(v) = &kv.value {
            out.insert(kv.key.clone(), v.to_display());
        }
    }
    out
}

fn convert(doc: &ExportRequest, out: &mut Vec<Span>) -> Result<(), IngestError> {
    for rs in &doc.resource_spans {
        let resource_attrs = rs
            .resource
            .as_ref()
            .map(|r| attributes_of(&r.attributes))
            .unwrap_or_default();
        let service = resource_attrs
            .get("service.name")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        for scope in &rs.scope_spans {
            for raw in &scope.spans {
                let start = nano_value(&raw.start).ok_or_else(|| {
                    fail(format!(
                        "span {} is missing a numeric startTimeUnixNano",
                        raw.span_id
                    ))
                })?;
                let end = nano_value(&raw.end).ok_or_else(|| {
                    fail(format!(
                        "span {} is missing a numeric endTimeUnixNano",
                        raw.span_id
                    ))
                })?;
                let mut attributes = attributes_of(&raw.attributes);
                for (k, v) in &resource_attrs {
                    attributes
                        .entry(format!("resource.{k}"))
                        .or_insert_with(|| v.clone());
                }
                let status_error = raw.status.as_ref().map(|s| s.code) == Some(2);
                let attr_error = attributes
                    .get("error")
                    .map(|v| v == "true")
                    .unwrap_or(false);
                out.push(Span {
                    trace_id: raw.trace_id.clone(),
                    span_id: raw.span_id.clone(),
                    parent_span_id: raw.parent_span_id.clone().filter(|p| !p.is_empty()),
                    name: raw.name.clone(),
                    kind: kind_from_i32(raw.kind),
                    start_unix_nano: start,
                    end_unix_nano: end,
                    is_error: status_error || attr_error,
                    service: service.clone(),
                    attributes,
                });
            }
        }
    }
    Ok(())
}

/// Parse OTLP trace JSON text into canonical spans, sorted by the
/// canonical key (trace id, start, span id) so the output is a pure
/// function of the data, not of the file's internal ordering.
///
/// Accepts one export document, or JSON Lines with one document per
/// line. An empty input is an error, not an empty result.
pub fn parse_otlp_traces(text: &str) -> Result<Vec<Span>, IngestError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(fail("empty input: no OTLP document found"));
    }
    let mut spans = Vec::new();
    match serde_json::from_str::<ExportRequest>(trimmed) {
        Ok(doc) => convert(&doc, &mut spans)?,
        Err(single_err) => {
            // Fall back to JSON Lines.
            let mut any = false;
            for (lineno, line) in trimmed.lines().enumerate() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let doc: ExportRequest = serde_json::from_str(line).map_err(|e| {
                    fail(format!(
                        "line {} is not an OTLP export document: {e} (whole-document parse also failed: {single_err})",
                        lineno + 1
                    ))
                })?;
                convert(&doc, &mut spans)?;
                any = true;
            }
            if !any {
                return Err(fail(format!(
                    "input is not an OTLP export document: {single_err}"
                )));
            }
        }
    }
    if spans.is_empty() {
        return Err(fail("OTLP document contained no spans"));
    }
    spans.sort_by(|a, b| {
        a.trace_id
            .cmp(&b.trace_id)
            .then_with(|| a.start_unix_nano.cmp(&b.start_unix_nano))
            .then_with(|| a.span_id.cmp(&b.span_id))
    });
    Ok(spans)
}

/// A compact summary of a span set, in the spirit of a frame summary: an
/// agent reads this to learn what data exists before writing a spec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    pub span_count: usize,
    pub trace_count: usize,
    pub from_unix_nano: i64,
    pub to_unix_nano: i64,
    pub services: Vec<ServiceSummary>,
}

/// Per-service counts in a summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceSummary {
    pub service: String,
    pub spans: usize,
    pub errors: usize,
}

/// Summarize spans deterministically: services sorted by name, window
/// from the earliest start to the latest end.
pub fn summarize(spans: &[Span]) -> Summary {
    let mut traces: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    let mut by_service: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    let mut from = i64::MAX;
    let mut to = i64::MIN;
    for span in spans {
        traces.insert(span.trace_id.as_str());
        let entry = by_service.entry(span.service.as_str()).or_default();
        entry.0 += 1;
        if span.is_error {
            entry.1 += 1;
        }
        from = from.min(span.start_unix_nano);
        to = to.max(span.end_unix_nano);
    }
    Summary {
        span_count: spans.len(),
        trace_count: traces.len(),
        from_unix_nano: if spans.is_empty() { 0 } else { from },
        to_unix_nano: if spans.is_empty() { 0 } else { to },
        services: by_service
            .into_iter()
            .map(|(service, (spans, errors))| ServiceSummary {
                service: service.to_string(),
                spans,
                errors,
            })
            .collect(),
    }
}

/// Parse folded-stack profile text: one stack per line, frames joined
/// by semicolons from root to leaf, then a space and the sample count.
/// This is the export shape of `perf script` foldings, Brendan Gregg's
/// stackcollapse tools, and async-profiler's collapsed output.
///
/// Bad lines fail loudly with their line number: a profile with a
/// silent hole in it would render as a confident lie.
pub fn parse_folded_stacks(text: &str) -> Result<Vec<StackSample>, IngestError> {
    let mut out = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let line_no = i + 1;
        let Some((stack, count_text)) = line.rsplit_once(' ') else {
            return Err(fail(format!(
                "folded stack line {line_no} has no sample count: {line:?}"
            )));
        };
        let count: u64 = count_text.parse().map_err(|_| {
            fail(format!(
                "folded stack line {line_no} has a non-numeric sample count: {count_text:?}"
            ))
        })?;
        if count == 0 {
            return Err(fail(format!(
                "folded stack line {line_no} has a sample count of zero"
            )));
        }
        let frames: Vec<String> = stack.split(';').map(|f| f.trim().to_string()).collect();
        if frames.iter().any(|f| f.is_empty()) {
            return Err(fail(format!(
                "folded stack line {line_no} has an empty frame name"
            )));
        }
        out.push(StackSample { frames, count });
    }
    if out.is_empty() {
        return Err(fail("no folded stacks found: the input is empty"));
    }
    Ok(out)
}
