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
    pub session_id: String,
    /// The dialect that produced the run, for example "claude_code".
    pub dialect: String,
    pub turns: Vec<AgentTurn>,
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
/// what the picture rests on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Coverage {
    pub log_events: u64,
    pub spans_seen: u64,
    pub spans_merged: u64,
}

/// One turn: one user prompt and everything the agent did about it.
/// Token and cost totals are the turn's own, summed from its model
/// calls, subagents included.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentTurn {
    pub prompt_id: String,
    pub start_unix_nano: i64,
    pub end_unix_nano: i64,
    pub steps: Vec<AgentStep>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cost_usd: f64,
}

/// One step inside a turn.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AgentStep {
    ModelCall(ModelCall),
    ToolCall(ToolCall),
    Subagent(SubagentRun),
}

impl AgentStep {
    pub fn start_unix_nano(&self) -> i64 {
        match self {
            AgentStep::ModelCall(s) => s.start_unix_nano,
            AgentStep::ToolCall(s) => s.start_unix_nano,
            AgentStep::Subagent(s) => s.start_unix_nano,
        }
    }

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
    pub model: String,
    pub start_unix_nano: i64,
    pub end_unix_nano: i64,
    pub ttft_nanos: Option<i64>,
    /// Attempts the call took, when the dialect reports retries.
    pub attempts: u32,
    pub is_error: bool,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cost_usd: f64,
}

/// One tool invocation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub start_unix_nano: i64,
    pub end_unix_nano: i64,
    /// Time blocked on the user for permission, taken from the
    /// dialect's wait spans when spans were supplied. Zero means no
    /// wait was observed, and the coverage line says whether spans
    /// were there to observe one.
    pub wait_nanos: i64,
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
    pub agent_id: String,
    pub start_unix_nano: i64,
    pub end_unix_nano: i64,
    pub steps: Vec<AgentStep>,
}
