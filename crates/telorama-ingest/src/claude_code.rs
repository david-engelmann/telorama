//! The Claude Code dialect adapter: OTLP log events in, one canonical
//! agent run out, with span fusion for waits and subagents.
//!
//! Claude Code's GA telemetry carries the money facts on log events
//! (tokens, cost, per-call timing) and the structure on spans (the
//! interaction tree, tool spans with their blocked_on_user children,
//! subagent spans carrying agent ids). The fusion rule is stated, not
//! implied: events win for tokens, cost, and step timing; spans win
//! for permission waits and subagent structure; the coverage counts
//! on the run say how much each source contributed.

use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use telorama_scene::{
    AgentRun, AgentStep, AgentTurn, Coverage, ModelCall, Span, SubagentRun, ToolCall,
};

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
    int_value: Option<String>,
    #[serde(default, rename = "doubleValue")]
    double_value: Option<f64>,
    #[serde(default, rename = "boolValue")]
    bool_value: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
enum Attr {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl Attr {
    fn as_str(&self) -> Option<&str> {
        match self {
            Attr::Str(s) => Some(s),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self {
            Attr::Int(v) => Some(*v),
            Attr::Float(v) => Some(*v as i64),
            Attr::Str(s) => s.parse().ok(),
            Attr::Bool(_) => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match self {
            Attr::Float(v) => Some(*v),
            Attr::Int(v) => Some(*v as f64),
            Attr::Str(s) => s.parse().ok(),
            Attr::Bool(_) => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            Attr::Bool(v) => Some(*v),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct Event {
    time_nano: i64,
    name: String,
    attrs: BTreeMap<String, Attr>,
}

impl Event {
    fn str_attr(&self, key: &str) -> Option<&str> {
        self.attrs.get(key).and_then(Attr::as_str)
    }

    fn i64_attr(&self, key: &str) -> i64 {
        self.attrs.get(key).and_then(Attr::as_i64).unwrap_or(0)
    }

    fn f64_attr(&self, key: &str) -> f64 {
        self.attrs.get(key).and_then(Attr::as_f64).unwrap_or(0.0)
    }
}

fn parse_events(text: &str) -> Result<Vec<Event>, IngestError> {
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
                        } else if let Some(s) = value.int_value {
                            match s.parse::<i64>() {
                                Ok(v) => Attr::Int(v),
                                Err(_) => Attr::Str(s),
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

struct TurnBuilder {
    prompt_time: Option<i64>,
    steps: Vec<AgentStep>,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_creation_tokens: u64,
    cost_usd: f64,
}

impl TurnBuilder {
    fn new() -> Self {
        Self {
            prompt_time: None,
            steps: Vec::new(),
            input_tokens: 0,
            output_tokens: 0,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
            cost_usd: 0.0,
        }
    }
}

/// Build the canonical run for one Claude Code session from its OTLP
/// log export, fusing already-parsed spans for permission waits and
/// subagent structure. The export must carry exactly one session.
pub fn parse_claude_code_session(
    logs_text: &str,
    spans: &[Span],
) -> Result<AgentRun, IngestError> {
    let events = parse_events(logs_text)?;
    let mut session_ids: Vec<&str> = events
        .iter()
        .filter_map(|e| e.str_attr("session.id"))
        .collect();
    session_ids.sort_unstable();
    session_ids.dedup();
    let session_id = match session_ids.len() {
        0 => {
            return Err(fail(
                "no log events carry a session.id: nothing to build a run from",
            ));
        }
        1 => session_ids[0].to_string(),
        n => {
            return Err(fail(format!(
                "the export carries {n} sessions: split it and adapt one session at a time"
            )));
        }
    };
    let relevant: Vec<&Event> = events
        .iter()
        .filter(|e| e.str_attr("session.id") == Some(session_id.as_str()))
        .collect();

    let mut order: Vec<String> = Vec::new();
    let mut builders: BTreeMap<String, TurnBuilder> = BTreeMap::new();
    let mut decisions: BTreeMap<(String, String), String> = BTreeMap::new();
    for event in &relevant {
        let Some(prompt_id) = event.str_attr("prompt.id") else {
            continue;
        };
        let prompt_id = prompt_id.to_string();
        if !builders.contains_key(&prompt_id) {
            order.push(prompt_id.clone());
            builders.insert(prompt_id.clone(), TurnBuilder::new());
        }
        if event.name == "claude_code.tool_decision" {
            if let (Some(tool), Some(decision)) = (
                event.str_attr("tool_name"),
                event.str_attr("decision"),
            ) {
                decisions.insert((prompt_id.clone(), tool.to_string()), decision.to_string());
            }
            continue;
        }
        let turn = builders.get_mut(&prompt_id).expect("builder exists");
        match event.name.as_str() {
            "claude_code.user_prompt" => {
                if turn.prompt_time.is_none() {
                    turn.prompt_time = Some(event.time_nano);
                }
            }
            "claude_code.api_request" => {
                let duration = event.i64_attr("duration_ms") * 1_000_000;
                let ttft_ms = event.i64_attr("ttft_ms");
                let call = ModelCall {
                    model: event
                        .str_attr("model")
                        .unwrap_or("unknown model")
                        .to_string(),
                    start_unix_nano: event.time_nano - duration,
                    end_unix_nano: event.time_nano,
                    ttft_nanos: if ttft_ms > 0 {
                        Some(ttft_ms * 1_000_000)
                    } else {
                        None
                    },
                    attempts: event.i64_attr("attempt").max(1) as u32,
                    is_error: false,
                    input_tokens: event.i64_attr("input_tokens") as u64,
                    output_tokens: event.i64_attr("output_tokens") as u64,
                    cache_read_tokens: event.i64_attr("cache_read_tokens") as u64,
                    cache_creation_tokens: event.i64_attr("cache_creation_tokens") as u64,
                    cost_usd: event.f64_attr("cost_usd"),
                };
                turn.input_tokens += call.input_tokens;
                turn.output_tokens += call.output_tokens;
                turn.cache_read_tokens += call.cache_read_tokens;
                turn.cache_creation_tokens += call.cache_creation_tokens;
                turn.cost_usd += call.cost_usd;
                turn.steps.push(AgentStep::ModelCall(call));
            }
            "claude_code.api_error" => {
                let duration = event.i64_attr("duration_ms") * 1_000_000;
                turn.steps.push(AgentStep::ModelCall(ModelCall {
                    model: event
                        .str_attr("model")
                        .unwrap_or("unknown model")
                        .to_string(),
                    start_unix_nano: event.time_nano - duration,
                    end_unix_nano: event.time_nano,
                    ttft_nanos: None,
                    attempts: event.i64_attr("attempt").max(1) as u32,
                    is_error: true,
                    input_tokens: 0,
                    output_tokens: 0,
                    cache_read_tokens: 0,
                    cache_creation_tokens: 0,
                    cost_usd: 0.0,
                }));
            }
            "claude_code.tool_result" => {
                let duration = event.i64_attr("duration_ms") * 1_000_000;
                let name = event
                    .str_attr("tool_name")
                    .unwrap_or("unknown tool")
                    .to_string();
                let success = event
                    .attrs
                    .get("success")
                    .and_then(Attr::as_bool)
                    .unwrap_or(true);
                let decision = decisions.get(&(prompt_id.clone(), name.clone())).cloned();
                turn.steps.push(AgentStep::ToolCall(ToolCall {
                    name,
                    start_unix_nano: event.time_nano - duration,
                    end_unix_nano: event.time_nano,
                    wait_nanos: 0,
                    is_error: !success,
                    decision,
                }));
            }
            _ => {}
        }
    }
    for turn in builders.values_mut() {
        turn.steps
            .sort_by_key(|s| (s.start_unix_nano(), s.end_unix_nano()));
    }

    // Fusion. Spans contribute what logs cannot prove.
    let mut merged_ids: BTreeSet<&str> = BTreeSet::new();
    // Permission waits: a tool span containing a tool step, with
    // blocked_on_user children, sets the step's wait to the sum of
    // those children's durations.
    let mut blocked: BTreeMap<&str, Vec<&Span>> = BTreeMap::new();
    for span in spans {
        if span.name == "claude_code.tool.blocked_on_user"
            && let Some(parent) = span.parent_span_id.as_deref()
        {
            blocked.entry(parent).or_default().push(span);
        }
    }
    let tool_spans: Vec<&Span> = spans
        .iter()
        .filter(|s| s.name == "claude_code.tool")
        .collect();
    for turn in builders.values_mut() {
        for step in turn.steps.iter_mut() {
            let AgentStep::ToolCall(tool) = step else {
                continue;
            };
            for span in &tool_spans {
                if span.start_unix_nano <= tool.start_unix_nano
                    && tool.end_unix_nano <= span.end_unix_nano
                    && let Some(children) = blocked.get(span.span_id.as_str())
                {
                    let wait: i64 = children.iter().map(|c| c.duration_nanos()).sum();
                    if wait > tool.wait_nanos {
                        tool.wait_nanos = wait;
                        for child in children {
                            merged_ids.insert(child.span_id.as_str());
                        }
                    }
                }
            }
        }
    }
    // Subagents: a span carrying an agent_id opens a window. Top-level
    // steps whose midpoint falls inside move into the run; smallest
    // windows first, so nested subagents compose. A window that
    // captures no steps still leaves its run, placed by start time,
    // when it overlaps the turn.
    let mut agent_spans: Vec<&Span> = spans
        .iter()
        .filter(|s| s.attributes.contains_key("agent_id"))
        .collect();
    agent_spans.sort_by_key(|s| (s.end_unix_nano - s.start_unix_nano, s.start_unix_nano));
    for span in &agent_spans {
        let agent_id = span.attributes["agent_id"].clone();
        for turn in builders.values_mut() {
            let mut taken: Vec<AgentStep> = Vec::new();
            let mut kept: Vec<AgentStep> = Vec::new();
            let mut insert_at: Option<usize> = None;
            for step in turn.steps.drain(..) {
                let mid = (step.start_unix_nano() + step.end_unix_nano()) / 2;
                if span.start_unix_nano <= mid && mid < span.end_unix_nano {
                    if insert_at.is_none() {
                        insert_at = Some(kept.len());
                    }
                    taken.push(step);
                } else {
                    kept.push(step);
                }
            }
            if !taken.is_empty() {
                let run = SubagentRun {
                    agent_id: agent_id.clone(),
                    start_unix_nano: span.start_unix_nano,
                    end_unix_nano: span.end_unix_nano,
                    steps: taken,
                };
                kept.insert(
                    insert_at.unwrap_or(kept.len()),
                    AgentStep::Subagent(run),
                );
                merged_ids.insert(span.span_id.as_str());
            } else {
                // No steps captured: place an empty run by start time
                // when the window overlaps this turn's steps.
                let overlaps = kept.iter().any(|s| {
                    s.start_unix_nano() < span.end_unix_nano
                        && span.start_unix_nano < s.end_unix_nano()
                });
                if overlaps {
                    let pos = kept
                        .iter()
                        .position(|s| s.start_unix_nano() > span.start_unix_nano)
                        .unwrap_or(kept.len());
                    kept.insert(
                        pos,
                        AgentStep::Subagent(SubagentRun {
                            agent_id: agent_id.clone(),
                            start_unix_nano: span.start_unix_nano,
                            end_unix_nano: span.end_unix_nano,
                            steps: Vec::new(),
                        }),
                    );
                    merged_ids.insert(span.span_id.as_str());
                }
            }
            turn.steps = kept;
        }
    }

    let mut turns = Vec::new();
    for prompt_id in order {
        let builder = builders.remove(&prompt_id).expect("builder exists");
        if builder.steps.is_empty() && builder.prompt_time.is_none() {
            // A prompt id that only ever carried a decision event is
            // not a turn.
            continue;
        }
        let start = builder
            .prompt_time
            .or_else(|| builder.steps.iter().map(|s| s.start_unix_nano()).min())
            .unwrap_or(0);
        let end = builder
            .steps
            .iter()
            .map(|s| s.end_unix_nano())
            .max()
            .unwrap_or(start);
        turns.push(AgentTurn {
            prompt_id,
            start_unix_nano: start,
            end_unix_nano: end,
            steps: builder.steps,
            input_tokens: builder.input_tokens,
            output_tokens: builder.output_tokens,
            cache_read_tokens: builder.cache_read_tokens,
            cache_creation_tokens: builder.cache_creation_tokens,
            cost_usd: builder.cost_usd,
        });
    }
    Ok(AgentRun {
        session_id,
        dialect: "claude_code".to_string(),
        turns,
        coverage: Coverage {
            log_events: relevant.len() as u64,
            spans_seen: spans.len() as u64,
            spans_merged: merged_ids.len() as u64,
        },
    })
}
