//! telorama-scene: the scene spec, its schema, the honesty validator, and
//! the pure derivations that turn spans into panel-ready facts.
//!
//! Correctness lives in this crate. It performs no I/O and renders
//! nothing; every public function is a deterministic function of its
//! inputs.

pub mod agent;
pub mod compose;
pub mod derive;
pub mod field;
pub mod frames;
pub mod investigate;
pub mod profile;
pub mod schema;
pub mod spec;
pub mod trace;
pub mod validate;

pub use agent::{
    AgentRun, AgentStateEntry, AgentStep, AgentTurn, Coverage, ModelCall, Pricing, SubagentRun,
    ToolCall,
};
pub use compose::incident_scene;
pub use derive::{
    Exemplar, FlameNode, HeatmapGrid, HeatmapRow, MapEdge, RedStats, SankeyFlow, SankeyLink,
    SankeyNode, ServiceGraph, WaterfallRow, content_hash, critical_path, flame_from_spans,
    flame_tree, heatmap_grid, percentile_nanos, red_by_service, sankey_flow, service_graph,
    waterfall_rows,
};
pub use field::{
    ThresholdStep, Thresholds, format_bytes, format_duration_nanos, format_number, format_ratio,
    format_value,
};
pub use frames::{Frame, session_frames};
pub use investigate::{EvidencePoint, Finding, investigate};
pub use profile::StackSample;
pub use schema::{scene_schema, scene_schema_text};
pub use spec::{
    Context, DataRef, NarrativeItem, Panel, Provenance, SCENE_VERSION, Sampling, Scene, Source,
    StatKind, TimeRange, Unit,
};
pub use trace::{Span, SpanKind};
pub use validate::{ValidationError, parse_scene, validate_scene_json};
