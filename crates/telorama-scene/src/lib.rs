//! telorama-scene: the scene spec, its schema, the honesty validator, and
//! the pure derivations that turn spans into panel-ready facts.
//!
//! Correctness lives in this crate. It performs no I/O and renders
//! nothing; every public function is a deterministic function of its
//! inputs.
//!
//! # Example
//!
//! Parse a scene spec from its JSON text. The same call validates
//! it against the schema and the honesty rules, so a scene that
//! parses is a scene that may be rendered.
//!
//! ```rust
//! let text = r#"{
//!     "scene": "0.1",
//!     "title": "One request",
//!     "headline": "The request answered in 500 ms",
//!     "context": {
//!         "range": {"from_unix_ms": 0, "to_unix_ms": 1000},
//!         "provenance": {"generated_by": "agent", "question": "How long did it take?"}
//!     },
//!     "data": [{
//!         "id": "trace",
//!         "sampled": "unknown",
//!         "source": {"kind": "otlp_file", "trace_id": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}
//!     }],
//!     "panels": [{"type": "waterfall", "data": "trace"}]
//! }"#;
//! let scene = telorama_scene::parse_scene(text).expect("the scene must validate");
//! assert_eq!(scene.title, "One request");
//! ```

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
pub use compose::{incident_scene, incident_scene_with_theme};
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
pub use frames::{Frame, session_frames, session_frames_with_theme};
pub use investigate::{EvidencePoint, Finding, investigate};
pub use profile::StackSample;
pub use schema::{scene_schema, scene_schema_text};
pub use spec::{
    Context, DataRef, NarrativeItem, Panel, Provenance, SCENE_VERSION, Sampling, Scene, Source,
    StatKind, Theme, TimeRange, Unit,
};
pub use trace::{Span, SpanKind};
pub use validate::{ValidationError, parse_scene, validate_scene_json};
