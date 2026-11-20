//! The Codex dialect adapter: OTLP log events in, one canonical agent
//! run out, with renderer-side token pricing.
//!
//! Codex carries the same event family as Claude Code under the
//! `codex.` prefix (user_prompt, api_request, tool_decision,
//! tool_result), keyed by `conversation.id` instead of `session.id`,
//! with three differences this adapter exists to absorb. Token counts
//! do not ride on the request event; they arrive on the streamed
//! response's completion event and join the turn's latest model call.
//! Time to first token is its own event. And Codex exports no cost at
//! all, so telorama prices the tokens itself from the table below and
//! marks the run as priced: the table's date and every unpriced model
//! are named on the run and printed on the render, so an estimate
//! never poses as a measured cost.
//!
//! Two normalizations keep the canonical model's meaning fixed across
//! dialects. Codex reports input tokens gross, cache reads included;
//! the model counts non-cache input only, so the adapter subtracts
//! the cache read before storing. And Codex's span structure, subagent
//! nesting in particular, is not verified against a real export, so
//! this adapter merges nothing from spans: they are counted in
//! coverage as seen, never as merged.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use telorama_scene::{
    AgentRun, AgentStep, AgentTurn, Coverage, ModelCall, Pricing, Span, ToolCall,
};

use crate::IngestError;
use crate::events::{Attr, parse_events};

fn fail(message: impl Into<String>) -> IngestError {
    IngestError {
        message: message.into(),
    }
}

pub use crate::pricing::PRICE_TABLE_AS_OF;
use crate::pricing::price_call;

struct TurnBuilder {
    prompt_time: Option<i64>,
    steps: Vec<AgentStep>,
    ttfts: VecDeque<i64>,
    events: u64,
}

impl TurnBuilder {
    fn new() -> Self {
        Self {
            prompt_time: None,
            steps: Vec::new(),
            ttfts: VecDeque::new(),
            events: 0,
        }
    }
}

/// Build the canonical run for one Codex conversation from its OTLP
/// log export. The export must carry exactly one conversation. Spans
/// may be supplied for coverage counting; this dialect merges none.
pub fn parse_codex_session(logs_text: &str, spans: &[Span]) -> Result<AgentRun, IngestError> {
    let events = parse_events(logs_text)?;
    let mut conversation_ids: Vec<&str> = events
        .iter()
        .filter_map(|e| e.str_attr("conversation.id"))
        .collect();
    conversation_ids.sort_unstable();
    conversation_ids.dedup();
    let session_id = match conversation_ids.len() {
        0 => {
            return Err(fail(
                "no log events carry a conversation.id: nothing to build a run from",
            ));
        }
        1 => conversation_ids[0].to_string(),
        n => {
            return Err(fail(format!(
                "the export carries {n} conversations: split it and adapt one session at a time"
            )));
        }
    };
    let relevant: Vec<&crate::events::Event> = events
        .iter()
        .filter(|e| e.str_attr("conversation.id") == Some(session_id.as_str()))
        .collect();

    let mut order: Vec<String> = Vec::new();
    let mut builders: BTreeMap<String, TurnBuilder> = BTreeMap::new();
    let mut decisions: BTreeMap<(String, String), String> = BTreeMap::new();
    let mut current: Option<String> = None;
    for event in &relevant {
        if event.name == "codex.user_prompt" {
            let turn_id = event
                .str_attr("turn.id")
                .map(str::to_string)
                .unwrap_or_else(|| format!("turn-{}", order.len() + 1));
            if !builders.contains_key(&turn_id) {
                order.push(turn_id.clone());
                builders.insert(turn_id.clone(), TurnBuilder::new());
            }
            let turn = builders.get_mut(&turn_id).expect("builder exists");
            turn.events += 1;
            if turn.prompt_time.is_none() {
                turn.prompt_time = Some(event.time_nano);
            }
            current = Some(turn_id);
            continue;
        }
        let turn_id = match event.str_attr("turn.id") {
            Some(id) => id.to_string(),
            None => match &current {
                Some(id) => id.clone(),
                None => continue,
            },
        };
        if !builders.contains_key(&turn_id) {
            order.push(turn_id.clone());
            builders.insert(turn_id.clone(), TurnBuilder::new());
        }
        builders.get_mut(&turn_id).expect("builder exists").events += 1;
        if event.name == "codex.tool_decision" {
            if let (Some(tool), Some(decision)) =
                (event.str_attr("tool_name"), event.str_attr("decision"))
            {
                decisions.insert((turn_id.clone(), tool.to_string()), decision.to_string());
            }
            continue;
        }
        let turn = builders.get_mut(&turn_id).expect("builder exists");
        match event.name.as_str() {
            "codex.turn_ttft" => {
                turn.ttfts
                    .push_back(event.i64_attr("duration_ms") * 1_000_000);
            }
            "codex.api_request" => {
                let duration = event.i64_attr("duration_ms") * 1_000_000;
                let status = event.str_attr("status");
                turn.steps.push(AgentStep::ModelCall(ModelCall {
                    model: event
                        .str_attr("model")
                        .unwrap_or("unknown model")
                        .to_string(),
                    start_unix_nano: event.time_nano - duration,
                    end_unix_nano: event.time_nano,
                    ttft_nanos: turn.ttfts.pop_front(),
                    attempts: event.i64_attr("attempt").max(1) as u32,
                    is_error: matches!(status, Some(s) if s != "success"),
                    input_tokens: 0,
                    output_tokens: 0,
                    cache_read_tokens: 0,
                    cache_creation_tokens: 0,
                    cost_usd: 0.0,
                }));
            }
            "codex.sse_event" => {
                if event.str_attr("kind") != Some("response.completed") {
                    continue;
                }
                let Some(AgentStep::ModelCall(call)) = turn
                    .steps
                    .iter_mut()
                    .rev()
                    .find(|s| matches!(s, AgentStep::ModelCall(_)))
                else {
                    return Err(fail(format!(
                        "codex token counts arrived in turn {turn_id} before any model call"
                    )));
                };
                let reported_input = event.i64_attr("input_tokens") as u64;
                let mut cache = event.i64_attr("cached_tokens") as u64;
                if cache == 0 {
                    cache = event.i64_attr("cache_read_tokens") as u64;
                }
                call.cache_read_tokens = cache;
                call.input_tokens = reported_input.saturating_sub(cache);
                call.output_tokens = event.i64_attr("output_tokens") as u64;
            }
            "codex.tool_result" => {
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
                let decision = decisions.get(&(turn_id.clone(), name.clone())).cloned();
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

    let mut priced_models: BTreeSet<String> = BTreeSet::new();
    let mut unpriced_models: BTreeSet<String> = BTreeSet::new();
    let mut turns = Vec::new();
    for turn_id in order {
        let mut builder = builders.remove(&turn_id).expect("builder exists");
        if builder.steps.is_empty() && builder.prompt_time.is_none() {
            continue;
        }
        builder
            .steps
            .sort_by_key(|s| (s.start_unix_nano(), s.end_unix_nano()));
        let mut input_tokens = 0u64;
        let mut output_tokens = 0u64;
        let mut cache_read_tokens = 0u64;
        let mut cost_usd = 0.0f64;
        for step in &mut builder.steps {
            let AgentStep::ModelCall(call) = step else {
                continue;
            };
            match price_call(call) {
                Some(cost) => {
                    call.cost_usd = cost;
                    priced_models.insert(call.model.clone());
                }
                None => {
                    unpriced_models.insert(call.model.clone());
                }
            }
            input_tokens += call.input_tokens;
            output_tokens += call.output_tokens;
            cache_read_tokens += call.cache_read_tokens;
            cost_usd += call.cost_usd;
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
            prompt_id: turn_id,
            start_unix_nano: start,
            end_unix_nano: end,
            steps: builder.steps,
            input_tokens,
            output_tokens,
            cache_read_tokens,
            cache_creation_tokens: 0,
            cost_usd,
            coverage: Coverage::logs(builder.events, 0),
        });
    }
    // Events that arrived before the first prompt belong to no
    // turn; the first turn's coverage counts them so the turn
    // coverages still sum to the run coverage.
    let counted: u64 = turns.iter().map(|t| t.coverage.log_events).sum();
    let total = relevant.len() as u64;
    if total > counted
        && let Some(first) = turns.first_mut()
    {
        first.coverage.log_events += total - counted;
    }
    // Seen spans belong to the turn whose window holds their
    // midpoint. A span in no turn's window counts toward the
    // first turn, so the turn coverages sum to the run coverage
    // on this axis too.
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
    let no_turns = turns.is_empty();
    Ok(AgentRun {
        session_id,
        dialect: "codex".to_string(),
        turns,
        coverage: Coverage {
            // Coverage counts what the run's turns folded; a run
            // with no turns folded nothing.
            log_events: if no_turns { 0 } else { relevant.len() as u64 },
            spans_seen: if no_turns { 0 } else { spans.len() as u64 },
            spans_merged: 0,
        },
        pricing: Some(Pricing {
            table_as_of: PRICE_TABLE_AS_OF.to_string(),
            priced_models: priced_models.into_iter().collect(),
            unpriced_models: unpriced_models.into_iter().collect(),
        }),
    })
}
