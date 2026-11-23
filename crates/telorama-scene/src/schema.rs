//! The JSON Schema for scene spec 0.1, as data.
//!
//! The schema is the contract other tools check against. The validator in
//! this crate enforces the honesty rules a schema cannot express; the
//! schema carries the shape.

use serde_json::{Value, json};

/// The JSON Schema (draft 2020-12) for the scene spec.
pub fn scene_schema() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://telorama.dev/schema/scene-0.1.json",
        "title": "telorama scene spec",
        "type": "object",
        "required": ["scene", "title", "headline", "context", "data", "panels"],
        "properties": {
            "scene": { "const": "0.1" },
            "theme": { "enum": ["light", "dark"], "default": "light" },
            "title": { "type": "string", "minLength": 1 },
            "headline": { "type": "string", "minLength": 1 },
            "context": {
                "type": "object",
                "required": ["range", "provenance"],
                "properties": {
                    "range": {
                        "type": "object",
                        "required": ["from_unix_ms", "to_unix_ms"],
                        "properties": {
                            "from_unix_ms": { "type": "integer" },
                            "to_unix_ms": { "type": "integer" }
                        }
                    },
                    "filter": { "type": "object", "additionalProperties": { "type": "string" } },
                    "provenance": {
                        "type": "object",
                        "required": ["generated_by", "question"],
                        "properties": {
                            "generated_by": { "type": "string", "minLength": 1 },
                            "question": { "type": "string", "minLength": 1 }
                        }
                    }
                }
            },
            "data": {
                "type": "array",
                "minItems": 1,
                "items": {
                    "type": "object",
                    "required": ["id", "source", "sampled"],
                    "properties": {
                        "id": { "type": "string", "minLength": 1 },
                        "source": {
                            "type": "object",
                            "required": ["kind"],
                            "properties": {
                                "kind": { "type": "string", "minLength": 1 },
                                "query": { "type": "string" },
                                "dialect": { "type": "string" },
                                "trace_id": { "type": "string" },
                                "buckets": { "type": "array", "items": { "type": "number" } }
                            },
                            "anyOf": [
                                { "required": ["query"] },
                                { "required": ["trace_id"] }
                            ]
                        },
                        "sampled": { "enum": ["unsampled", "sampled", "unknown"] }
                    }
                }
            },
            "panels": {
                "type": "array",
                "minItems": 1,
                "items": {
                    "type": "object",
                    "required": ["type", "data"],
                    "properties": {
                        "type": { "enum": ["waterfall", "stat", "red_row", "sankey", "flame", "heatmap", "service_map", "agent_run", "agent_state"] },
                        "data": { "type": "string", "minLength": 1 },
                        "thresholds": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["above", "color"],
                                "properties": {
                                    "above": { "type": "number" },
                                    "color": { "type": "string", "pattern": "^#[0-9A-Fa-f]{6}$" }
                                }
                            }
                        }
                    }
                }
            },
            "narrative": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["text", "panel"],
                    "properties": {
                        "text": { "type": "string", "minLength": 1 },
                        "panel": { "type": "integer", "minimum": 0 }
                    }
                }
            }
        }
    })
}

/// The schema as canonical JSON text: sorted keys, two-space indent, one
/// trailing newline. Byte-stable for a given crate version.
pub fn scene_schema_text() -> String {
    let mut text = serde_json::to_string_pretty(&scene_schema()).unwrap_or_default();
    text.push('\n');
    text
}
