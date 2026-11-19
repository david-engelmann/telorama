//! The scene spec, version 0.1.
//!
//! The spec is the agent-facing authoring surface. It is deliberately
//! small and semantic: an agent supplies the question, the headline, data
//! references, panel choices, and thresholds with meaning. Geometry,
//! formatting, and layout belong to the renderer and never appear here.

use crate::field::Thresholds;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The only scene version this build reads and writes.
pub const SCENE_VERSION: &str = "0.1";

/// A complete scene: one question, one headline, a small set of panels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    /// Spec version. Must equal [`SCENE_VERSION`].
    pub scene: String,
    pub title: String,
    /// The takeaway sentence. Required, because an answer-shaped scene
    /// leads with its conclusion and uses the chart as proof.
    pub headline: String,
    pub context: Context,
    pub data: Vec<DataRef>,
    pub panels: Vec<Panel>,
    #[serde(default)]
    pub narrative: Vec<NarrativeItem>,
}

/// Shared scene context: the window every panel reads, plus provenance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Context {
    pub range: TimeRange,
    #[serde(default)]
    pub filter: BTreeMap<String, String>,
    pub provenance: Provenance,
}

/// A half-open time window in Unix milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    pub from_unix_ms: i64,
    pub to_unix_ms: i64,
}

impl TimeRange {
    /// Window length in seconds, for rate calculations.
    pub fn seconds(&self) -> f64 {
        (self.to_unix_ms - self.from_unix_ms) as f64 / 1000.0
    }
}

/// Who generated the scene and what question it answers. Printed on every
/// rendered scene as part of the provenance caption.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub generated_by: String,
    pub question: String,
}

/// A named data source a panel can reference. Every entry records exactly
/// how its data was obtained, so a rendered scene stays reproducible.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataRef {
    pub id: String,
    pub source: Source,
    /// Sampling state is always stated, never implied. A scene that does
    /// not know must say `unknown`.
    pub sampled: Sampling,
}

/// How a data entry was obtained.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Source {
    /// Source kind, for example `otlp_file`, `prometheus`, `tempo`.
    pub kind: String,
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub dialect: Option<String>,
    #[serde(default)]
    pub trace_id: Option<String>,
    /// Explicit histogram bucket edges for heatmap data. Bucket edges are
    /// data, not style, so they live in the spec.
    #[serde(default)]
    pub buckets: Option<Vec<f64>>,
}

/// Sampling state of a data entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sampling {
    Unsampled,
    Sampled,
    Unknown,
}

/// One panel in a scene. The set is closed on purpose: a new panel type
/// is a spec change, not a renderer plugin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Panel {
    Waterfall {
        data: String,
        #[serde(default)]
        highlight_critical_path: bool,
        #[serde(default)]
        highlight_errors: bool,
    },
    Stat {
        data: String,
        stat: StatKind,
        unit: Unit,
        #[serde(default)]
        service: Option<String>,
        /// Optional verdict thresholds for the reported value, in the
        /// field's own scale (nanoseconds for durations, a fraction for
        /// ratios). Empty means the value renders without a verdict.
        #[serde(default)]
        thresholds: Thresholds,
    },
    RedRow {
        data: String,
        #[serde(default)]
        service: Option<String>,
        /// Optional verdict thresholds for the error ratio. Empty means
        /// the standing rule applies: vermillion at five percent.
        #[serde(default)]
        thresholds: Thresholds,
    },
    Sankey {
        data: String,
        /// Name of the explicit residual bucket. Sankey flow must
        /// conserve, and whatever does not balance lands here, in the
        /// open, instead of vanishing.
        residual: String,
    },
    Flame {
        data: String,
        sample_type: String,
        min_samples: u64,
        sample_count: u64,
    },
    Heatmap {
        data: String,
        unit: Unit,
    },
    ServiceMap {
        data: String,
    },
    AgentRun {
        data: String,
    },
    AgentState {
        data: String,
    },
}

impl Panel {
    /// The data entry this panel reads.
    pub fn data_id(&self) -> &str {
        match self {
            Panel::Waterfall { data, .. }
            | Panel::Stat { data, .. }
            | Panel::RedRow { data, .. }
            | Panel::Sankey { data, .. }
            | Panel::Flame { data, .. }
            | Panel::Heatmap { data, .. }
            | Panel::ServiceMap { data, .. }
            | Panel::AgentRun { data, .. }
            | Panel::AgentState { data, .. } => data,
        }
    }

    /// Stable type name, matching the spec's `type` tag.
    pub fn type_name(&self) -> &'static str {
        match self {
            Panel::Waterfall { .. } => "waterfall",
            Panel::Stat { .. } => "stat",
            Panel::RedRow { .. } => "red_row",
            Panel::Sankey { .. } => "sankey",
            Panel::Flame { .. } => "flame",
            Panel::Heatmap { .. } => "heatmap",
            Panel::ServiceMap { .. } => "service_map",
            Panel::AgentRun { .. } => "agent_run",
            Panel::AgentState { .. } => "agent_state",
        }
    }
}

/// The statistic a stat panel reports. There is no average variant for
/// durations anywhere in this enum, and the validator rejects average and
/// mean duration statistics at the JSON layer before typing, so the rule
/// holds even for hand-written specs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatKind {
    #[serde(rename = "p50")]
    P50,
    #[serde(rename = "p95")]
    P95,
    #[serde(rename = "p99")]
    P99,
    #[serde(rename = "count")]
    Count,
    #[serde(rename = "rate")]
    Rate,
    #[serde(rename = "error_ratio")]
    ErrorRatio,
    #[serde(rename = "last")]
    Last,
}

impl StatKind {
    /// The percentile for percentile statistics.
    pub fn percentile(&self) -> Option<u32> {
        match self {
            StatKind::P50 => Some(50),
            StatKind::P95 => Some(95),
            StatKind::P99 => Some(99),
            _ => None,
        }
    }

    /// Spec name of the statistic.
    pub fn name(&self) -> &'static str {
        match self {
            StatKind::P50 => "p50",
            StatKind::P95 => "p95",
            StatKind::P99 => "p99",
            StatKind::Count => "count",
            StatKind::Rate => "rate",
            StatKind::ErrorRatio => "error_ratio",
            StatKind::Last => "last",
        }
    }
}

/// The unit a value is measured in. Units are explicit in the spec so the
/// renderer never guesses a scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    Duration,
    Count,
    Ratio,
    RequestsPerSecond,
    Bytes,
}

impl Unit {
    /// Spec name of the unit.
    pub fn name(&self) -> &'static str {
        match self {
            Unit::Duration => "duration",
            Unit::Count => "count",
            Unit::Ratio => "ratio",
            Unit::RequestsPerSecond => "requests_per_second",
            Unit::Bytes => "bytes",
        }
    }
}

/// One narrative statement, anchored to a panel by index so the text can
/// never float free of the evidence it describes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NarrativeItem {
    pub text: String,
    pub panel: usize,
}
