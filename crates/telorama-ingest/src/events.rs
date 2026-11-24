//! OTLP log-event parsing shared by the agent dialect adapters.
//!
//! Both dialects arrive as OTLP logs: one export document or JSON
//! Lines, each record carrying an `event.name` attribute and a bag of
//! typed attributes. This module is the one reader for that shape, so
//! the Claude Code and Codex adapters can never drift apart on how an
//! export is decoded.

use serde::Deserialize;
use std::collections::BTreeMap;

use crate::IngestError;

fn fail(message: impl Into<String>) -> IngestError {
    IngestError {
        message: message.into(),
    }
}

#[derive(Debug, Deserialize)]
struct LogsRequest {
    #[serde(default, rename = "resourceLogs")]
    resource_logs: Vec<ResourceLogs>,
}

#[derive(Debug, Deserialize)]
struct ResourceLogs {
    #[serde(default, rename = "scopeLogs")]
    scope_logs: Vec<ScopeLogs>,
}

#[derive(Debug, Deserialize)]
struct ScopeLogs {
    #[serde(default, rename = "logRecords")]
    log_records: Vec<LogRecord>,
}

#[derive(Debug, Deserialize)]
struct LogRecord {
    #[serde(default, rename = "timeUnixNano")]
    time_unix_nano: Option<String>,
    #[serde(default)]
    attributes: Vec<KeyValue>,
}

#[derive(Debug, Deserialize)]
struct KeyValue {
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
    #[serde(default, rename = "doubleValue")]
    double_value: Option<f64>,
    #[serde(default, rename = "boolValue")]
    bool_value: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Attr {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl Attr {
    pub(crate) fn as_str(&self) -> Option<&str> {
        match self {
            Attr::Str(s) => Some(s),
            _ => None,
        }
    }

    pub(crate) fn as_i64(&self) -> Option<i64> {
        match self {
            Attr::Int(v) => Some(*v),
            Attr::Float(v) => Some(*v as i64),
            Attr::Str(s) => s.parse().ok(),
            Attr::Bool(_) => None,
        }
    }

    pub(crate) fn as_f64(&self) -> Option<f64> {
        match self {
            Attr::Float(v) => Some(*v),
            Attr::Int(v) => Some(*v as f64),
            Attr::Str(s) => s.parse().ok(),
            Attr::Bool(_) => None,
        }
    }

    pub(crate) fn as_bool(&self) -> Option<bool> {
        match self {
            Attr::Bool(v) => Some(*v),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Event {
    pub(crate) time_nano: i64,
    pub(crate) name: String,
    pub(crate) attrs: BTreeMap<String, Attr>,
}

impl Event {
    pub(crate) fn str_attr(&self, key: &str) -> Option<&str> {
        self.attrs.get(key).and_then(Attr::as_str)
    }

    pub(crate) fn i64_attr(&self, key: &str) -> i64 {
        self.attrs.get(key).and_then(Attr::as_i64).unwrap_or(0)
    }

    pub(crate) fn f64_attr(&self, key: &str) -> f64 {
        self.attrs
            .get(key)
            .and_then(Attr::as_f64)
            .filter(|v| v.is_finite())
            .unwrap_or(0.0)
    }

    /// A token count read from an attribute. A negative
    /// count is malformed and reads as zero, the same
    /// outcome the span dialects get by parsing the
    /// attribute as unsigned (INV-88).
    pub(crate) fn token_attr(&self, key: &str) -> u64 {
        self.i64_attr(key).max(0) as u64
    }

    /// A millisecond duration read from an attribute,
    /// converted to nanoseconds with saturation, so an
    /// extreme value cannot overflow the conversion
    /// (INV-88).
    pub(crate) fn duration_nanos_attr(&self, key: &str) -> i64 {
        self.i64_attr(key).saturating_mul(1_000_000)
    }
}

pub(crate) fn parse_events(text: &str) -> Result<Vec<Event>, IngestError> {
    let docs: Vec<LogsRequest> = match serde_json::from_str::<LogsRequest>(text) {
        Ok(doc) => vec![doc],
        Err(_) => {
            let mut out = Vec::new();
            for (i, line) in text.lines().enumerate() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let doc: LogsRequest = serde_json::from_str(line).map_err(|e| {
                    fail(format!(
                        "log line {} is not an OTLP logs document: {e}",
                        i + 1
                    ))
                })?;
                out.push(doc);
            }
            out
        }
    };
    let mut events = Vec::new();
    for doc in docs {
        for resource in doc.resource_logs {
            for scope in resource.scope_logs {
                for record in scope.log_records {
                    let mut attrs = BTreeMap::new();
                    for kv in record.attributes {
                        let Some(value) = kv.value else { continue };
                        let attr = if let Some(s) = value.string_value {
                            Attr::Str(s)
                        } else if let Some(v) = value.int_value {
                            // OTLP JSON encodes intValue as a
                            // string. A raw JSON number is
                            // accepted too, the same leniency
                            // the traces reader applies, so
                            // the two readers cannot disagree
                            // on a document (INV-88).
                            match &v {
                                serde_json::Value::String(s) => match s.parse::<i64>() {
                                    Ok(n) => Attr::Int(n),
                                    Err(_) => Attr::Str(s.clone()),
                                },
                                serde_json::Value::Number(n) => match n.as_i64() {
                                    Some(n) => Attr::Int(n),
                                    None => Attr::Str(n.to_string()),
                                },
                                _ => continue,
                            }
                        } else if let Some(v) = value.double_value {
                            Attr::Float(v)
                        } else if let Some(v) = value.bool_value {
                            Attr::Bool(v)
                        } else {
                            continue;
                        };
                        attrs.insert(kv.key, attr);
                    }
                    let name = attrs
                        .get("event.name")
                        .and_then(Attr::as_str)
                        .unwrap_or("")
                        .to_string();
                    let time_nano = record
                        .time_unix_nano
                        .as_deref()
                        .and_then(|t| t.parse::<i64>().ok())
                        .unwrap_or(0);
                    events.push(Event {
                        time_nano,
                        name,
                        attrs,
                    });
                }
            }
        }
    }
    // Stable sort: input order breaks time ties, so the reading is
    // reproducible from the export alone.
    events.sort_by_key(|e| e.time_nano);
    Ok(events)
}
