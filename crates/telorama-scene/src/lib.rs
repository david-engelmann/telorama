//! telorama-scene: the scene spec, its schema, the honesty validator, and
//! the pure derivations that turn spans into panel-ready facts.
//!
//! Correctness lives in this crate. It performs no I/O and renders
//! nothing; every public function is a deterministic function of its
//! inputs.

pub mod derive;
pub mod field;
pub mod profile;
pub mod schema;
pub mod spec;
pub mod trace;
pub mod validate;

pub use derive::{
    Exemplar, FlameNode, HeatmapGrid, HeatmapRow, MapEdge, RedStats, ServiceGraph, WaterfallRow,
    content_hash, critical_path, flame_from_spans, flame_tree, heatmap_grid, percentile_nanos,
    red_by_service, service_graph, waterfall_rows,
};
pub use field::{
    ThresholdStep, Thresholds, format_bytes, format_duration_nanos, format_number, format_ratio,
    format_value,
};
pub use profile::StackSample;
pub use schema::{scene_schema, scene_schema_text};
pub use spec::{
    Context, DataRef, NarrativeItem, Panel, Provenance, SCENE_VERSION, Sampling, Scene, Source,
    StatKind, TimeRange, Unit,
};
pub use trace::{Span, SpanKind};
pub use validate::{ValidationError, parse_scene, validate_scene_json};
