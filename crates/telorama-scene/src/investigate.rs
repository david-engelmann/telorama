//! The investigation layer. One pure derivation ranks where an
//! investigator should look first, from the same derived truths the
//! panels print, so the human reading the render and the agent reading
//! the payload receive the same ranking with the same evidence.
//!
//! A finding states evidence and never prescribes a remedy. Every
//! finding carries at least one exemplar that exists in the input
//! data, and a candidate with no exemplar is not emitted. At most one
//! finding per kind is emitted, in the fixed kind order errors,
//! latency, wait, cost, so the list is short enough to act on.

use crate::agent::{AgentRun, AgentStep};
use crate::derive::{critical_path, red_by_service, waterfall_rows};
use crate::field::{format_duration_nanos, Thresholds};
use crate::trace::Span;
use serde::Serialize;
use std::collections::BTreeSet;

/// One evidence number behind a finding's statement, carried as
/// data so the agent reader consumes the same evidence the human
/// reads in the statement.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EvidencePoint {
    /// The measure's name, such as `error_ratio`.
    pub label: String,
    /// The measured value, formatted by the field formatters.
    pub value: String,
}

fn ev(label: &str, value: String) -> EvidencePoint {
    EvidencePoint {
        label: label.to_string(),
        value,
    }
}

/// One ranked investigation marker.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Finding {
    /// One-based position in the ranked list.
    pub rank: usize,
    /// One of `errors`, `latency`, `wait`, `cost`.
    pub kind: String,
    /// What the finding is about. A service, a span, a turn, or the
    /// session, depending on the kind.
    pub subject: String,
    /// The evidence stated in one line, generated from measured
    /// values only.
    pub statement: String,
    /// The evidence numbers behind the statement, as data.
    pub evidence: Vec<EvidencePoint>,
    /// Span ids, trace ids, or prompt ids that exist in the input
    /// data and prove the statement.
    pub exemplars: Vec<String>,
}

fn pct(share: f64) -> String {
    format!("{:.1}%", share * 100.0)
}

fn errors_finding(spans: &[Span], window_seconds: f64) -> Option<Finding> {
    let reds = red_by_service(spans, window_seconds);
    let total_errors: u64 = reds.iter().map(|r| r.errors).sum();
    if total_errors == 0 {
        return None;
    }
    let mut leaders: Vec<_> = reds.iter().filter(|r| r.errors > 0).collect();
    leaders.sort_by(|a, b| {
        b.errors
            .cmp(&a.errors)
            .then_with(|| b.error_ratio.partial_cmp(&a.error_ratio).unwrap())
            .then_with(|| a.service.cmp(&b.service))
    });
    let leader = leaders[0];
    let mut error_spans: Vec<&Span> = spans
        .iter()
        .filter(|s| s.service == leader.service && s.is_error)
        .collect();
    if error_spans.is_empty() {
        return None;
    }
    error_spans.sort_by(|a, b| {
        a.start_unix_nano
            .cmp(&b.start_unix_nano)
            .then_with(|| a.span_id.cmp(&b.span_id))
    });
    let exemplar = error_spans[0];
    let threshold = Thresholds::error_ratio().steps[0].above;
    let mut statement = format!(
        "{} carries {} of the errors in the window, {} of {} errors across {} calls at {} errors",
        leader.service,
        pct(leader.errors as f64 / total_errors as f64),
        leader.errors,
        total_errors,
        leader.calls,
        pct(leader.error_ratio),
    );
    let mut evidence = vec![
        ev("service_errors", leader.errors.to_string()),
        ev("service_calls", leader.calls.to_string()),
        ev("error_ratio", pct(leader.error_ratio)),
        ev("window_errors", total_errors.to_string()),
    ];
    if leader.error_ratio >= threshold {
        statement.push_str(", above the standing 5% verdict threshold");
        evidence.push(ev("verdict_threshold", pct(threshold)));
    }
    Some(Finding {
        rank: 0,
        kind: "errors".to_string(),
        subject: leader.service.clone(),
        statement,
        evidence,
        exemplars: vec![exemplar.span_id.clone(), exemplar.trace_id.clone()],
    })
}

fn latency_finding(spans: &[Span]) -> Option<Finding> {
    if spans.is_empty() {
        return None;
    }
    let by_id: std::collections::BTreeMap<(&str, &str), &Span> = spans
        .iter()
        .map(|s| ((s.trace_id.as_str(), s.span_id.as_str()), s))
        .collect();
    let trace_ids: BTreeSet<&str> = spans.iter().map(|s| s.trace_id.as_str()).collect();
    let mut total_self: i64 = 0;
    let mut best: Option<(&Span, i64)> = None;
    for trace_id in &trace_ids {
        for row in waterfall_rows(spans, trace_id) {
            total_self += row.self_time_nanos;
            let span = by_id[&(*trace_id, row.span_id.as_str())];
            let better = match best {
                None => true,
                Some((best_span, best_self)) => {
                    row.self_time_nanos > best_self
                        || (row.self_time_nanos == best_self && span.span_id < best_span.span_id)
                }
            };
            if better {
                best = Some((span, row.self_time_nanos));
            }
        }
    }
    let (span, self_nanos) = best?;
    if total_self <= 0 || self_nanos <= 0 {
        return None;
    }
    let mut statement = format!(
        "{} in {} holds the largest self time at {} ({} of the window self time)",
        span.name,
        span.service,
        format_duration_nanos(self_nanos),
        pct(self_nanos as f64 / total_self as f64),
    );
    if critical_path(spans, &span.trace_id).contains(&span.span_id) {
        statement.push_str(", on the critical path of its trace");
    }
    Some(Finding {
        rank: 0,
        kind: "latency".to_string(),
        subject: span.name.clone(),
        statement,
        evidence: vec![
            ev("self_time", format_duration_nanos(self_nanos)),
            ev("window_self_time", format_duration_nanos(total_self)),
            ev("share", pct(self_nanos as f64 / total_self as f64)),
        ],
        exemplars: vec![span.span_id.clone(), span.trace_id.clone()],
    })
}

/// Every tool call in a run, subagents included, with the prompt id of
/// the turn it belongs to.
fn tool_calls(run: &AgentRun) -> Vec<(String, String, i64)> {
    let mut out = Vec::new();
    for turn in &run.turns {
        AgentStep::walk(&turn.steps, &mut |step| {
            if let AgentStep::ToolCall(tool) = step {
                out.push((turn.prompt_id.clone(), tool.name.clone(), tool.wait_nanos));
            }
        });
    }
    out
}

fn wait_finding(run: &AgentRun) -> Option<Finding> {
    let calls = tool_calls(run);
    let total_wait: i64 = calls.iter().map(|c| c.2).sum();
    if total_wait <= 0 || run.turns.is_empty() {
        return None;
    }
    let wall = run.turns.iter().map(|t| t.end_unix_nano).max().unwrap_or(0)
        - run
            .turns
            .iter()
            .map(|t| t.start_unix_nano)
            .min()
            .unwrap_or(0);
    if wall <= 0 {
        return None;
    }
    let (prompt_id, tool, longest) = calls
        .iter()
        .filter(|c| c.2 > 0)
        .max_by(|a, b| a.2.cmp(&b.2).then_with(|| b.1.cmp(&a.1)))
        .expect("a positive total wait means a positive call");
    Some(Finding {
        rank: 0,
        kind: "wait".to_string(),
        subject: tool.clone(),
        statement: format!(
            "The agent spent {} waiting on the user ({} of the session), longest {} on {} in one turn",
            format_duration_nanos(total_wait),
            pct(total_wait as f64 / wall as f64),
            format_duration_nanos(*longest),
            tool,
        ),
        evidence: vec![
            ev("total_wait", format_duration_nanos(total_wait)),
            ev("longest_wait", format_duration_nanos(*longest)),
            ev("session_wall", format_duration_nanos(wall)),
            ev("share", pct(total_wait as f64 / wall as f64)),
        ],
        exemplars: vec![prompt_id.clone()],
    })
}

fn cost_finding(run: &AgentRun) -> Option<Finding> {
    if run.turns.is_empty() {
        return None;
    }
    let total: f64 = run.turns.iter().map(|t| t.cost_usd).sum();
    let unpriced: &[String] = run
        .pricing
        .as_ref()
        .map(|p| p.unpriced_models.as_slice())
        .unwrap_or(&[]);
    if total <= 0.0 && unpriced.is_empty() {
        return None;
    }
    if total <= 0.0 {
        return Some(Finding {
            rank: 0,
            kind: "cost".to_string(),
            subject: "session".to_string(),
            statement: format!(
                "No priced cost in this session. Unpriced models in use, {}",
                unpriced.join(", "),
            ),
            evidence: vec![ev("unpriced_models", unpriced.join(", "))],
            exemplars: vec![run.turns[0].prompt_id.clone()],
        });
    }
    let leader = run
        .turns
        .iter()
        .max_by(|a, b| {
            a.cost_usd
                .partial_cmp(&b.cost_usd)
                .unwrap()
                .then_with(|| b.prompt_id.cmp(&a.prompt_id))
        })
        .expect("a non-empty run has a costliest turn");
    let mut models: BTreeSet<String> = BTreeSet::new();
    AgentStep::walk(&leader.steps, &mut |step| {
        if let AgentStep::ModelCall(call) = step {
            models.insert(call.model.clone());
        }
    });
    let models: Vec<String> = models.into_iter().collect();
    let mut statement = format!(
        "The most expensive turn cost ${:.4} ({} of the session priced cost ${:.4})",
        leader.cost_usd,
        pct(leader.cost_usd / total),
        total,
    );
    if !models.is_empty() {
        statement.push_str(&format!(" on {}", models.join(", ")));
    }
    let mut evidence = vec![
        ev("turn_cost_usd", format!("${:.4}", leader.cost_usd)),
        ev("session_cost_usd", format!("${:.4}", total)),
        ev("share", pct(leader.cost_usd / total)),
    ];
    if !models.is_empty() {
        evidence.push(ev("model", models.join(", ")));
    }
    if !unpriced.is_empty() {
        statement.push_str(&format!(
            ". Priced cost excludes unpriced models, {}",
            unpriced.join(", ")
        ));
        evidence.push(ev("unpriced_models", unpriced.join(", ")));
    }
    Some(Finding {
        rank: 0,
        kind: "cost".to_string(),
        subject: leader.prompt_id.clone(),
        statement,
        evidence,
        exemplars: vec![leader.prompt_id.clone()],
    })
}

/// Rank where an investigator should look first over a span set and
/// an optional attached run. Pure and deterministic. Empty input
/// yields an empty list, never an error.
pub fn investigate(spans: &[Span], run: Option<&AgentRun>, window_seconds: f64) -> Vec<Finding> {
    let mut out = Vec::new();
    if let Some(f) = errors_finding(spans, window_seconds) {
        out.push(f);
    }
    if let Some(f) = latency_finding(spans) {
        out.push(f);
    }
    if let Some(run) = run {
        if let Some(f) = wait_finding(run) {
            out.push(f);
        }
        if let Some(f) = cost_finding(run) {
            out.push(f);
        }
    }
    for (i, finding) in out.iter_mut().enumerate() {
        finding.rank = i + 1;
    }
    out
}
