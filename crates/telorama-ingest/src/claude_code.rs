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

use std::collections::{BTreeMap, BTreeSet};
use telorama_scene::{
    AgentRun, AgentStep, AgentTurn, Coverage, MAX_AGENT_NESTING, ModelCall, Span, SubagentRun,
    ToolCall,
};

use crate::events::{Attr, Event, parse_events};

use crate::IngestError;

fn fail(message: impl Into<String>) -> IngestError {
    IngestError {
        message: message.into(),
    }
}

/// The deepest subagent nesting in a step list, counted in
/// subagent levels, a list with no subagents at zero. An
/// explicit stack, not recursion (INV-87).
fn nesting_depth(steps: &[AgentStep]) -> usize {
    fn level(step: &AgentStep) -> usize {
        usize::from(matches!(step, AgentStep::Subagent(_)))
    }
    let mut deepest = 0usize;
    let mut stack: Vec<(&AgentStep, usize)> = steps.iter().map(|s| (s, level(s))).collect();
    while let Some((step, depth)) = stack.pop() {
        deepest = deepest.max(depth);
        if let AgentStep::Subagent(run) = step {
            for child in &run.steps {
                stack.push((child, depth + level(child)));
            }
        }
    }
    deepest
}

struct TurnBuilder {
    prompt_time: Option<i64>,
    steps: Vec<AgentStep>,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_creation_tokens: u64,
    cost_usd: f64,
    events: u64,
    merged_ids: BTreeSet<String>,
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
            events: 0,
            merged_ids: BTreeSet::new(),
        }
    }
}

/// Build the canonical run for one Claude Code session from its OTLP
/// log export, fusing already-parsed spans for permission waits and
/// subagent structure. The export must carry exactly one session.
pub fn parse_claude_code_session(logs_text: &str, spans: &[Span]) -> Result<AgentRun, IngestError> {
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
        builders.get_mut(&prompt_id).expect("builder exists").events += 1;
        if event.name == "claude_code.tool_decision" {
            if let (Some(tool), Some(decision)) =
                (event.str_attr("tool_name"), event.str_attr("decision"))
            {
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
            "claude_code.api_refusal" => {
                // A refusal is a failed call with no tokens, given
                // exactly the shape an api_error call gets, so the
                // step list and the error drawing see it (INV-83).
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
                            turn.merged_ids.insert(child.span_id.clone());
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
                if nesting_depth(&taken) + 1 > MAX_AGENT_NESTING {
                    return Err(fail(format!(
                        "session nests agents deeper than the {MAX_AGENT_NESTING} level budget"
                    )));
                }
                let run = SubagentRun {
                    agent_id: agent_id.clone(),
                    start_unix_nano: span.start_unix_nano,
                    end_unix_nano: span.end_unix_nano,
                    steps: taken,
                };
                kept.insert(insert_at.unwrap_or(kept.len()), AgentStep::Subagent(run));
                turn.merged_ids.insert(span.span_id.clone());
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
                    turn.merged_ids.insert(span.span_id.clone());
                }
            }
            turn.steps = kept;
        }
    }

    // Session-level records carry no prompt id. They belong to the
    // session setup, so the first turn's coverage counts them; a
    // truncated export keeps them, which keeps frame coverage pure.
    let stray = relevant
        .iter()
        .filter(|e| e.str_attr("prompt.id").is_none())
        .count() as u64;
    if stray > 0
        && let Some(first) = order.first()
        && let Some(builder) = builders.get_mut(first)
    {
        builder.events += stray;
    }
    let mut turns = Vec::new();
    let mut carry = 0u64;
    let mut kept_merged: BTreeSet<String> = BTreeSet::new();
    for prompt_id in order {
        let builder = builders.remove(&prompt_id).expect("builder exists");
        if builder.steps.is_empty() && builder.prompt_time.is_none() {
            // A prompt id that only ever carried a decision event is
            // not a turn. Its events carry forward so the turn
            // coverages still sum to the run coverage.
            carry += builder.events;
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
        kept_merged.extend(builder.merged_ids.iter().cloned());
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
            coverage: Coverage::logs(builder.events + carry, builder.merged_ids.len() as u64),
        });
        carry = 0;
    }
    if carry > 0
        && let Some(last) = turns.last_mut()
    {
        last.coverage.log_events += carry;
    }
    // Seen spans belong to the turn whose window holds their
    // midpoint, the placement the subagent fusion uses. A span in
    // no turn's window counts toward the first turn, the stray
    // rule the log events follow, so the turn coverages sum to
    // the run coverage on this axis too.
    if !turns.is_empty() {
        for span in spans {
            let mid = span.start_unix_nano + (span.end_unix_nano - span.start_unix_nano) / 2;
            let idx = turns
                .iter()
                .position(|t| mid >= t.start_unix_nano && mid <= t.end_unix_nano)
                .unwrap_or(0);
            turns[idx].coverage.spans_seen += 1;
        }
    }
    // Coverage counts what the run's turns folded. A run whose
    // turns all dropped away folded nothing, so its counts are
    // zero and the turn sums still equal the run totals.
    let folded = !turns.is_empty();
    Ok(AgentRun {
        session_id,
        dialect: "claude_code".to_string(),
        turns,
        coverage: Coverage {
            log_events: if folded { relevant.len() as u64 } else { 0 },
            spans_seen: if folded { spans.len() as u64 } else { 0 },
            spans_merged: kept_merged.len() as u64,
        },
        pricing: None,
    })
}
