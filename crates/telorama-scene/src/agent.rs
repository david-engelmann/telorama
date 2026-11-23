//! The canonical agent-run model: one agent session as turns of model
//! calls, tool calls, and subagents.
//!
//! Dialect adapters (Claude Code in M2, Codex in M3) translate their
//! native telemetry into this shape, and the agent_run panel renders
//! only this shape, so a scene means the same thing whatever produced
//! the run.

use serde::{Deserialize, Serialize};

/// One agent session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentRun {
    /// The session's identifier in the source dialect. Frame
    /// scenes title themselves with it.
    pub session_id: String,
    /// The dialect that produced the run, for example "claude_code".
    pub dialect: String,
    /// The session's turns in the order their prompts arrived.
    pub turns: Vec<AgentTurn>,
    /// What the sources contributed across every turn, as
    /// session totals. Each turn carries its own share in its
    /// coverage.
    pub coverage: Coverage,
    /// Renderer-side pricing, present when the dialect exports tokens
    /// but no cost (Codex). The run's cost figures are then estimates
    /// from a price table the renderer owns, and the render says so.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pricing: Option<Pricing>,
}

/// The price table a run's costs were estimated from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pricing {
    /// The date the table was compiled, as an ISO date. A price table
    /// rots; the date rides on the run so a reader can judge it.
    pub table_as_of: String,
    /// Models the table priced, sorted.
    pub priced_models: Vec<String>,
    /// Models the session used that the table does not price, sorted.
    /// Their tokens are in the run and their cost is excluded from
    /// every total, never silently counted as zero-cost usage.
    pub unpriced_models: Vec<String>,
}

/// How much of the run each source proved. Log events are
/// authoritative for tokens and cost; spans are authoritative for
/// permission waits and subagent structure. The counts ride on the
/// model and print in every agent_run render, so a reader can see
/// what the picture rests on. On a turn, the same counts cover that
/// turn alone, which is what makes session frames pure functions of
/// a prefix.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Coverage {
    /// Log events folded into this part of the run. Log events
    /// are authoritative for tokens and cost.
    pub log_events: u64,
    /// Spans the sources supplied in this part of the run,
    /// counted before any merging.
    pub spans_seen: u64,
    /// Spans merged into the step tree. Spans are authoritative
    /// for permission waits and subagent structure.
    pub spans_merged: u64,
}

impl Coverage {
    /// Coverage for a turn of a log dialect: the events it folded
    /// and the spans it merged, with its seen-span count filled in
    /// by the parser's window pass.
    pub fn logs(log_events: u64, spans_merged: u64) -> Self {
        Coverage {
            log_events,
            spans_seen: 0,
            spans_merged,
        }
    }
}

/// One agent's reported state inside a live scene: what it is doing,
/// why, what it is waiting on, and what it will do next. The state is
/// runtime data supplied alongside spans and runs, never spec content.
/// The rationale is the point: parsing refuses an entry without one,
/// so the scene cannot render the lazy version of an agent's work.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentStateEntry {
    /// Who is reporting, a label the reader can follow across frames.
    pub agent: String,
    /// What the agent is doing now, in its own words.
    pub doing: String,
    /// Why it is doing that. Required, and enforced non-empty at parse.
    pub why: String,
    /// What the agent is blocked on, when it is blocked. Waiting is a
    /// first-class part of the picture, on the blocked-on-user
    /// precedent in the span tree.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waiting_on: Option<String>,
    /// What the agent will do next, when it knows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    /// When this state began, when the reporter knows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since_unix_ms: Option<i64>,
}

/// One turn: one user prompt and everything the agent did about it.
/// Token and cost totals are the turn's own, summed from its model
/// calls, subagents included.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentTurn {
    /// Identifier of the user prompt this turn answers, in the
    /// source dialect's own terms.
    pub prompt_id: String,
    /// The turn's first activity, in Unix nanoseconds.
    pub start_unix_nano: i64,
    /// The turn's last activity, in Unix nanoseconds.
    pub end_unix_nano: i64,
    /// The turn's steps in the order they ran.
    pub steps: Vec<AgentStep>,
    /// Input tokens summed over the turn's model calls,
    /// subagents included.
    pub input_tokens: u64,
    /// Output tokens summed over the turn's model calls,
    /// subagents included.
    pub output_tokens: u64,
    /// Tokens the turn's model calls read from the prompt cache.
    pub cache_read_tokens: u64,
    /// Tokens the turn's model calls wrote into the prompt cache.
    pub cache_creation_tokens: u64,
    /// The turn's cost in US dollars, summed over its model
    /// calls. An estimate from the price table when the run
    /// carries pricing.
    pub cost_usd: f64,
    /// What the sources contributed to this turn alone. The run's
    /// coverage is the session total; a frame's coverage is the sum
    /// over its prefix turns, plus the session's supplied span count
    /// where the dialect counts spans as an input statistic.
    #[serde(default)]
    pub coverage: Coverage,
}

/// One step inside a turn.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AgentStep {
    /// One model invocation, with its timing, tokens, and cost.
    ModelCall(ModelCall),
    /// One tool invocation, with any permission wait the spans
    /// observed.
    ToolCall(ToolCall),
    /// A delegated subagent's run, nested inside the turn that
    /// spawned it.
    Subagent(SubagentRun),
}

impl AgentStep {
    /// Visit every step in a list, descending into subagent runs.
    /// The one traversal of the step tree: totals, model censuses,
    /// and wait walks all ride it so they can never diverge on
    /// what a step tree contains.
    pub fn walk<'a>(steps: &'a [AgentStep], visit: &mut impl FnMut(&'a AgentStep)) {
        for step in steps {
            visit(step);
            if let AgentStep::Subagent(sub) = step {
                AgentStep::walk(&sub.steps, visit);
            }
        }
    }

    /// The step's start in Unix nanoseconds, whichever kind of
    /// step it is.
    pub fn start_unix_nano(&self) -> i64 {
        match self {
            AgentStep::ModelCall(s) => s.start_unix_nano,
            AgentStep::ToolCall(s) => s.start_unix_nano,
            AgentStep::Subagent(s) => s.start_unix_nano,
        }
    }

    /// The step's end in Unix nanoseconds, whichever kind of
    /// step it is.
    pub fn end_unix_nano(&self) -> i64 {
        match self {
            AgentStep::ModelCall(s) => s.end_unix_nano,
            AgentStep::ToolCall(s) => s.end_unix_nano,
            AgentStep::Subagent(s) => s.end_unix_nano,
        }
    }
}

/// One call to a model, successful or not.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelCall {
    /// The model that served the call, under the name the
    /// dialect reports. Pricing looks models up by this name.
    pub model: String,
    /// When the call began, in Unix nanoseconds.
    pub start_unix_nano: i64,
    /// When the call finished, in Unix nanoseconds.
    pub end_unix_nano: i64,
    /// Time to first token in nanoseconds, when the dialect
    /// reports it.
    pub ttft_nanos: Option<i64>,
    /// Attempts the call took, when the dialect reports retries.
    pub attempts: u32,
    /// True when the call ended in error, after any retries
    /// `attempts` records.
    pub is_error: bool,
    /// Input tokens for this call, with cache reads and writes
    /// counted in their own fields.
    pub input_tokens: u64,
    /// Output tokens for this call.
    pub output_tokens: u64,
    /// Input tokens served from the prompt cache.
    pub cache_read_tokens: u64,
    /// Input tokens written into the prompt cache.
    pub cache_creation_tokens: u64,
    /// Cost of this call in US dollars. An estimate from the
    /// price table when the run carries pricing.
    pub cost_usd: f64,
}

/// One tool invocation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    /// The tool's name as the dialect reports it.
    pub name: String,
    /// When the invocation began, in Unix nanoseconds.
    pub start_unix_nano: i64,
    /// When the invocation finished, in Unix nanoseconds.
    pub end_unix_nano: i64,
    /// Time blocked on the user for permission, taken from the
    /// dialect's wait spans when spans were supplied. Zero means no
    /// wait was observed, and the coverage line says whether spans
    /// were there to observe one.
    pub wait_nanos: i64,
    /// True when the invocation ended in error.
    pub is_error: bool,
    /// The permission decision the user gave, when the dialect
    /// reports one: "accept", "reject", and their kin.
    pub decision: Option<String>,
}

/// A delegated subagent and the steps it took. A subagent exists in
/// the model only where a span carried its agent id; log events
/// alone never invent one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubagentRun {
    /// The subagent's identifier, carried by the span that
    /// proved it exists.
    pub agent_id: String,
    /// When the subagent's first step began, in Unix
    /// nanoseconds.
    pub start_unix_nano: i64,
    /// When the subagent's last step ended, in Unix
    /// nanoseconds.
    pub end_unix_nano: i64,
    /// The steps the subagent took, in the order they ran.
    pub steps: Vec<AgentStep>,
}
