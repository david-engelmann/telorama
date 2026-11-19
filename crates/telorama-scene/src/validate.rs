//! The honesty validator.
//!
//! Validation runs on the raw JSON value before typed deserialization, so
//! the honesty rules bind hand-written and agent-written specs alike, and
//! every rejection carries a specific, machine-actionable message an agent
//! can repair in one round trip.
//!
//! Rules, in the order they are checked:
//! - the scene version is the one this build supports
//! - title and headline are present and non-empty
//! - the context window is present and runs forward, and provenance names
//!   a generator and a question
//! - data ids are unique, every entry names a source kind and a query or
//!   a trace id, and sampling state is always stated
//! - every panel names a known type and reads a declared data entry
//! - duration statistics are percentiles or distributions, never an
//!   average or a mean
//! - stat and heatmap panels declare a known unit
//! - threshold steps on stat and red_row panels are strictly increasing
//!   in `above` and carry #RRGGBB hex colors
//! - sankey panels name an explicit residual bucket
//! - flame panels declare a sample type and a sample count at or above
//!   their stated floor
//! - heatmap data declares real, strictly increasing bucket edges
//! - narrative items anchor to panels that exist

use crate::spec::{SCENE_VERSION, Scene};
use serde_json::Value;
use std::collections::BTreeSet;
use std::fmt;

/// One validation failure, located by a JSON path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

impl std::error::Error for ValidationError {}

fn err(errors: &mut Vec<ValidationError>, path: &str, message: impl Into<String>) {
    errors.push(ValidationError {
        path: path.to_string(),
        message: message.into(),
    });
}

fn non_empty_str<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get(key).and_then(Value::as_str).filter(|s| !s.is_empty())
}

fn is_hex_color(s: &str) -> bool {
    s.len() == 7 && s.starts_with('#') && s[1..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Threshold steps, when a panel declares them, must be usable as a
/// verdict scale: numeric, strictly increasing, with real colors.
fn validate_thresholds(panel: &Value, base: &str, errors: &mut Vec<ValidationError>) {
    let Some(steps) = panel.get("thresholds").and_then(Value::as_array) else {
        return;
    };
    let mut prev: Option<f64> = None;
    for (j, step) in steps.iter().enumerate() {
        let spath = format!("{base}.thresholds[{j}]");
        match step.get("above").and_then(Value::as_f64) {
            Some(a) => {
                if let Some(p) = prev
                    && a <= p
                {
                    err(
                        errors,
                        &spath,
                        "threshold steps must be strictly increasing in above: a later step must sit above every earlier one",
                    );
                }
                prev = Some(a);
            }
            None => err(errors, &spath, "threshold step needs a numeric above"),
        }
        match step.get("color").and_then(Value::as_str) {
            Some(c) if is_hex_color(c) => {}
            _ => err(
                errors,
                &format!("{spath}.color"),
                "threshold color must be a #RRGGBB hex color",
            ),
        }
    }
}

const PANEL_TYPES: [&str; 9] = [
    "waterfall",
    "stat",
    "red_row",
    "sankey",
    "flame",
    "heatmap",
    "service_map",
    "agent_run",
    "agent_state",
];
const UNITS: [&str; 5] = ["duration", "count", "ratio", "requests_per_second", "bytes"];
const STATS: [&str; 7] = ["p50", "p95", "p99", "count", "rate", "error_ratio", "last"];
const SAMPLING: [&str; 3] = ["unsampled", "sampled", "unknown"];

/// Validate a scene spec in raw JSON form. An empty result means valid.
pub fn validate_scene_json(v: &Value) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let Some(obj) = v.as_object() else {
        err(&mut errors, "$", "scene must be a JSON object");
        return errors;
    };

    match obj.get("scene").and_then(Value::as_str) {
        Some(SCENE_VERSION) => {}
        Some(other) => err(
            &mut errors,
            "$.scene",
            format!("unsupported scene version {other:?}, this build reads {SCENE_VERSION:?}"),
        ),
        None => err(&mut errors, "$.scene", "missing scene version"),
    }

    if non_empty_str(v, "title").is_none() {
        err(&mut errors, "$.title", "title must be a non-empty string");
    }
    if non_empty_str(v, "headline").is_none() {
        err(
            &mut errors,
            "$.headline",
            "headline must be a non-empty string: an answer-shaped scene leads with its takeaway",
        );
    }

    // Context, window, provenance.
    let context = obj.get("context");
    match context.and_then(|c| c.get("range")) {
        Some(range) => {
            let from = range.get("from_unix_ms").and_then(Value::as_i64);
            let to = range.get("to_unix_ms").and_then(Value::as_i64);
            match (from, to) {
                (Some(f), Some(t)) if f < t => {}
                (Some(_), Some(_)) => err(
                    &mut errors,
                    "$.context.range",
                    "range must run forward: from_unix_ms must be less than to_unix_ms",
                ),
                _ => err(
                    &mut errors,
                    "$.context.range",
                    "range needs integer from_unix_ms and to_unix_ms",
                ),
            }
        }
        None => err(&mut errors, "$.context.range", "missing context range"),
    }
    match context.and_then(|c| c.get("provenance")) {
        Some(p) => {
            if non_empty_str(p, "generated_by").is_none() {
                err(
                    &mut errors,
                    "$.context.provenance.generated_by",
                    "provenance must name who generated the scene",
                );
            }
            if non_empty_str(p, "question").is_none() {
                err(
                    &mut errors,
                    "$.context.provenance.question",
                    "provenance must state the question the scene answers",
                );
            }
        }
        None => err(&mut errors, "$.context.provenance", "missing provenance"),
    }

    // Data entries.
    let mut data_ids: BTreeSet<String> = BTreeSet::new();
    match obj.get("data").and_then(Value::as_array) {
        Some(data) => {
            if data.is_empty() {
                err(
                    &mut errors,
                    "$.data",
                    "a scene needs at least one data entry",
                );
            }
            for (i, entry) in data.iter().enumerate() {
                let base = format!("$.data[{i}]");
                match non_empty_str(entry, "id") {
                    Some(id) => {
                        if !data_ids.insert(id.to_string()) {
                            err(&mut errors, &base, format!("duplicate data id {id:?}"));
                        }
                    }
                    None => err(&mut errors, &base, "data entry needs a non-empty id"),
                }
                match entry.get("source") {
                    Some(source) => {
                        if non_empty_str(source, "kind").is_none() {
                            err(
                                &mut errors,
                                &format!("{base}.source.kind"),
                                "source must name its kind",
                            );
                        }
                        let has_query = non_empty_str(source, "query").is_some();
                        let has_trace = non_empty_str(source, "trace_id").is_some();
                        if !has_query && !has_trace {
                            err(
                                &mut errors,
                                &format!("{base}.source"),
                                "source must carry the exact query or a trace id: a diagram without its query is not reproducible",
                            );
                        }
                    }
                    None => err(&mut errors, &base, "data entry needs a source"),
                }
                match entry.get("sampled").and_then(Value::as_str) {
                    Some(s) if SAMPLING.contains(&s) => {}
                    Some(s) => err(
                        &mut errors,
                        &format!("{base}.sampled"),
                        format!("unknown sampling state {s:?}, expected one of {SAMPLING:?}"),
                    ),
                    None => err(
                        &mut errors,
                        &format!("{base}.sampled"),
                        "sampling state must be stated: unsampled, sampled, or unknown",
                    ),
                }
            }
        }
        None => err(&mut errors, "$.data", "missing data entries"),
    }

    // Panels.
    let panels = obj.get("panels").and_then(Value::as_array);
    match panels {
        Some(list) => {
            if list.is_empty() {
                err(&mut errors, "$.panels", "a scene needs at least one panel");
            }
            for (i, panel) in list.iter().enumerate() {
                let base = format!("$.panels[{i}]");
                let ptype = panel.get("type").and_then(Value::as_str);
                match ptype {
                    Some(t) if PANEL_TYPES.contains(&t) => {}
                    Some(t) => {
                        err(&mut errors, &base, format!("unknown panel type {t:?}"));
                        continue;
                    }
                    None => {
                        err(&mut errors, &base, "panel needs a type");
                        continue;
                    }
                }
                let ptype = ptype.unwrap_or_default();
                match non_empty_str(panel, "data") {
                    Some(id) if data_ids.contains(id) => {}
                    Some(id) => err(
                        &mut errors,
                        &format!("{base}.data"),
                        format!("panel reads undeclared data id {id:?}"),
                    ),
                    None => err(
                        &mut errors,
                        &format!("{base}.data"),
                        "panel needs a data id",
                    ),
                }
                if matches!(ptype, "stat" | "red_row") {
                    validate_thresholds(panel, &base, &mut errors);
                }
                match ptype {
                    "stat" => {
                        let unit = panel.get("unit").and_then(Value::as_str);
                        match unit {
                            Some(u) if UNITS.contains(&u) => {}
                            Some(u) => err(
                                &mut errors,
                                &format!("{base}.unit"),
                                format!("unknown unit {u:?}"),
                            ),
                            None => err(
                                &mut errors,
                                &format!("{base}.unit"),
                                "stat panel must declare its unit",
                            ),
                        }
                        let stat = panel.get("stat").and_then(Value::as_str);
                        match stat {
                            Some("average") | Some("mean") if unit == Some("duration") => err(
                                &mut errors,
                                &format!("{base}.stat"),
                                "duration statistics are percentiles, never an average: use p50, p95, or p99",
                            ),
                            Some("average") | Some("mean") => err(
                                &mut errors,
                                &format!("{base}.stat"),
                                "average is not a scene statistic: use p50, p95, p99, count, rate, error_ratio, or last",
                            ),
                            Some(s) if STATS.contains(&s) => {}
                            Some(s) => err(
                                &mut errors,
                                &format!("{base}.stat"),
                                format!("unknown statistic {s:?}"),
                            ),
                            None => err(
                                &mut errors,
                                &format!("{base}.stat"),
                                "stat panel must name its statistic",
                            ),
                        }
                    }
                    "sankey" => {
                        if non_empty_str(panel, "residual").is_none() {
                            err(
                                &mut errors,
                                &format!("{base}.residual"),
                                "sankey flow must conserve: name an explicit residual bucket for whatever does not balance",
                            );
                        }
                    }
                    "flame" => {
                        if non_empty_str(panel, "sample_type").is_none() {
                            err(
                                &mut errors,
                                &format!("{base}.sample_type"),
                                "flame panel must declare its sample type",
                            );
                        }
                        let floor = panel.get("min_samples").and_then(Value::as_u64);
                        let count = panel.get("sample_count").and_then(Value::as_u64);
                        match (floor, count) {
                            (Some(f), Some(c)) if f >= 1 && c >= f => {}
                            (Some(f), Some(c)) if f >= 1 => err(
                                &mut errors,
                                &format!("{base}.sample_count"),
                                format!(
                                    "flame graph has {c} samples, below its stated floor of {f}: refusing to render an empty flame as no cost"
                                ),
                            ),
                            _ => err(
                                &mut errors,
                                &base.to_string(),
                                "flame panel must declare min_samples (at least 1) and sample_count",
                            ),
                        }
                    }
                    "heatmap" => {
                        let unit = panel.get("unit").and_then(Value::as_str);
                        if !matches!(unit, Some(u) if UNITS.contains(&u)) {
                            err(
                                &mut errors,
                                &format!("{base}.unit"),
                                "heatmap panel must declare a known unit",
                            );
                        }
                        // Bucket edges live on the referenced data entry.
                        if let Some(id) = non_empty_str(panel, "data") {
                            let entry = obj.get("data").and_then(Value::as_array).and_then(|d| {
                                d.iter().find(|e| non_empty_str(e, "id") == Some(id))
                            });
                            let buckets = entry
                                .and_then(|e| e.get("source"))
                                .and_then(|s| s.get("buckets"))
                                .and_then(Value::as_array);
                            match buckets {
                                Some(b) if b.len() >= 2 => {
                                    let edges: Vec<f64> =
                                        b.iter().filter_map(Value::as_f64).collect();
                                    if edges.len() != b.len()
                                        || !edges.windows(2).all(|w| w[0] < w[1])
                                    {
                                        err(
                                            &mut errors,
                                            &format!("{base}.data"),
                                            "heatmap bucket edges must be numbers in strictly increasing order: bucket boundaries are data, not style",
                                        );
                                    }
                                }
                                _ => err(
                                    &mut errors,
                                    &format!("{base}.data"),
                                    "heatmap data must declare its real histogram bucket edges in source.buckets: no interpolated surfaces",
                                ),
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        None => err(&mut errors, "$.panels", "missing panels"),
    }

    // Narrative anchors.
    if let Some(items) = obj.get("narrative").and_then(Value::as_array) {
        let panel_count = panels.map_or(0, |p| p.len());
        for (i, item) in items.iter().enumerate() {
            let base = format!("$.narrative[{i}]");
            if non_empty_str(item, "text").is_none() {
                err(&mut errors, &base, "narrative item needs text");
            }
            match item.get("panel").and_then(Value::as_u64) {
                Some(p) if (p as usize) < panel_count => {}
                _ => err(
                    &mut errors,
                    &format!("{base}.panel"),
                    "narrative item must anchor to a panel that exists",
                ),
            }
        }
    }

    errors
}

/// Parse and validate a scene spec from JSON text. On failure the caller
/// gets every validation error, not just the first.
pub fn parse_scene(text: &str) -> Result<Scene, Vec<ValidationError>> {
    let value: Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(e) => {
            return Err(vec![ValidationError {
                path: "$".to_string(),
                message: format!("invalid JSON: {e}"),
            }]);
        }
    };
    let errors = validate_scene_json(&value);
    if !errors.is_empty() {
        return Err(errors);
    }
    match serde_json::from_value::<Scene>(value) {
        Ok(scene) => Ok(scene),
        Err(e) => Err(vec![ValidationError {
            path: "$".to_string(),
            message: format!("scene does not match the spec shape: {e}"),
        }]),
    }
}
