//! Session frames: the live milestone's unit of motion. A session
//! becomes a stream of complete scenes, one per turn boundary, where
//! frame k shows the session prefix through turn k and nothing else.
//!
//! Purity is the contract. A frame's run carries the prefix turns
//! verbatim, its coverage is the sum of the prefix turns' own
//! coverage (plus, for the log dialects, the supplied span count,
//! which names the input the export carried and does not change
//! under a turn truncation), and its pricing names only the models
//! the prefix actually used. Frame k is therefore a pure function of
//! the session prefix through turn k, and the truncation test proves
//! it: frames built from an export cut after turn 1 equal the leading
//! frames built from the whole export.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::agent::{AgentRun, Coverage, Pricing};
use crate::investigate::{Finding, investigate};
use crate::spec::{
    Context, DataRef, NarrativeItem, Panel, Provenance, SCENE_VERSION, Scene, Source, TimeRange,
};

/// One frame of a live session: a complete scene over the session
/// prefix, the prefix run it renders, and the findings the
/// investigation layer derives from that prefix alone.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Frame {
    /// Zero-based position in the frame stream.
    pub index: usize,
    /// How many turns the frame covers.
    pub turns: usize,
    pub scene: Scene,
    pub run: AgentRun,
    pub findings: Vec<Finding>,
}

/// Build the frame stream for a parsed session run. An empty run
/// yields an empty stream.
pub fn session_frames(run: &AgentRun, generated_by: &str) -> Vec<Frame> {
    let total = run.turns.len();
    let mut frames = Vec::with_capacity(total);
    for k in 1..=total {
        let prefix = prefix_run(run, k);
        let from_ms = run.turns[0].start_unix_nano / 1_000_000;
        let end_nano = run.turns[k - 1].end_unix_nano;
        let to_ms = ((end_nano + 999_999) / 1_000_000).max(from_ms + 1);
        let range = TimeRange {
            from_unix_ms: from_ms,
            to_unix_ms: to_ms,
        };
        let input: u64 = prefix.turns.iter().map(|t| t.input_tokens).sum();
        let output: u64 = prefix.turns.iter().map(|t| t.output_tokens).sum();
        let cost: f64 = prefix.turns.iter().map(|t| t.cost_usd).sum();
        let scene = Scene {
            scene: SCENE_VERSION.to_string(),
            title: format!("Session {}", run.session_id),
            headline: format!(
                "After turn {k} of {total}: {input} input tokens, {output} output tokens, ${cost:.4} so far"
            ),
            context: Context {
                range,
                filter: BTreeMap::new(),
                provenance: Provenance {
                    generated_by: generated_by.to_string(),
                    question: "What has the agent done so far, and what has it cost up to here?"
                        .to_string(),
                },
            },
            data: vec![DataRef {
                id: "session".to_string(),
                source: Source {
                    kind: format!("{}_session", run.dialect),
                    query: Some(format!(
                        "session {} prefix through turn {k}",
                        run.session_id
                    )),
                    dialect: None,
                    trace_id: None,
                    buckets: None,
                },
                sampled: "unsampled".to_string(),
            }],
            panels: vec![Panel::AgentRun {
                data: "session".to_string(),
            }],
            narrative: vec![NarrativeItem {
                text: format!("Frame {} covers the first {k} of {total} turns.", k),
                panel: 0,
            }],
        };
        let findings = investigate(&[], Some(&prefix), range.seconds());
        frames.push(Frame {
            index: k - 1,
            turns: k,
            scene,
            run: prefix,
            findings,
        });
    }
    frames
}

/// The run restricted to its first k turns, with coverage and
/// pricing restated over the prefix alone.
fn prefix_run(run: &AgentRun, k: usize) -> AgentRun {
    let turns: Vec<_> = run.turns.iter().take(k).cloned().collect();
    let log_events: u64 = turns.iter().map(|t| t.coverage.log_events).sum();
    let merged: u64 = turns.iter().map(|t| t.coverage.spans_merged).sum();
    // The span dialects count spans per turn exactly, so their prefix
    // coverage is the turn sum. The log dialects count the supplied
    // span input at session level; that statistic names the export
    // the frames were built from and does not change when the log
    // stream is cut at a turn boundary.
    let spans_seen = match run.dialect.as_str() {
        "genai" | "openinference" => turns.iter().map(|t| t.coverage.spans_seen).sum(),
        _ => run.coverage.spans_seen,
    };
    let pricing = run.pricing.as_ref().map(|p| {
        let mut models: Vec<&str> = Vec::new();
        for turn in &turns {
            collect_models(&turn.steps, &mut models);
        }
        Pricing {
            table_as_of: p.table_as_of.clone(),
            priced_models: p
                .priced_models
                .iter()
                .filter(|m| models.contains(&m.as_str()))
                .cloned()
                .collect(),
            unpriced_models: p
                .unpriced_models
                .iter()
                .filter(|m| models.contains(&m.as_str()))
                .cloned()
                .collect(),
        }
    });
    AgentRun {
        session_id: run.session_id.clone(),
        dialect: run.dialect.clone(),
        turns,
        coverage: Coverage {
            log_events,
            spans_seen,
            spans_merged: merged,
        },
        pricing,
    }
}

fn collect_models<'a>(steps: &'a [crate::agent::AgentStep], out: &mut Vec<&'a str>) {
    for step in steps {
        match step {
            crate::agent::AgentStep::ModelCall(call) => out.push(call.model.as_str()),
            crate::agent::AgentStep::Subagent(sub) => collect_models(&sub.steps, out),
            crate::agent::AgentStep::ToolCall(_) => {}
        }
    }
}
