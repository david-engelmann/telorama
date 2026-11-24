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

/// The render theme a scene declares. Light is the default and
/// the historical rendering. Dark carries the same layout and
/// figures with a palette derived for dark grounds, per
/// INV-74 to INV-77.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    /// The historical palette on a white ground. This is the
    /// default when a scene declares no theme.
    #[default]
    Light,
    /// The palette derived for dark grounds. Layout and
    /// figures are the same as in light.
    Dark,
}

impl Theme {
    fn is_light(&self) -> bool {
        matches!(self, Theme::Light)
    }
}

/// A complete scene. One question, one headline, a small set of panels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    /// Spec version. Must equal [`SCENE_VERSION`].
    pub scene: String,
    /// The scene's name, printed at the top of every rendering.
    pub title: String,
    /// The takeaway sentence. Required, because an answer-shaped scene
    /// leads with its conclusion and uses the chart as proof.
    pub headline: String,
    /// The window, filter, and provenance every panel in this
    /// scene shares.
    pub context: Context,
    /// The data entries panels reference by id. At least one
    /// entry is required.
    pub data: Vec<DataRef>,
    /// The panels to render, in the order they appear on the
    /// page. At least one is required.
    pub panels: Vec<Panel>,
    /// Statements printed with the scene, each anchored to a
    /// panel by index. Empty when the scene carries none.
    #[serde(default)]
    pub narrative: Vec<NarrativeItem>,
    /// The render theme. Absent means light, and light scenes
    /// serialize without the field, so existing payloads keep
    /// their exact shape.
    #[serde(default, skip_serializing_if = "Theme::is_light")]
    pub theme: Theme,
}

/// Shared scene context. The window every panel reads, plus provenance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Context {
    /// The half-open window every panel reads its data from.
    pub range: TimeRange,
    /// Selection criteria the scene's data was gathered under,
    /// as key and value pairs (for example `env` set to `prod`).
    /// Empty when the data is unfiltered.
    #[serde(default)]
    pub filter: BTreeMap<String, String>,
    /// Printed on every rendering as part of the provenance
    /// caption.
    pub provenance: Provenance,
}

/// A half-open time window in Unix milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start of the window in Unix milliseconds. Inclusive.
    pub from_unix_ms: i64,
    /// End of the window in Unix milliseconds. Exclusive,
    /// because the window is half-open.
    pub to_unix_ms: i64,
}

impl TimeRange {
    /// Window length in seconds, for rate calculations.
    pub fn seconds(&self) -> f64 {
        self.to_unix_ms.saturating_sub(self.from_unix_ms) as f64 / 1000.0
    }
}

/// Who generated the scene and what question it answers. Printed on every
/// rendered scene as part of the provenance caption.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    /// The agent or tool that wrote the scene, named in the
    /// provenance caption.
    pub generated_by: String,
    /// The question the scene answers, in the words it was
    /// asked. Printed in the provenance caption.
    pub question: String,
}

/// A named data source a panel can reference. Every entry records exactly
/// how its data was obtained, so a rendered scene stays reproducible.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataRef {
    /// The name panels use to reference this entry in their
    /// `data` field.
    pub id: String,
    /// How this entry's data was obtained.
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
    /// The query that produced the data, recorded so the scene
    /// can be reproduced. A source declares a query, a trace
    /// id, or both.
    #[serde(default)]
    pub query: Option<String>,
    /// The dialect the data follows, when the source kind has
    /// more than one. Absent when the kind names a single
    /// format.
    #[serde(default)]
    pub dialect: Option<String>,
    /// The trace this entry carries, by its OpenTelemetry
    /// trace id. A source declares a trace id, a query, or
    /// both.
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
    /// The data is complete. Nothing was dropped before the
    /// scene read it.
    Unsampled,
    /// The data passed through sampling, so counts and ratios
    /// describe the sample rather than the whole stream.
    Sampled,
    /// The sampling state is not known. A scene that cannot
    /// say must say this rather than imply completeness.
    Unknown,
}

/// One panel in a scene. The set is closed on purpose, so a new
/// panel type is a spec change, not a renderer plugin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Panel {
    /// A span waterfall for one trace, one bar per span over
    /// time.
    Waterfall {
        /// The `id` of the data entry holding the trace.
        data: String,
        /// When true, the spans on the trace's critical path
        /// are drawn in the accent color.
        #[serde(default)]
        highlight_critical_path: bool,
        /// When true, error spans are drawn in the error color.
        #[serde(default)]
        highlight_errors: bool,
    },
    /// A single headline number computed over a data entry,
    /// with its derivation printed under it.
    Stat {
        /// The `id` of the data entry this panel reads.
        data: String,
        /// Which statistic is computed over the data entry's
        /// spans.
        stat: StatKind,
        /// The scale the reported value is measured in. It also
        /// fixes how the value is formatted.
        unit: Unit,
        /// When set, only this service's spans feed the
        /// statistic. Absent means every span in the data entry
        /// counts.
        #[serde(default)]
        service: Option<String>,
        /// Optional verdict thresholds for the reported value, in the
        /// field's own scale (nanoseconds for durations, a fraction for
        /// ratios). Empty means the value renders without a verdict.
        #[serde(default)]
        thresholds: Thresholds,
    },
    /// Rate, errors, and duration percentiles for each
    /// service in a data entry, one row per service.
    RedRow {
        /// The `id` of the data entry this panel reads.
        data: String,
        /// When set, the table keeps only this service's row.
        /// Absent means one row per service in the data.
        #[serde(default)]
        service: Option<String>,
        /// Optional verdict thresholds for the error ratio. Empty means
        /// the standing rule applies, vermillion at five percent.
        #[serde(default)]
        thresholds: Thresholds,
    },
    /// A flow diagram of calls between services, staged by
    /// call depth. Flow must conserve, with any imbalance
    /// landing in the residual bucket.
    Sankey {
        /// The `id` of the data entry this panel reads.
        data: String,
        /// Name of the explicit residual bucket. Sankey flow must
        /// conserve, and whatever does not balance lands here, in the
        /// open, instead of vanishing.
        residual: String,
    },
    /// A flame graph over profile samples, where a frame's
    /// width is its share of the sample total.
    Flame {
        /// The `id` of the data entry this panel reads.
        data: String,
        /// What one sample measures, for example `cpu`.
        sample_type: String,
        /// The smallest sample total the panel will render.
        /// Rendering fails when the data carries fewer, so a
        /// thin profile cannot pose as a solid one. At least 1.
        min_samples: u64,
        /// The sample total the author declares. It must equal
        /// the total the profile data actually carries, and
        /// rendering fails when the two disagree.
        sample_count: u64,
    },
    /// A distribution over time, bucketed by the explicit
    /// edges declared in the data entry's source.
    Heatmap {
        /// The `id` of the data entry this panel reads.
        data: String,
        /// The scale the bucketed values are measured in.
        unit: Unit,
    },
    /// A graph of the services in a data entry and the calls
    /// observed between them.
    ServiceMap {
        /// The `id` of the data entry this panel reads.
        data: String,
    },
    /// An agent session drawn as a waterfall of turns, model
    /// calls, and tool calls.
    AgentRun {
        /// The `id` of the data entry holding the session.
        data: String,
    },
    /// A panel of what each agent in a session is doing, why,
    /// and what it is waiting on.
    AgentState {
        /// The `id` of the data entry holding the agent state.
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
    /// The median span duration.
    #[serde(rename = "p50")]
    P50,
    /// The 95th percentile span duration.
    #[serde(rename = "p95")]
    P95,
    /// The 99th percentile span duration.
    #[serde(rename = "p99")]
    P99,
    /// The number of spans in the window.
    #[serde(rename = "count")]
    Count,
    /// Spans per second over the scene's window.
    #[serde(rename = "rate")]
    Rate,
    /// Error spans as a fraction of all spans in the window.
    #[serde(rename = "error_ratio")]
    ErrorRatio,
    /// The duration of the last span to finish in the window.
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
    /// A time span, carried in nanoseconds and formatted to a
    /// readable scale.
    Duration,
    /// A number of things, with no unit suffix.
    Count,
    /// A fraction between zero and one, such as an error
    /// ratio.
    Ratio,
    /// A rate of requests over one second.
    RequestsPerSecond,
    /// A size in bytes, formatted to a readable scale.
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
    /// The statement itself, printed beside the panel it is
    /// anchored to.
    pub text: String,
    /// Index into the scene's `panels` list, naming the panel
    /// this statement describes.
    pub panel: usize,
}
