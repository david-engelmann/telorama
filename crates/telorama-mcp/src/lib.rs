//! telorama-mcp: the MCP server for the telorama visualization engine.
//!
//! The server speaks JSON-RPC 2.0 over stdio, one message per line,
//! and implements the MCP surface the dossier settled: tools that
//! validate, inspect, render, and summarize, returning the triple
//! output contract on every render (concise text for the model and
//! text-only hosts, compact model-legible structuredContent, and the
//! SVG itself in `_meta` for the MCP Apps view), plus the `ui://`
//! view resource the render tool links to, and one app-only
//! drill-down tool the view can call for panel detail the model never
//! needs in context.
//!
//! The protocol core is a pure function from a request line to a
//! response line, so the whole surface is testable without a host.
//!
//! # Example
//!
//! Ask the server what tools it offers, through the same pure
//! handler the stdio loop drives.
//!
//! ```rust
//! let response = telorama_mcp::handle_message(
//!     r#"{"jsonrpc": "2.0", "id": 1, "method": "tools/list"}"#,
//! )
//! .expect("a request with an id always answers");
//! assert!(response.contains("validate_scene"));
//! ```

#![deny(missing_docs)]

use serde_json::{Value, json};
use telorama_scene::{
    AgentRun, Finding, Panel, Scene, Span, StackSample, StatKind, content_hash, critical_path,
    flame_from_spans, flame_tree, heatmap_grid, investigate, percentile_nanos, red_by_service,
    sankey_flow, service_graph, waterfall_rows,
};

/// The MCP Apps view resource the render tool links to.
pub const VIEW_URI: &str = "ui://telorama/scene";

const PROTOCOL_VERSION: &str = "2025-06-18";
const VIEW_MIME: &str = "text/html;profile=mcp-app";

/// The MCP Apps view: one self-contained HTML document. It speaks the
/// app bridge (ui/initialize, then it waits for the tool result
/// notification) and injects the SVG the render tool carried in its
/// `_meta`. It fetches nothing.
pub const VIEW_HTML: &str = concat!(
    r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>telorama scene</title>
<style>body{margin:0;background:#FFFFFF;color:#1A1A1A;font-family:system-ui,sans-serif}main{padding:12px}svg{width:100%;height:auto}nav{display:none;padding:8px 12px;gap:10px;align-items:center}nav button{font:inherit;padding:2px 10px}nav span{color:#555555}</style>
</head>
<body>
<nav id="stepper"><button id="prev" type="button">Previous</button><span id="pos"></span><button id="next" type="button">Next</button></nav>
<main id="scene"><p>Waiting for a scene render.</p></main>
<script>
"use strict";
var sceneEl = document.getElementById("scene");
var stepperEl = document.getElementById("stepper");
var posEl = document.getElementById("pos");
var prevEl = document.getElementById("prev");
var nextEl = document.getElementById("next");
var frames = null;
var frameIndex = 0;
function showFrame(i) {
  frameIndex = i;
  sceneEl.innerHTML = frames[i];
  posEl.textContent = "Frame " + (i + 1) + " of " + frames.length;
  prevEl.disabled = i === 0;
  nextEl.disabled = i === frames.length - 1;
  notifySize();
}
prevEl.addEventListener("click", function () { if (frames && frameIndex > 0) { showFrame(frameIndex - 1); } });
nextEl.addEventListener("click", function () { if (frames && frameIndex < frames.length - 1) { showFrame(frameIndex + 1); } });
var initId = 1;
function post(msg) { window.parent.postMessage(msg, "*"); }
function notifySize() {
  post({jsonrpc: "2.0", method: "ui/notifications/size-changed", params: {height: document.body.scrollHeight}});
}
window.addEventListener("message", function (event) {
  var msg = event.data;
  if (!msg || msg.jsonrpc !== "2.0") { return; }
  if (msg.id === initId && msg.result) {
    post({jsonrpc: "2.0", method: "ui/notifications/initialized"});
    return;
  }
  if (msg.method === "ui/notifications/tool-result") {
    var meta = (msg.params && msg.params._meta) || {};
    var delivered = meta["telorama/frames"];
    if (Array.isArray(delivered) && delivered.length > 0) {
      frames = delivered;
      stepperEl.style.display = "flex";
      showFrame(frames.length - 1);
      return;
    }
    frames = null;
    stepperEl.style.display = "none";
    var svg = meta["telorama/svg"];
    if (typeof svg === "string") {
      sceneEl.innerHTML = svg;
      notifySize();
    }
  }
});
post({jsonrpc: "2.0", id: initId, method: "ui/initialize", params: {appCapabilities: {}, protocolVersion: "2026-01-26", appInfo: {name: "telorama-view", version: ""##,
    env!("CARGO_PKG_VERSION"),
    r##""}}});
</script>
</body>
</html>
"##
);

/// Handle one JSON-RPC line. Returns the response line, or None for
/// notifications, which take no reply.
pub fn handle_message(line: &str) -> Option<String> {
    let msg: Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(e) => {
            return Some(error_line(
                Value::Null,
                -32700,
                &format!("parse error: {e}"),
            ));
        }
    };
    let id = msg.get("id").cloned();
    let method = msg.get("method").and_then(Value::as_str).unwrap_or("");
    id.as_ref()?;
    let id = id.expect("checked above");
    let params = msg.get("params").cloned().unwrap_or(Value::Null);
    let result = match method {
        "initialize" => Ok(json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {"tools": {}, "resources": {}},
            "serverInfo": {"name": "telorama", "version": env!("CARGO_PKG_VERSION")}
        })),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({"tools": tool_definitions()})),
        "tools/call" => Ok(call_tool(&params)),
        "resources/list" => Ok(json!({
            "resources": [{
                "uri": VIEW_URI,
                "name": "telorama scene view",
                "description": "The MCP Apps view that displays a rendered telorama scene",
                "mimeType": VIEW_MIME
            }]
        })),
        "resources/read" => read_resource(&params),
        _ => Err((-32601, format!("method not found: {method}"))),
    };
    match result {
        Ok(value) => Some(
            serde_json::to_string(&json!({"jsonrpc": "2.0", "id": id, "result": value}))
                .expect("response must serialize"),
        ),
        Err((code, message)) => Some(error_line(id, code, &message)),
    }
}

fn error_line(id: Value, code: i64, message: &str) -> String {
    serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {"code": code, "message": message}
    }))
    .expect("error response must serialize")
}

fn read_resource(params: &Value) -> Result<Value, (i64, String)> {
    let uri = params.get("uri").and_then(Value::as_str).unwrap_or("");
    if uri != VIEW_URI {
        return Err((-32602, format!("unknown resource: {uri}")));
    }
    Ok(json!({
        "contents": [{
            "uri": VIEW_URI,
            "mimeType": VIEW_MIME,
            "text": VIEW_HTML
        }]
    }))
}

/// The tool catalogue. The render tool links the view in its
/// definition metadata, as SEP-1865 specifies; the drill-down tool is
/// app-only, so hosts must not show it to the model.
fn tool_definitions() -> Value {
    let data_args = json!({
        "traces": {"type": "string", "description": "OTLP trace export JSON text"},
        "profile": {"type": "string", "description": "Folded-stack profile text for flame panels"},
        "session_logs": {"type": "string", "description": "OTLP log export text for agent_run panels"},
        "session_spans": {"type": "string", "description": "OTLP trace export text to fuse into the session"},
        "dialect": {"type": "string", "enum": ["claude_code", "codex", "genai", "openinference"], "description": "Session dialect, default claude_code. The genai and openinference dialects read session_spans."},
        "agent_state": {"type": "string", "description": "Agent state report JSON text for agent_state panels, a list of entries with agent, doing, and why"}
    });
    let mut render_props = json!({
        "scene": {"type": "string", "description": "The scene spec JSON text"}
    });
    let mut detail_props = json!({
        "scene": {"type": "string", "description": "The scene spec JSON text"},
        "panel_index": {"type": "integer", "description": "Zero-based index of the panel to detail"}
    });
    for (k, v) in data_args.as_object().expect("object") {
        render_props[k] = v.clone();
        detail_props[k] = v.clone();
    }
    json!([
        {
            "name": "validate_scene",
            "description": "Validate a telorama scene spec against the schema and the honesty rules without rendering it.",
            "inputSchema": {
                "type": "object",
                "properties": {"scene": {"type": "string", "description": "The scene spec JSON text"}},
                "required": ["scene"]
            }
        },
        {
            "name": "inspect_traces",
            "description": "Summarize an OTLP trace export: span and trace counts, the window, and per-service span and error counts.",
            "inputSchema": {
                "type": "object",
                "properties": {"traces": {"type": "string", "description": "OTLP trace export JSON text"}},
                "required": ["traces"]
            }
        },
        {
            "name": "render_scene",
            "description": "Render a telorama scene to a deterministic SVG. Returns concise text, a compact structured summary, and the SVG for the view.",
            "inputSchema": {
                "type": "object",
                "properties": render_props,
                "required": ["scene"]
            },
            "_meta": {"ui": {"resourceUri": VIEW_URI}}
        },
        {
            "name": "session_run",
            "description": "Adapt an agent session export into the canonical run model and return it as JSON. Claude Code and Codex read OTLP logs; the GenAI and OpenInference dialects read OTLP spans.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "logs": {"type": "string", "description": "OTLP log export text"},
                    "spans": {"type": "string", "description": "OTLP trace export text: fusion input for the Claude Code dialect, the session itself for the span dialects"},
                    "dialect": {"type": "string", "enum": ["claude_code", "codex", "genai", "openinference"]}
                },
                "required": []
            }
        },
        {
            "name": "session_frames",
            "description": "Build the live frame stream for a session export: one complete scene per turn boundary, each a pure function of the session prefix through that turn. The view steps through the frames; the final frame rides telorama/svg and every frame rides telorama/frames.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "logs": {"type": "string", "description": "OTLP log export text"},
                    "spans": {"type": "string", "description": "OTLP trace export text: fusion input for the Claude Code dialect, the session itself for the span dialects"},
                    "dialect": {"type": "string", "enum": ["claude_code", "codex", "genai", "openinference"]}
                },
                "required": []
            },
            "_meta": {"ui": {"resourceUri": VIEW_URI}}
        },
        {
            "name": "scene_panel_detail",
            "description": "Derived detail for one panel of a scene, for the view's drill-down. App-only: not shown to the model.",
            "inputSchema": {
                "type": "object",
                "properties": detail_props,
                "required": ["scene", "panel_index"]
            },
            "_meta": {"ui": {"visibility": ["app"]}}
        },
        {
            "name": "incident_scene",
            "description": "Compose the incident template from a trace export: service map, RED row, exemplar waterfall, and latency heatmap, with the headline and narrative derived from the data. Returns the triple contract and the composed scene spec.",
            "inputSchema": {
                "type": "object",
                "properties": detail_props,
                "required": ["traces"]
            },
            "_meta": {"ui": {"resourceUri": VIEW_URI}}
        }
    ])
}

/// The data a tool call carries, built once per call.
struct Inputs {
    spans: Vec<Span>,
    stacks: Vec<StackSample>,
    run: Option<AgentRun>,
    state: Vec<telorama_scene::AgentStateEntry>,
}

fn arg_str<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(Value::as_str)
}

fn build_inputs(args: &Value) -> Result<Inputs, String> {
    let parsed = match arg_str(args, "traces") {
        Some(text) => telorama_ingest::parse_otlp_traces(text).map_err(|e| e.to_string())?,
        None => Vec::new(),
    };
    // One count per span on every surface (INV-86): the dedup
    // the CLI's combined set applies, first occurrence kept, so
    // the MCP tools report the same counts as the CLI on the
    // same export.
    let mut seen: std::collections::BTreeSet<(&str, &str)> = std::collections::BTreeSet::new();
    let spans: Vec<Span> = parsed
        .iter()
        .filter(|s| seen.insert((s.trace_id.as_str(), s.span_id.as_str())))
        .cloned()
        .collect();
    let stacks = match arg_str(args, "profile") {
        Some(text) => telorama_ingest::parse_folded_stacks(text).map_err(|e| e.to_string())?,
        None => Vec::new(),
    };
    let run = match arg_str(args, "session_logs") {
        Some(text) => Some(parse_session(text, arg_str(args, "session_spans"), args)?),
        None => match (
            arg_str(args, "dialect").unwrap_or("claude_code"),
            arg_str(args, "session_spans"),
        ) {
            ("genai", Some(text)) => Some(
                telorama_ingest::parse_genai_session(
                    &telorama_ingest::parse_otlp_traces(text).map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?,
            ),
            ("openinference", Some(text)) => Some(
                telorama_ingest::parse_openinference_session(
                    &telorama_ingest::parse_otlp_traces(text).map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?,
            ),
            _ => None,
        },
    };
    let state = match arg_str(args, "agent_state") {
        Some(text) => telorama_ingest::parse_agent_state(text).map_err(|e| e.to_string())?,
        None => Vec::new(),
    };
    Ok(Inputs {
        spans,
        stacks,
        run,
        state,
    })
}

fn parse_session(logs: &str, spans_text: Option<&str>, args: &Value) -> Result<AgentRun, String> {
    let spans = match spans_text {
        Some(text) => telorama_ingest::parse_otlp_traces(text).map_err(|e| e.to_string())?,
        None => Vec::new(),
    };
    match arg_str(args, "dialect").unwrap_or("claude_code") {
        "claude_code" => {
            telorama_ingest::parse_claude_code_session(logs, &spans).map_err(|e| e.to_string())
        }
        "codex" => telorama_ingest::parse_codex_session(logs, &spans).map_err(|e| e.to_string()),
        other => Err(format!(
            "unknown dialect {other:?}: expected claude_code or codex"
        )),
    }
}

fn call_tool(params: &Value) -> Value {
    let name = params.get("name").and_then(Value::as_str).unwrap_or("");
    let args = params.get("arguments").cloned().unwrap_or(json!({}));
    match name {
        "validate_scene" => tool_validate(&args),
        "inspect_traces" => tool_inspect(&args),
        "render_scene" => tool_render(&args),
        "session_run" => tool_session(&args),
        "session_frames" => tool_frames(&args),
        "scene_panel_detail" => tool_detail(&args),
        "incident_scene" => tool_incident(&args),
        _ => tool_error(&format!("unknown tool: {name}")),
    }
}

fn tool_error(message: &str) -> Value {
    json!({"content": [{"type": "text", "text": message}], "isError": true})
}

fn tool_ok(text: String, structured: Value, meta: Option<Value>) -> Value {
    let mut out = json!({
        "content": [{"type": "text", "text": text}],
        "isError": false,
        "structuredContent": structured
    });
    if let Some(m) = meta {
        out["_meta"] = m;
    }
    out
}

fn invalid_scene(errors: &[telorama_scene::ValidationError]) -> Value {
    let messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
    let mut out = tool_error(&messages.join("\n"));
    out["structuredContent"] = json!({"valid": false, "errors": messages});
    out
}

fn parse_scene_arg(args: &Value) -> Result<Scene, Value> {
    let Some(text) = arg_str(args, "scene") else {
        return Err(tool_error("missing required argument: scene"));
    };
    telorama_scene::parse_scene(text).map_err(|errors| invalid_scene(&errors))
}

fn tool_validate(args: &Value) -> Value {
    match parse_scene_arg(args) {
        Ok(scene) => tool_ok(
            format!(
                "valid: {} ({} panels, {} data sources)",
                scene.title,
                scene.panels.len(),
                scene.data.len()
            ),
            json!({
                "valid": true,
                "title": scene.title,
                "panels": scene.panels.len(),
                "data_sources": scene.data.len()
            }),
            None,
        ),
        Err(result) => result,
    }
}

fn tool_inspect(args: &Value) -> Value {
    let Some(text) = arg_str(args, "traces") else {
        return tool_error("missing required argument: traces");
    };
    let spans = match telorama_ingest::parse_otlp_traces(text) {
        Ok(s) => s,
        Err(e) => return tool_error(&e.to_string()),
    };
    let summary = telorama_ingest::summarize(&spans);
    let mut text = format!(
        "spans: {}\ntraces: {}\nwindow: {} .. {}\n",
        summary.span_count,
        summary.trace_count,
        telorama_svg::fmt_time_ms(summary.from_unix_nano / 1_000_000),
        telorama_svg::fmt_time_ms(summary.to_unix_nano / 1_000_000)
    );
    let services: Vec<Value> = summary
        .services
        .iter()
        .map(|s| {
            text.push_str(&format!(
                "service {}: {} spans, {} errors\n",
                s.service, s.spans, s.errors
            ));
            json!({"service": s.service, "spans": s.spans, "errors": s.errors})
        })
        .collect();
    tool_ok(
        text.trim_end().to_string(),
        json!({
            "span_count": summary.span_count,
            "trace_count": summary.trace_count,
            "from_unix_nano": summary.from_unix_nano,
            "to_unix_nano": summary.to_unix_nano,
            "services": services
        }),
        None,
    )
}

/// The investigation layer for a scene and its inputs. Every render
/// path computes it automatically; no argument suppresses it.
fn investigate_inputs(scene: &Scene, inputs: &Inputs) -> Vec<Finding> {
    investigate(
        &inputs.spans,
        inputs.run.as_ref(),
        scene.context.range.seconds(),
    )
}

/// The findings as one text run, appended to a tool's text result so
/// the model reader gets the same ranking the human reader sees.
fn findings_text(findings: &[Finding]) -> String {
    if findings.is_empty() {
        return String::new();
    }
    let mut text = " Look here first:".to_string();
    for finding in findings {
        text.push_str(&format!(" {}. {}", finding.rank, finding.statement));
    }
    text
}

fn tool_render(args: &Value) -> Value {
    let scene = match parse_scene_arg(args) {
        Ok(s) => s,
        Err(result) => return result,
    };
    let inputs = match build_inputs(args) {
        Ok(i) => i,
        Err(m) => return tool_error(&m),
    };
    let findings = investigate_inputs(&scene, &inputs);
    let data = telorama_svg::SceneData {
        spans: &inputs.spans,
        stacks: &inputs.stacks,
        agent_run: inputs.run.as_ref(),
        agent_state: &inputs.state,
        findings: &findings,
    };
    let svg = match telorama_svg::render_scene_with_data(&scene, &data) {
        Ok(s) => s,
        Err(e) => return tool_error(&e.to_string()),
    };
    let types: Vec<&str> = scene.panels.iter().map(|p| p.type_name()).collect();
    let mut text = format!(
        "{}. {} Panels: {}. SVG {} bytes, fnv1a64 {}.",
        scene.title,
        scene.headline,
        types.join(", "),
        svg.len(),
        content_hash(svg.as_bytes())
    );
    text.push_str(&findings_text(&findings));
    let mut structured = scene_summary(&scene, &inputs, &svg);
    structured["investigation"] = serde_json::to_value(&findings).expect("findings must serialize");
    tool_ok(text, structured, Some(json!({"telorama/svg": svg})))
}

fn fmt_cost(value: f64) -> String {
    let mut text = format!("{value:.4}");
    while text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.push_str("00");
    }
    format!("${text}")
}

/// Adapt a session export into the canonical run, shared by the
/// session_run and session_frames tools.
fn adapt_session(args: &Value) -> Result<AgentRun, String> {
    let dialect = arg_str(args, "dialect").unwrap_or("claude_code");
    match dialect {
        "genai" | "openinference" => {
            let Some(spans_text) = arg_str(args, "spans") else {
                return Err(
                    "missing required argument: spans (span dialects read spans)".to_string(),
                );
            };
            let spans =
                telorama_ingest::parse_otlp_traces(spans_text).map_err(|e| e.to_string())?;
            match dialect {
                "genai" => telorama_ingest::parse_genai_session(&spans).map_err(|e| e.to_string()),
                _ => {
                    telorama_ingest::parse_openinference_session(&spans).map_err(|e| e.to_string())
                }
            }
        }
        _ => {
            let Some(logs) = arg_str(args, "logs") else {
                return Err("missing required argument: logs".to_string());
            };
            parse_session(logs, arg_str(args, "spans"), args)
        }
    }
}

fn tool_frames(args: &Value) -> Value {
    let run = match adapt_session(args) {
        Ok(r) => r,
        Err(m) => return tool_error(&m),
    };
    let frames = telorama_scene::session_frames(&run, "telorama-mcp");
    if frames.is_empty() {
        return tool_error("the session has no turns: nothing to frame");
    }
    let mut lines = Vec::new();
    let mut payloads = Vec::new();
    let mut svgs = Vec::new();
    for frame in &frames {
        let data = telorama_svg::SceneData {
            spans: &[],
            stacks: &[],
            agent_run: Some(&frame.run),
            agent_state: &[],
            findings: &frame.findings,
        };
        let svg = match telorama_svg::render_scene_with_data(&frame.scene, &data) {
            Ok(s) => s,
            Err(e) => return tool_error(&e.to_string()),
        };
        let input: u64 = frame
            .run
            .turns
            .iter()
            .fold(0u64, |acc, t| acc.saturating_add(t.input_tokens));
        let output: u64 = frame
            .run
            .turns
            .iter()
            .fold(0u64, |acc, t| acc.saturating_add(t.output_tokens));
        let cost: f64 = frame.run.turns.iter().map(|t| t.cost_usd).sum();
        lines.push(format!(
            "Frame {} of {}: {} {}, {} in, {} out, {}.",
            frame.index + 1,
            frames.len(),
            frame.turns,
            if frame.turns == 1 { "turn" } else { "turns" },
            input,
            output,
            fmt_cost(cost)
        ));
        payloads.push(json!({
            "frame": frame.index + 1,
            "turns": frame.turns,
            "scene": frame.scene,
            "findings": frame.findings,
            "svg_fnv1a64": content_hash(svg.as_bytes()),
        }));
        svgs.push(svg);
    }
    let structured = json!({
        "session_id": run.session_id,
        "dialect": run.dialect,
        "frames": payloads,
    });
    let meta = json!({
        "telorama/svg": svgs.last().expect("at least one frame"),
        "telorama/frames": svgs,
    });
    tool_ok(lines.join("\n"), structured, Some(meta))
}

fn tool_session(args: &Value) -> Value {
    let run = match adapt_session(args) {
        Ok(r) => r,
        Err(m) => return tool_error(&m),
    };
    let input: u64 = run
        .turns
        .iter()
        .fold(0u64, |acc, t| acc.saturating_add(t.input_tokens));
    let output: u64 = run
        .turns
        .iter()
        .fold(0u64, |acc, t| acc.saturating_add(t.output_tokens));
    let cost: f64 = run.turns.iter().map(|t| t.cost_usd).sum();
    let mut text = format!(
        "Session {} ({}): {} turns, {} in, {} out, {} total.",
        run.session_id,
        run.dialect,
        run.turns.len(),
        input,
        output,
        fmt_cost(cost)
    );
    if let Some(pricing) = &run.pricing {
        text.push_str(&format!(
            " The cost is an estimate priced by telorama from tokens at the price table dated {}.",
            pricing.table_as_of
        ));
        if !pricing.unpriced_models.is_empty() {
            text.push_str(&format!(
                " Unpriced models excluded: {}.",
                pricing.unpriced_models.join(", ")
            ));
        }
    }
    let run_findings = investigate(&[], Some(&run), 0.0);
    text.push_str(&findings_text(&run_findings));
    let mut structured = serde_json::to_value(&run).expect("the run model must serialize");
    structured["investigation"] =
        serde_json::to_value(&run_findings).expect("findings must serialize");
    tool_ok(text, structured, None)
}

fn tool_incident(args: &Value) -> Value {
    let inputs = match build_inputs(args) {
        Ok(i) => i,
        Err(m) => return tool_error(&m),
    };
    if inputs.spans.is_empty() {
        return tool_error("incident_scene needs traces");
    }
    let scene =
        match telorama_scene::incident_scene(&inputs.spans, inputs.run.as_ref(), "telorama-mcp") {
            Ok(s) => s,
            Err(m) => return tool_error(&m),
        };
    let findings = investigate_inputs(&scene, &inputs);
    let data = telorama_svg::SceneData {
        spans: &inputs.spans,
        stacks: &inputs.stacks,
        agent_run: inputs.run.as_ref(),
        agent_state: &inputs.state,
        findings: &findings,
    };
    let svg = match telorama_svg::render_scene_with_data(&scene, &data) {
        Ok(s) => s,
        Err(e) => return tool_error(&e.to_string()),
    };
    let mut structured = scene_summary(&scene, &inputs, &svg);
    structured["scene"] = serde_json::to_value(&scene).expect("the composed scene must serialize");
    structured["investigation"] = serde_json::to_value(&findings).expect("findings must serialize");
    let types: Vec<&str> = scene.panels.iter().map(|p| p.type_name()).collect();
    let mut text = format!(
        "{}. {} Panels: {}. SVG {} bytes, fnv1a64 {}.",
        scene.title,
        scene.headline,
        types.join(", "),
        svg.len(),
        telorama_scene::content_hash(svg.as_bytes())
    );
    text.push_str(&findings_text(&findings));
    tool_ok(text, structured, Some(json!({"telorama/svg": svg})))
}

fn tool_detail(args: &Value) -> Value {
    let scene = match parse_scene_arg(args) {
        Ok(s) => s,
        Err(result) => return result,
    };
    let inputs = match build_inputs(args) {
        Ok(i) => i,
        Err(m) => return tool_error(&m),
    };
    let Some(index) = args
        .get("panel_index")
        .and_then(Value::as_u64)
        .map(|i| i as usize)
    else {
        return tool_error("missing required argument: panel_index");
    };
    let Some(panel) = scene.panels.get(index) else {
        return tool_error(&format!(
            "panel_index {index} is out of range: the scene has {} panels",
            scene.panels.len()
        ));
    };
    let detail = panel_detail(&scene, &inputs, panel);
    tool_ok(
        format!(
            "Panel {index} ({}) of {}: derived detail in structuredContent.",
            panel.type_name(),
            scene.title
        ),
        detail,
        None,
    )
}

/// The spans a panel reads: the data entry's trace when it names one,
/// the whole set otherwise. The same rule the renderer uses.
fn dataset(scene: &Scene, spans: &[Span], data_id: &str) -> Vec<Span> {
    let entry = scene.data.iter().find(|d| d.id == data_id);
    match entry.and_then(|e| e.source.trace_id.as_deref()) {
        Some(trace_id) => spans
            .iter()
            .filter(|s| s.trace_id == trace_id)
            .cloned()
            .collect(),
        None => spans.to_vec(),
    }
}

fn trace_id_of<'a>(scene: &'a Scene, data_id: &str) -> Option<&'a str> {
    scene
        .data
        .iter()
        .find(|d| d.id == data_id)
        .and_then(|e| e.source.trace_id.as_deref())
}

fn flame_tree_for(scene: &Scene, inputs: &Inputs, data_id: &str) -> telorama_scene::FlameNode {
    match trace_id_of(scene, data_id) {
        Some(trace_id) => flame_from_spans(&inputs.spans, trace_id),
        None => flame_tree(&inputs.stacks),
    }
}

fn red_json(stats: &telorama_scene::RedStats) -> Value {
    json!({
        "service": stats.service,
        "calls": stats.calls,
        "errors": stats.errors,
        "error_ratio": stats.error_ratio,
        "rate_per_sec": stats.rate_per_sec,
        "p50_nanos": stats.p50_nanos,
        "p95_nanos": stats.p95_nanos,
        "p99_nanos": stats.p99_nanos,
        "basis": stats.basis
    })
}

fn scene_summary(scene: &Scene, inputs: &Inputs, svg: &str) -> Value {
    let panels: Vec<Value> = scene
        .panels
        .iter()
        .map(|p| panel_summary(scene, inputs, p))
        .collect();
    json!({
        "title": scene.title,
        "headline": scene.headline,
        "panels": panels,
        "provenance": {
            "generated_by": scene.context.provenance.generated_by,
            "question": scene.context.provenance.question
        },
        "window": {
            "from_unix_ms": scene.context.range.from_unix_ms,
            "to_unix_ms": scene.context.range.to_unix_ms
        },
        "svg_bytes": svg.len(),
        "svg_fnv1a64": content_hash(svg.as_bytes())
    })
}

/// Compact per-panel figures for structuredContent: enough for the
/// model to reason about the scene, small enough to stay in context.
fn panel_summary(scene: &Scene, inputs: &Inputs, panel: &Panel) -> Value {
    let window = scene.context.range.seconds();
    match panel {
        Panel::Waterfall { data, .. } => {
            let tid = trace_id_of(scene, data).unwrap_or("");
            json!({
                "type": "waterfall",
                "spans": waterfall_rows(&inputs.spans, tid).len(),
                "critical_path_spans": critical_path(&inputs.spans, tid).len()
            })
        }
        Panel::Stat { stat, service, .. } => {
            json!({"type": "stat", "stat": stat.name(), "service": service})
        }
        Panel::RedRow { data, .. } => {
            let red = red_by_service(&dataset(scene, &inputs.spans, data), window);
            let worst = red.iter().map(|r| r.error_ratio).fold(0.0f64, f64::max);
            json!({"type": "red_row", "services": red.len(), "worst_error_ratio": worst})
        }
        Panel::Sankey { data, residual } => {
            let flow = sankey_flow(&dataset(scene, &inputs.spans, data), residual);
            json!({
                "type": "sankey",
                "stages": flow.stages,
                "nodes": flow.nodes.len(),
                "links": flow.links.len(),
                "residual_spans": flow.residual_spans
            })
        }
        Panel::Flame { data, .. } => {
            let mut tree = flame_tree_for(scene, inputs, data);
            let summary =
                json!({"type": "flame", "total": tree.value, "top_frames": tree.children.len()});
            telorama_scene::dismantle_flame_tree(&mut tree);
            summary
        }
        Panel::Heatmap { data, .. } => {
            let grid = heatmap_grid(
                &dataset(scene, &inputs.spans, data),
                scene.context.range,
                bucket_edges(scene, data),
                24,
            );
            json!({
                "type": "heatmap",
                "busiest_cell": grid.max_count,
                "outside_window": grid.outside_window,
                "exemplar_trace": grid.exemplar.as_ref().map(|e| e.trace_id.clone())
            })
        }
        Panel::ServiceMap { data, .. } => {
            let graph = service_graph(&dataset(scene, &inputs.spans, data), window);
            json!({"type": "service_map", "nodes": graph.nodes.len(), "edges": graph.edges.len()})
        }
        Panel::AgentRun { .. } => match &inputs.run {
            Some(run) => json!({
                "type": "agent_run",
                "turns": run.turns.len(),
                "input_tokens": run.turns.iter().map(|t| t.input_tokens).sum::<u64>(),
                "output_tokens": run.turns.iter().map(|t| t.output_tokens).sum::<u64>(),
                "cost_usd": run.turns.iter().map(|t| t.cost_usd).sum::<f64>(),
                "priced": run.pricing.is_some()
            }),
            None => json!({"type": "agent_run", "turns": 0}),
        },
        Panel::AgentState { .. } => json!({
            "type": "agent_state",
            "entries": inputs.state.len(),
            "waiting": inputs.state.iter().filter(|e| e.waiting_on.is_some()).count()
        }),
    }
}

fn bucket_edges<'a>(scene: &'a Scene, data_id: &str) -> &'a [f64] {
    scene
        .data
        .iter()
        .find(|d| d.id == data_id)
        .and_then(|e| e.source.buckets.as_deref())
        .unwrap_or(&[])
}

/// Full derived detail for one panel, for the view's drill-down.
fn panel_detail(scene: &Scene, inputs: &Inputs, panel: &Panel) -> Value {
    let window = scene.context.range.seconds();
    match panel {
        Panel::Waterfall { data, .. } => {
            let tid = trace_id_of(scene, data).unwrap_or("");
            let set = dataset(scene, &inputs.spans, data);
            let origin = set.iter().map(|s| s.start_unix_nano).min().unwrap_or(0);
            let rows: Vec<Value> = waterfall_rows(&inputs.spans, tid)
                .iter()
                .map(|row| {
                    let span = set.iter().find(|s| s.span_id == row.span_id);
                    json!({
                        "span_id": row.span_id,
                        "name": span.map(|s| s.name.clone()),
                        "service": span.map(|s| s.service.clone()),
                        "depth": row.depth,
                        "start_offset_nanos": span.map(|s| s.start_unix_nano - origin),
                        "duration_nanos": span.map(|s| s.duration_nanos()),
                        "self_time_nanos": row.self_time_nanos,
                        "is_error": span.map(|s| s.is_error)
                    })
                })
                .collect();
            json!({
                "type": "waterfall",
                "trace_id": tid,
                "rows": rows,
                "critical_path": critical_path(&inputs.spans, tid)
            })
        }
        Panel::Stat {
            data,
            stat,
            service,
            ..
        } => {
            let mut set = dataset(scene, &inputs.spans, data);
            if let Some(want) = service {
                set.retain(|s| &s.service == want);
            }
            let durations: Vec<i64> = set.iter().map(|s| s.duration_nanos()).collect();
            let calls = set.len() as u64;
            let errors = set.iter().filter(|s| s.is_error).count() as u64;
            let mut out = json!({
                "type": "stat",
                "stat": stat.name(),
                "service": service,
                "spans": calls,
                "errors": errors
            });
            match stat {
                StatKind::Count => out["value"] = json!(calls),
                StatKind::Rate => {
                    out["value"] = json!(if window > 0.0 {
                        calls as f64 / window
                    } else {
                        0.0
                    })
                }
                StatKind::ErrorRatio => {
                    out["value"] = json!(if calls > 0 {
                        errors as f64 / calls as f64
                    } else {
                        0.0
                    })
                }
                StatKind::Last => {
                    out["value_nanos"] = json!(
                        set.iter()
                            .max_by_key(|s| s.end_unix_nano)
                            .map(|s| s.duration_nanos())
                    )
                }
                _ => {
                    out["value_nanos"] = json!(percentile_nanos(
                        &durations,
                        stat.percentile().unwrap_or(95)
                    ))
                }
            }
            out
        }
        Panel::RedRow { data, .. } => {
            let red = red_by_service(&dataset(scene, &inputs.spans, data), window);
            json!({
                "type": "red_row",
                "rows": red.iter().map(red_json).collect::<Vec<_>>()
            })
        }
        Panel::Sankey { data, residual } => {
            let flow = sankey_flow(&dataset(scene, &inputs.spans, data), residual);
            json!({
                "type": "sankey",
                "stages": flow.stages,
                "nodes": flow.nodes.iter().map(|n| json!({
                    "stage": n.stage,
                    "service": n.service,
                    "spans": n.value,
                    "outflow": n.outflow,
                    "residual": n.residual
                })).collect::<Vec<_>>(),
                "links": flow.links.iter().map(|l| json!({
                    "from_stage": l.from_stage,
                    "from": l.from_service,
                    "from_residual": l.from_residual,
                    "to_stage": l.to_stage,
                    "to": l.to_service,
                    "spans": l.value
                })).collect::<Vec<_>>(),
                "residual_spans": flow.residual_spans
            })
        }
        Panel::Flame { data, .. } => {
            let mut tree = flame_tree_for(scene, inputs, data);
            let frames: Vec<Value> = tree
                .children
                .iter()
                .map(|child| {
                    json!({
                        "name": child.name,
                        "value": child.value,
                        "share": if tree.value > 0 {
                            child.value as f64 / tree.value as f64
                        } else {
                            0.0
                        }
                    })
                })
                .collect();
            let detail = json!({"type": "flame", "total": tree.value, "frames": frames});
            telorama_scene::dismantle_flame_tree(&mut tree);
            detail
        }
        Panel::Heatmap { data, .. } => {
            let grid = heatmap_grid(
                &dataset(scene, &inputs.spans, data),
                scene.context.range,
                bucket_edges(scene, data),
                24,
            );
            json!({
                "type": "heatmap",
                "columns": grid.columns,
                "rows": grid.rows.iter().map(|row| json!({
                    "upper_edge_seconds": row.upper_edge_seconds,
                    "total": row.total,
                    "counts": row.counts
                })).collect::<Vec<_>>(),
                "outside_window": grid.outside_window,
                "exemplar": grid.exemplar.as_ref().map(|e| json!({
                    "trace_id": e.trace_id,
                    "span_id": e.span_id,
                    "service": e.service,
                    "name": e.name,
                    "duration_nanos": e.duration_nanos
                }))
            })
        }
        Panel::ServiceMap { data, .. } => {
            let graph = service_graph(&dataset(scene, &inputs.spans, data), window);
            json!({
                "type": "service_map",
                "nodes": graph.nodes.iter().map(red_json).collect::<Vec<_>>(),
                "edges": graph.edges.iter().map(|e| json!({
                    "from": e.from,
                    "to": e.to,
                    "calls": e.calls,
                    "errors": e.errors,
                    "p50_nanos": e.p50_nanos,
                    "p99_nanos": e.p99_nanos
                })).collect::<Vec<_>>()
            })
        }
        Panel::AgentRun { .. } => json!({
            "type": "agent_run",
            "run": inputs.run.as_ref().map(|r| serde_json::to_value(r).expect("the run model must serialize"))
        }),
        Panel::AgentState { .. } => json!({
            "type": "agent_state",
            "state": serde_json::to_value(&inputs.state).expect("the state model must serialize")
        }),
    }
}
