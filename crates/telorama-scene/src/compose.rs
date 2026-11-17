//! Incident composition: the parameterised incident template.
//!
//! Given the spans of a window (and optionally the agent run that
//! overlaps it), compose the scene an investigator needs first: a
//! service map, the RED row, a waterfall of one exemplar trace, and
//! a latency heatmap, with the headline and narrative derived from
//! the same numbers the panels print. Composition is a pure
//! function of the span set, so the same window composes the same
//! scene, byte for byte.
//!
//! The choice rules are fixed and stated here, because a template
//! that picks quietly is a claim nobody can check. The failing
//! service is the one with the highest error ratio among services
//! that recorded an error; ties break on more errors, then on the
//! service name. The exemplar trace is the longest trace, by root
//! span duration, among traces that contain an error; when no trace
//! errors, it is the longest trace in the window. Ties break on the
//! trace id. The heatmap's bucket edges are part of the template
//! and are declared on the data entry like any other scene.

use std::collections::BTreeMap;

use crate::agent::AgentRun;
use crate::derive::red_by_service;
use crate::field::Thresholds;
use crate::spec::{
    Context, DataRef, NarrativeItem, Panel, Provenance, Sampling, Scene, Source, TimeRange, Unit,
    SCENE_VERSION,
};
use crate::trace::Span;

/// The template's declared histogram edges, in seconds.
pub const INCIDENT_BUCKETS: [f64; 7] = [0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5];

/// Compose the incident scene for a window of spans. `run` attaches
/// the overlapping agent session as the last panel when present.
pub fn incident_scene(
    spans: &[Span],
    run: Option<&AgentRun>,
    generated_by: &str,
) -> Result<Scene, String> {
    if spans.is_empty() {
        return Err("no spans: nothing to compose an incident from".to_string());
    }
    let from_ms = spans.iter().map(|s| s.start_unix_nano).min().expect("spans") / 1_000_000;
    let max_end = spans.iter().map(|s| s.end_unix_nano).max().expect("spans");
    let to_ms = ((max_end + 999_999) / 1_000_000).max(from_ms + 1);
    let range = TimeRange {
        from_unix_ms: from_ms,
        to_unix_ms: to_ms,
    };
    let red = red_by_service(spans, range.seconds());
    let worst = red
        .iter()
        .filter(|r| r.errors > 0)
        .max_by(|a, b| {
            a.error_ratio
                .partial_cmp(&b.error_ratio)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.errors.cmp(&b.errors))
                .then_with(|| b.service.cmp(&a.service))
        });
    let exemplar = exemplar_trace(spans, true)
        .or_else(|| exemplar_trace(spans, false))
        .expect("a non-empty span set has an exemplar trace");

    let (title, headline, red_note) = match worst {
        Some(w) => (
            format!("Incident: {} errors", w.service),
            format!(
                "{} is failing at {:.1}% errors across {} {} in the window",
                w.service,
                w.error_ratio * 100.0,
                w.calls,
                plural(w.calls, "call", "calls")
            ),
            format!(
                "{} carries the worst error ratio at {:.1}% over {} {}.",
                w.service,
                w.error_ratio * 100.0,
                w.calls,
                plural(w.calls, "call", "calls")
            ),
        ),
        None => (
            "Incident: no failing service".to_string(),
            "No service is erroring in the window; the slowest trace carries the scene"
                .to_string(),
            "No service recorded an error in the window; the RED row stands as the evidence."
                .to_string(),
        ),
    };
    let waterfall_note = if worst.is_some() {
        format!(
            "The exemplar trace {exemplar} is the longest trace with an error in the window."
        )
    } else {
        format!("The exemplar trace {exemplar} is the longest trace in the window.")
    };

    let mut data = vec![
        DataRef {
            id: "spans".to_string(),
            source: Source {
                kind: "otlp_file".to_string(),
                query: Some("all spans in the incident window".to_string()),
                dialect: None,
                trace_id: None,
                buckets: Some(INCIDENT_BUCKETS.to_vec()),
            },
            sampled: Sampling::Unsampled,
        },
        DataRef {
            id: "exemplar".to_string(),
            source: Source {
                kind: "otlp_file".to_string(),
                query: Some(format!("trace {exemplar}")),
                dialect: None,
                trace_id: Some(exemplar.clone()),
                buckets: None,
            },
            sampled: Sampling::Unsampled,
        },
    ];
    let mut panels = vec![
        Panel::ServiceMap {
            data: "spans".to_string(),
        },
        Panel::RedRow {
            data: "spans".to_string(),
            service: None,
            thresholds: Thresholds::default(),
        },
        Panel::Waterfall {
            data: "exemplar".to_string(),
            highlight_critical_path: true,
            highlight_errors: true,
        },
        Panel::Heatmap {
            data: "spans".to_string(),
            unit: Unit::Duration,
        },
    ];
    let mut narrative = vec![
        NarrativeItem {
            panel: 0,
            text: "Edges exist only where a span's parent belongs to another service in the same trace.".to_string(),
        },
        NarrativeItem {
            panel: 1,
            text: red_note,
        },
        NarrativeItem {
            panel: 2,
            text: waterfall_note,
        },
    ];
    if let Some(run) = run {
        data.push(DataRef {
            id: "session".to_string(),
            source: Source {
                kind: format!("{}_session", run.dialect),
                query: Some(format!(
                    "session {} overlapping the incident window",
                    run.session_id
                )),
                dialect: None,
                trace_id: None,
                buckets: None,
            },
            sampled: Sampling::Unsampled,
        });
        panels.push(Panel::AgentRun {
            data: "session".to_string(),
        });
        narrative.push(NarrativeItem {
            panel: 4,
            text: "The agent session overlapping this window is attached below the telemetry panels.".to_string(),
        });
    }

    Ok(Scene {
        scene: SCENE_VERSION.to_string(),
        title,
        headline,
        context: Context {
            range,
            filter: BTreeMap::new(),
            provenance: Provenance {
                generated_by: generated_by.to_string(),
                question: "What is failing, where, and how badly in this window?".to_string(),
            },
        },
        data,
        panels,
        narrative,
    })
}

/// The longest trace in the set by root span duration, optionally
/// restricted to traces containing an error. Roots are spans whose
/// parent is absent from the trace.
fn exemplar_trace(spans: &[Span], errors_only: bool) -> Option<String> {
    let mut by_trace: BTreeMap<&str, Vec<&Span>> = BTreeMap::new();
    for span in spans {
        by_trace.entry(span.trace_id.as_str()).or_default().push(span);
    }
    let mut best: Option<(&str, i64)> = None;
    for (trace_id, trace_spans) in &by_trace {
        if errors_only && !trace_spans.iter().any(|s| s.is_error) {
            continue;
        }
        let ids: std::collections::BTreeSet<&str> =
            trace_spans.iter().map(|s| s.span_id.as_str()).collect();
        let root_duration = trace_spans
            .iter()
            .filter(|s| match s.parent_span_id.as_deref() {
                Some(parent) => !ids.contains(parent),
                None => true,
            })
            .map(|s| s.duration_nanos())
            .max()
            .unwrap_or(0);
        let better = match best {
            None => true,
            Some((_, d)) => root_duration > d,
        };
        if better {
            best = Some((trace_id, root_duration));
        }
    }
    best.map(|(id, _)| id.to_string())
}

fn plural(n: u64, singular: &'static str, plural_form: &'static str) -> &'static str {
    if n == 1 { singular } else { plural_form }
}
