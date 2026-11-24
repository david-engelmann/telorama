//! Span dialects: the OpenTelemetry GenAI semantic conventions and
//! OpenInference, mapped from spans alone into the canonical agent
//! run model.
//!
//! Both conventions describe an agent run as a span tree, so one
//! tree walk serves both; the dialects differ only in how a span is
//! classified and where its facts live. Agent spans and workflow
//! spans are structure, model and tool spans are steps, and every
//! other span is a transparent container whose children still walk.
//!
//! The GenAI conventions churn: the provider attribute was renamed
//! from gen_ai.system to gen_ai.provider.name in v1.37, and later
//! generations added cache token attributes and a response model.
//! The adapter reads the union of the 1.30, 1.37, and 1.41 shapes,
//! so a span from any of those generations maps the same way. The
//! conventions have since moved to their own repository, which
//! has cut no versioned conventions release, so the adapter pins
//! to that repository at commit
//! ae8214a8f3a98786071736855093b74621174dc4 (2026-11-23), read
//! against the released 1.41 text, plus the deltas the Round 3
//! currency pass verified from the current sources: cache
//! creation's new spelling gen_ai.usage.cache_write.input_tokens,
//! the invoke_workflow operation, and the generate_content
//! operation.
//!
//! Token accounting follows the house rule: reported input counts
//! every input token including cached ones, so the model stores
//! non-cache input (input minus cache read). Neither dialect
//! exports cost, so calls are priced from the renderer's table and
//! the run carries the priced and unpriced split, as with Codex.
//!
//! Session identity comes from the dialect's conversation
//! attribute when spans carry one. Without one, a single trace keys
//! the run by its trace id; several traces with no shared id are
//! refused rather than guessed into one session.

use std::collections::{BTreeMap, BTreeSet};

use telorama_scene::{
    AgentRun, AgentStep, AgentTurn, Coverage, MAX_AGENT_NESTING, ModelCall, Pricing, Span,
    SubagentRun, ToolCall,
};

use crate::IngestError;
use crate::pricing::{PRICE_TABLE_AS_OF, price_call};

fn fail(message: impl Into<String>) -> IngestError {
    IngestError {
        message: message.into(),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    Model,
    Tool,
    Agent,
    Container,
}

#[derive(Default)]
struct Tokens {
    gross_input: u64,
    output: u64,
    cache_read: u64,
    cache_creation: u64,
}

struct Dialect {
    name: &'static str,
    session_attr: &'static str,
    classify: fn(&Span) -> Role,
    model: fn(&Span) -> String,
    tokens: fn(&Span) -> Tokens,
    tool_name: fn(&Span) -> String,
    agent_id: fn(&Span) -> String,
}

fn attr<'a>(span: &'a Span, key: &str) -> Option<&'a str> {
    span.attributes.get(key).map(String::as_str)
}

fn attr_u64(span: &Span, keys: &[&str]) -> u64 {
    keys.iter()
        .find_map(|k| attr(span, k))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

fn genai_classify(span: &Span) -> Role {
    match attr(span, "gen_ai.operation.name") {
        Some("chat" | "text_completion" | "embeddings" | "generate_content") => Role::Model,
        Some("execute_tool" | "retrieval") => Role::Tool,
        Some("invoke_agent" | "create_agent" | "invoke_workflow") => Role::Agent,
        _ => Role::Container,
    }
}

fn genai_model(span: &Span) -> String {
    attr(span, "gen_ai.response.model")
        .or_else(|| attr(span, "gen_ai.request.model"))
        .unwrap_or("unknown")
        .to_string()
}

fn genai_tokens(span: &Span) -> Tokens {
    Tokens {
        gross_input: attr_u64(span, &["gen_ai.usage.input_tokens"]),
        output: attr_u64(span, &["gen_ai.usage.output_tokens"]),
        cache_read: attr_u64(
            span,
            &[
                "gen_ai.usage.cache_read.input_tokens",
                "gen_ai.usage.cache_read_tokens",
            ],
        ),
        cache_creation: attr_u64(
            span,
            &[
                "gen_ai.usage.cache_write.input_tokens",
                "gen_ai.usage.cache_creation.input_tokens",
                "gen_ai.usage.cache_creation_tokens",
            ],
        ),
    }
}

fn genai_tool_name(span: &Span) -> String {
    attr(span, "gen_ai.tool.name")
        .unwrap_or(&span.name)
        .to_string()
}

fn genai_agent_id(span: &Span) -> String {
    attr(span, "gen_ai.agent.id")
        .unwrap_or(&span.span_id)
        .to_string()
}

fn oi_classify(span: &Span) -> Role {
    match attr(span, "openinference.span.kind") {
        Some("LLM" | "EMBEDDING") => Role::Model,
        Some("TOOL" | "RETRIEVER" | "RERANKER") => Role::Tool,
        Some("AGENT") => Role::Agent,
        _ => Role::Container,
    }
}

fn oi_model(span: &Span) -> String {
    attr(span, "llm.model_name")
        .unwrap_or("unknown")
        .to_string()
}

fn oi_tokens(span: &Span) -> Tokens {
    Tokens {
        gross_input: attr_u64(span, &["llm.token_count.prompt"]),
        output: attr_u64(span, &["llm.token_count.completion"]),
        cache_read: attr_u64(span, &["llm.token_count.prompt_details.cache_read"]),
        cache_creation: 0,
    }
}

fn oi_tool_name(span: &Span) -> String {
    attr(span, "tool.name").unwrap_or(&span.name).to_string()
}

fn oi_agent_id(span: &Span) -> String {
    span.span_id.clone()
}

const GENAI: Dialect = Dialect {
    name: "genai",
    session_attr: "gen_ai.conversation.id",
    classify: genai_classify,
    model: genai_model,
    tokens: genai_tokens,
    tool_name: genai_tool_name,
    agent_id: genai_agent_id,
};

const OPENINFERENCE: Dialect = Dialect {
    name: "openinference",
    session_attr: "session.id",
    classify: oi_classify,
    model: oi_model,
    tokens: oi_tokens,
    tool_name: oi_tool_name,
    agent_id: oi_agent_id,
};

/// Adapt GenAI semantic-convention spans into a run.
pub fn parse_genai_session(spans: &[Span]) -> Result<AgentRun, IngestError> {
    build_run(spans, &GENAI)
}

/// Adapt OpenInference spans into a run.
pub fn parse_openinference_session(spans: &[Span]) -> Result<AgentRun, IngestError> {
    build_run(spans, &OPENINFERENCE)
}

fn build_run(spans: &[Span], dialect: &Dialect) -> Result<AgentRun, IngestError> {
    if spans.is_empty() {
        return Err(fail("no spans: nothing to build a run from"));
    }
    let ids: BTreeSet<&str> = spans
        .iter()
        .filter_map(|s| attr(s, dialect.session_attr))
        .collect();
    let session_id = match ids.len() {
        1 => ids.iter().next().expect("one id").to_string(),
        n if n > 1 => {
            return Err(fail(format!(
                "the export carries {n} sessions: split it and adapt one session at a time"
            )));
        }
        _ => {
            let traces: BTreeSet<&str> = spans.iter().map(|s| s.trace_id.as_str()).collect();
            if traces.len() == 1 {
                traces.iter().next().expect("one trace").to_string()
            } else {
                return Err(fail(format!(
                    "spans carry no {}: cannot tell whether the {} traces form one session",
                    dialect.session_attr,
                    traces.len()
                )));
            }
        }
    };
    let mut by_trace: BTreeMap<&str, Vec<&Span>> = BTreeMap::new();
    for span in spans {
        by_trace
            .entry(span.trace_id.as_str())
            .or_default()
            .push(span);
    }
    let mut priced: BTreeSet<String> = BTreeSet::new();
    let mut unpriced: BTreeSet<String> = BTreeSet::new();
    let mut turns = Vec::new();
    for trace_spans in by_trace.values() {
        let in_trace: BTreeSet<&str> = trace_spans.iter().map(|s| s.span_id.as_str()).collect();
        let mut children: BTreeMap<&str, Vec<&Span>> = BTreeMap::new();
        let mut roots: Vec<&Span> = Vec::new();
        for span in trace_spans {
            match span.parent_span_id.as_deref() {
                Some(parent) if in_trace.contains(parent) => {
                    children.entry(parent).or_default().push(span);
                }
                _ => roots.push(span),
            }
        }
        let by_start = |a: &&Span, b: &&Span| {
            a.start_unix_nano
                .cmp(&b.start_unix_nano)
                .then_with(|| a.span_id.cmp(&b.span_id))
        };
        for list in children.values_mut() {
            list.sort_by(by_start);
        }
        roots.sort_by(by_start);
        for root in roots {
            turns.push(build_turn(
                root,
                &children,
                dialect,
                &mut priced,
                &mut unpriced,
            )?);
        }
    }
    turns.sort_by_key(|a| a.start_unix_nano);
    Ok(AgentRun {
        session_id,
        dialect: dialect.name.to_string(),
        turns,
        coverage: Coverage {
            log_events: 0,
            spans_seen: spans.len() as u64,
            spans_merged: spans.len() as u64,
        },
        pricing: Some(Pricing {
            table_as_of: PRICE_TABLE_AS_OF.to_string(),
            priced_models: priced.into_iter().collect(),
            unpriced_models: unpriced.into_iter().collect(),
        }),
    })
}

fn build_turn(
    root: &Span,
    children: &BTreeMap<&str, Vec<&Span>>,
    dialect: &Dialect,
    priced: &mut BTreeSet<String>,
    unpriced: &mut BTreeSet<String>,
) -> Result<AgentTurn, IngestError> {
    let mut steps = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    match (dialect.classify)(root) {
        Role::Agent | Role::Container => {
            // The root itself never emits as a step, so seed the
            // set with its id and a cycle back to it stops too.
            seen.insert(root.span_id.clone());
            for child in children_of(children, root) {
                emit(
                    child, children, dialect, priced, unpriced, &mut seen, &mut steps,
                )?;
            }
        }
        _ => emit(
            root, children, dialect, priced, unpriced, &mut seen, &mut steps,
        )?,
    }
    let span_count = subtree_len(root, children);
    let mut turn = AgentTurn {
        prompt_id: root.span_id.clone(),
        start_unix_nano: root.start_unix_nano,
        end_unix_nano: root.end_unix_nano,
        steps,
        input_tokens: 0,
        output_tokens: 0,
        cache_read_tokens: 0,
        cache_creation_tokens: 0,
        cost_usd: 0.0,
        coverage: Coverage {
            log_events: 0,
            spans_seen: span_count,
            spans_merged: span_count,
        },
    };
    let steps = std::mem::take(&mut turn.steps);
    AgentStep::walk(&steps, &mut |step| add_step_totals(&mut turn, step));
    turn.steps = steps;
    Ok(turn)
}

/// Turn totals sum the turn's model calls, subagents included: the
/// same rule the log dialects follow. The walk is the shared step
/// traversal, so this census cannot diverge from the others.
fn add_step_totals(turn: &mut AgentTurn, step: &AgentStep) {
    if let AgentStep::ModelCall(call) = step {
        turn.input_tokens = turn.input_tokens.saturating_add(call.input_tokens);
        turn.output_tokens = turn.output_tokens.saturating_add(call.output_tokens);
        turn.cache_read_tokens = turn
            .cache_read_tokens
            .saturating_add(call.cache_read_tokens);
        turn.cache_creation_tokens = turn
            .cache_creation_tokens
            .saturating_add(call.cache_creation_tokens);
        turn.cost_usd += call.cost_usd;
    }
}

fn children_of<'a>(children: &'a BTreeMap<&str, Vec<&'a Span>>, span: &Span) -> &'a [&'a Span] {
    children
        .get(span.span_id.as_str())
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

/// How many spans a root's subtree carries, the root included. The
/// span dialects read and merge every span in the trace, so a turn's
/// coverage is its subtree size. Each span id counts once, so a
/// cycle through repeated ids terminates (INV-82).
fn subtree_len(root: &Span, children: &BTreeMap<&str, Vec<&Span>>) -> u64 {
    let mut count = 0u64;
    let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    let mut stack: Vec<&Span> = vec![root];
    while let Some(span) = stack.pop() {
        if !seen.insert(span.span_id.as_str()) {
            continue;
        }
        count += 1;
        if let Some(kids) = children.get(span.span_id.as_str()) {
            stack.extend(kids.iter());
        }
    }
    count
}

fn emit(
    span: &Span,
    children: &BTreeMap<&str, Vec<&Span>>,
    dialect: &Dialect,
    priced: &mut BTreeSet<String>,
    unpriced: &mut BTreeSet<String>,
    seen: &mut BTreeSet<String>,
    sink: &mut Vec<AgentStep>,
) -> Result<(), IngestError> {
    // The walk runs on an explicit stack (INV-87). Each frame
    // holds the steps its subtree emitted. A container frame
    // splices them into its parent on completion, an agent
    // frame wraps them in its subagent step. The seen rule
    // fires when a frame opens or a leaf is reached, the
    // points the recursive walk tested it.
    struct Frame<'a> {
        span: &'a Span,
        kids: &'a [&'a Span],
        next: usize,
        agent_depth: usize,
        steps: Vec<AgentStep>,
    }

    fn open<'a>(
        span: &'a Span,
        children: &'a BTreeMap<&str, Vec<&'a Span>>,
        agent_depth: usize,
        seen: &mut BTreeSet<String>,
    ) -> Option<Frame<'a>> {
        // Each span id emits once per turn, so a cycle through
        // repeated ids stops instead of walking forever (INV-82).
        if !seen.insert(span.span_id.clone()) {
            return None;
        }
        Some(Frame {
            span,
            kids: children_of(children, span),
            next: 0,
            agent_depth,
            steps: Vec::new(),
        })
    }

    let mut stack: Vec<Frame> = Vec::new();
    match (dialect.classify)(span) {
        Role::Model => {
            if seen.insert(span.span_id.clone()) {
                sink.push(AgentStep::ModelCall(model_call(
                    span, dialect, priced, unpriced,
                )));
            }
            return Ok(());
        }
        Role::Tool => {
            if seen.insert(span.span_id.clone()) {
                sink.push(AgentStep::ToolCall(ToolCall {
                    name: (dialect.tool_name)(span),
                    start_unix_nano: span.start_unix_nano,
                    end_unix_nano: span.end_unix_nano,
                    wait_nanos: 0,
                    is_error: span.is_error,
                    decision: None,
                }));
            }
            return Ok(());
        }
        Role::Container | Role::Agent => {
            if let Some(frame) = open(span, children, 0, seen) {
                stack.push(frame);
            }
        }
    }
    while !stack.is_empty() {
        let (child, child_depth) = {
            let top = stack.last_mut().expect("frames under construction");
            if top.next >= top.kids.len() {
                (None, 0)
            } else {
                let child = top.kids[top.next];
                top.next += 1;
                let depth = top.agent_depth
                    + usize::from(matches!((dialect.classify)(top.span), Role::Agent));
                (Some(child), depth)
            }
        };
        match child {
            Some(child) => match (dialect.classify)(child) {
                Role::Model => {
                    if seen.insert(child.span_id.clone()) {
                        let call = model_call(child, dialect, priced, unpriced);
                        stack
                            .last_mut()
                            .expect("frames under construction")
                            .steps
                            .push(AgentStep::ModelCall(call));
                    }
                }
                Role::Tool => {
                    if seen.insert(child.span_id.clone()) {
                        let step = AgentStep::ToolCall(ToolCall {
                            name: (dialect.tool_name)(child),
                            start_unix_nano: child.start_unix_nano,
                            end_unix_nano: child.end_unix_nano,
                            wait_nanos: 0,
                            is_error: child.is_error,
                            decision: None,
                        });
                        stack
                            .last_mut()
                            .expect("frames under construction")
                            .steps
                            .push(step);
                    }
                }
                Role::Container | Role::Agent => {
                    if matches!((dialect.classify)(child), Role::Agent)
                        && !seen.contains(child.span_id.as_str())
                        && child_depth + 1 > MAX_AGENT_NESTING
                    {
                        return Err(fail(format!(
                            "session nests agents deeper than the {MAX_AGENT_NESTING} level budget"
                        )));
                    }
                    if let Some(frame) = open(child, children, child_depth, seen) {
                        stack.push(frame);
                    }
                }
            },
            None => {
                let mut frame = stack.pop().expect("frames under construction");
                match (dialect.classify)(frame.span) {
                    Role::Agent => {
                        let step = AgentStep::Subagent(SubagentRun {
                            agent_id: (dialect.agent_id)(frame.span),
                            start_unix_nano: frame.span.start_unix_nano,
                            end_unix_nano: frame.span.end_unix_nano,
                            steps: frame.steps,
                        });
                        match stack.last_mut() {
                            Some(parent) => parent.steps.push(step),
                            None => sink.push(step),
                        }
                    }
                    _ => match stack.last_mut() {
                        Some(parent) => parent.steps.append(&mut frame.steps),
                        None => sink.append(&mut frame.steps),
                    },
                }
            }
        }
    }
    Ok(())
}

fn model_call(
    span: &Span,
    dialect: &Dialect,
    priced: &mut BTreeSet<String>,
    unpriced: &mut BTreeSet<String>,
) -> ModelCall {
    let tokens = (dialect.tokens)(span);
    let cache_read = tokens.cache_read.min(tokens.gross_input);
    let mut call = ModelCall {
        model: (dialect.model)(span),
        start_unix_nano: span.start_unix_nano,
        end_unix_nano: span.end_unix_nano,
        ttft_nanos: None,
        attempts: 1,
        is_error: span.is_error,
        input_tokens: tokens.gross_input - cache_read,
        output_tokens: tokens.output,
        cache_read_tokens: cache_read,
        cache_creation_tokens: tokens.cache_creation,
        cost_usd: 0.0,
    };
    match price_call(&call) {
        Some(cost) => {
            call.cost_usd = cost;
            priced.insert(call.model.clone());
        }
        None => {
            unpriced.insert(call.model.clone());
        }
    }
    call
}
