//! Protocol tests for the telorama MCP server: the whole surface is
//! exercised through `handle_message`, the same pure function the
//! stdio binary delegates to.

use serde_json::{Value, json};

fn call(request: Value) -> Value {
    let line = serde_json::to_string(&request).expect("request must serialize");
    let response = telorama_mcp::handle_message(&line).expect("requests get replies");
    serde_json::from_str(&response).expect("the reply must be JSON")
}

fn call_tool(id: u64, name: &str, arguments: Value) -> Value {
    let reply = call(json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/call",
        "params": {"name": name, "arguments": arguments}
    }));
    reply["result"].clone()
}

fn workspace_file(rel: &str) -> String {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

#[test]
fn initialize_returns_server_info_and_capabilities() {
    let reply = call(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "test", "version": "0"}}
    }));
    assert_eq!(reply["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(reply["result"]["serverInfo"]["name"], "telorama");
    assert_eq!(
        reply["result"]["serverInfo"]["version"],
        env!("CARGO_PKG_VERSION")
    );
    assert!(reply["result"]["capabilities"]["tools"].is_object());
    assert!(reply["result"]["capabilities"]["resources"].is_object());
}

#[test]
fn notifications_get_no_reply() {
    let out =
        telorama_mcp::handle_message(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#);
    assert!(out.is_none());
}

#[test]
fn unknown_method_is_a_protocol_error() {
    let reply = call(json!({"jsonrpc": "2.0", "id": 9, "method": "nope/nope"}));
    assert_eq!(reply["error"]["code"], -32601);
}

#[test]
fn tools_list_links_the_view_and_hides_the_drilldown() {
    let reply = call(json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}));
    let tools = reply["result"]["tools"].as_array().expect("tools array");
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert_eq!(
        names,
        vec![
            "validate_scene",
            "inspect_traces",
            "render_scene",
            "session_run",
            "session_frames",
            "scene_panel_detail",
            "incident_scene"
        ]
    );
    let render = tools.iter().find(|t| t["name"] == "render_scene").unwrap();
    assert_eq!(render["_meta"]["ui"]["resourceUri"], telorama_mcp::VIEW_URI);
    let detail = tools
        .iter()
        .find(|t| t["name"] == "scene_panel_detail")
        .unwrap();
    assert_eq!(detail["_meta"]["ui"]["visibility"], json!(["app"]));
    let validate = tools
        .iter()
        .find(|t| t["name"] == "validate_scene")
        .unwrap();
    assert!(
        validate.get("_meta").is_none(),
        "model tools carry no ui meta"
    );
}

#[test]
fn resources_read_returns_the_app_view() {
    let reply = call(json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "resources/read",
        "params": {"uri": telorama_mcp::VIEW_URI}
    }));
    let contents = &reply["result"]["contents"][0];
    assert_eq!(contents["mimeType"], "text/html;profile=mcp-app");
    let html = contents["text"].as_str().expect("view text");
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("ui/initialize"), "the view opens the bridge");
    assert!(
        html.contains(&format!("version: \"{}\"", env!("CARGO_PKG_VERSION"))),
        "the view names the crate version"
    );
    assert!(
        html.contains("ui/notifications/tool-result"),
        "the view listens for results"
    );
    assert!(
        html.contains("telorama/svg"),
        "the view reads the svg from _meta"
    );
    assert!(
        html.contains("telorama/frames"),
        "the view reads the frame stream from _meta"
    );
    assert!(
        html.contains("id=\"stepper\""),
        "the view carries the stepper"
    );
    assert!(
        html.contains("delivered.length > 0"),
        "the stepper shows whenever a frame stream is delivered"
    );
    assert!(
        !html.contains("transition"),
        "frames replace: the view declares no transitions"
    );
}

#[test]
fn render_scene_returns_the_triple_contract() {
    let result = call_tool(
        4,
        "render_scene",
        json!({
            "scene": workspace_file("tests/fixtures/scenes/checkout_scene.json"),
            "traces": workspace_file("tests/fixtures/otel-demo-checkout.json")
        }),
    );
    assert_eq!(result["isError"], false);
    let text = result["content"][0]["text"].as_str().expect("text part");
    assert!(text.contains("Checkout trace 4bf92f35"), "got: {text}");
    assert!(text.contains("waterfall, stat, red_row"), "got: {text}");
    // Since M5 the tool render carries the investigation layer the
    // library render does not: the forcing function means the tool
    // never returns the lazy version. The panels stay identical.
    let svg = result["_meta"]["telorama/svg"]
        .as_str()
        .expect("svg in _meta");
    assert!(
        svg.contains("Look here first"),
        "the strip rides the tool render"
    );
    assert!(svg.contains("email carries 100.0% of the errors"), "{svg}");
    assert_eq!(
        result["structuredContent"]["svg_fnv1a64"],
        telorama_scene::content_hash(svg.as_bytes())
    );
    assert_eq!(
        result["structuredContent"]["investigation"][0]["kind"],
        "errors"
    );
    assert_eq!(
        result["structuredContent"]["investigation"][0]["subject"],
        "email"
    );
    assert!(text.contains("Look here first"), "got: {text}");
    assert_eq!(
        result["structuredContent"]["panels"][0]["type"],
        "waterfall"
    );
    assert_eq!(result["structuredContent"]["panels"][0]["spans"], 9);
    assert_eq!(
        result["structuredContent"]["panels"][0]["critical_path_spans"],
        3
    );
}

#[test]
fn validate_scene_reports_the_honesty_rules_as_tool_errors() {
    let bad = workspace_file("tests/fixtures/scenes/checkout_scene.json")
        .replace("\"p95\"", "\"average\"");
    let result = call_tool(5, "validate_scene", json!({"scene": bad}));
    assert_eq!(result["isError"], true);
    assert_eq!(result["structuredContent"]["valid"], false);
    let text = result["content"][0]["text"].as_str().expect("text part");
    assert!(text.contains("percentile"), "got: {text}");
}

#[test]
fn session_run_tool_prices_a_codex_session() {
    let result = call_tool(
        6,
        "session_run",
        json!({
            "logs": workspace_file("tests/fixtures/agents/codex-session-logs.json"),
            "dialect": "codex"
        }),
    );
    assert_eq!(result["isError"], false);
    assert_eq!(result["structuredContent"]["dialect"], "codex");
    assert_eq!(
        result["structuredContent"]["pricing"]["table_as_of"],
        "2026-11-23"
    );
    let text = result["content"][0]["text"].as_str().expect("text part");
    assert!(text.contains("$0.0289"), "got: {text}");
    assert!(text.contains("estimate"), "got: {text}");
    // The session payload carries the run's findings too (INV-57).
    let investigation = result["structuredContent"]["investigation"]
        .as_array()
        .expect("investigation list");
    assert!(
        investigation
            .iter()
            .any(|f| f["kind"] == "cost" && f["evidence"].is_array()),
        "the cost finding rides the session payload with its evidence"
    );
}

#[test]
fn panel_detail_drills_into_one_panel() {
    let result = call_tool(
        7,
        "scene_panel_detail",
        json!({
            "scene": workspace_file("tests/fixtures/scenes/checkout_scene.json"),
            "traces": workspace_file("tests/fixtures/otel-demo-checkout.json"),
            "panel_index": 0
        }),
    );
    assert_eq!(result["isError"], false);
    let detail = &result["structuredContent"];
    assert_eq!(detail["type"], "waterfall");
    assert_eq!(detail["rows"].as_array().expect("rows").len(), 9);
    assert_eq!(detail["critical_path"].as_array().expect("path").len(), 3);
    let out_of_range = call_tool(
        8,
        "scene_panel_detail",
        json!({
            "scene": workspace_file("tests/fixtures/scenes/checkout_scene.json"),
            "panel_index": 99
        }),
    );
    assert_eq!(out_of_range["isError"], true);
}

#[test]
fn sankey_panel_reports_its_flow_through_both_tools() {
    let scene = workspace_file("tests/fixtures/scenes/sankey_scene.json");
    let traces = workspace_file("tests/fixtures/otel-demo-checkout.json");
    let rendered = call_tool(
        12,
        "render_scene",
        json!({"scene": scene, "traces": traces}),
    );
    assert_eq!(rendered["isError"], false);
    let panels = rendered["structuredContent"]["panels"]
        .as_array()
        .expect("panels");
    let sankey = &panels[0];
    assert_eq!(sankey["type"], "sankey");
    assert_eq!(sankey["stages"], 4);
    assert_eq!(sankey["nodes"], 9);
    assert_eq!(sankey["links"], 8);
    assert_eq!(sankey["residual_spans"], 0);
    let detail = call_tool(
        13,
        "scene_panel_detail",
        json!({"scene": scene, "traces": traces, "panel_index": 0}),
    );
    assert_eq!(detail["isError"], false);
    let flow = &detail["structuredContent"];
    assert_eq!(flow["type"], "sankey");
    let nodes = flow["nodes"].as_array().expect("nodes");
    assert_eq!(nodes.len(), 9);
    let checkout = nodes
        .iter()
        .find(|n| n["stage"] == 1 && n["service"] == "checkout")
        .expect("the checkout node is reported");
    assert_eq!(checkout["spans"], 1);
    assert_eq!(
        checkout["outflow"], 6,
        "fan-out is reported as its own figure"
    );
    let links = flow["links"].as_array().expect("links");
    assert_eq!(links.len(), 8);
    assert!(links.iter().all(|l| l["from_residual"] == false));
}

#[test]
fn session_run_tool_maps_genai_spans() {
    let result = call_tool(
        10,
        "session_run",
        json!({"spans": workspace_file("tests/fixtures/agents/genai-session-spans.json"), "dialect": "genai"}),
    );
    assert_eq!(result["isError"], false);
    assert_eq!(result["structuredContent"]["dialect"], "genai");
    assert_eq!(result["structuredContent"]["session_id"], "conv-genai-demo");
    let text = result["content"][0]["text"].as_str().expect("text");
    assert!(text.contains("conv-genai-demo"), "{text}");
    assert!(text.contains("estimate"), "{text}");
}

#[test]
fn incident_scene_tool_composes_and_renders() {
    let result = call_tool(
        11,
        "incident_scene",
        json!({"traces": workspace_file("tests/fixtures/otel-demo-checkout.json")}),
    );
    assert_eq!(result["isError"], false);
    let text = result["content"][0]["text"].as_str().expect("text");
    assert!(text.contains("email is failing at 100.0%"), "{text}");
    assert_eq!(
        result["structuredContent"]["scene"]["title"],
        "Incident: email errors"
    );
    let svg = result["_meta"]["telorama/svg"]
        .as_str()
        .expect("view payload");
    assert!(svg.contains("<svg"), "the view payload is the render");
}

#[test]
fn panel_detail_reads_a_very_deep_profile() {
    // INV-87: the detail path builds the folded tree
    // iteratively and dismantles it after reading, so a
    // profile whose single stack runs fifty thousand
    // frames deep returns its frames instead of
    // overflowing on the way in or out. Fifty thousand
    // is the depth at which the drop itself is proven
    // to overflow, so the dismantle is pinned too.
    let frames: Vec<String> = (0..50_000).map(|i| format!("f{i}")).collect();
    let profile = format!("{} 970", frames.join(";"));
    let result = call_tool(
        15,
        "scene_panel_detail",
        json!({
            "scene": workspace_file("tests/fixtures/scenes/flame_scene.json"),
            "profile": profile,
            "panel_index": 0
        }),
    );
    assert_eq!(result["isError"], false);
    let detail = &result["structuredContent"];
    assert_eq!(detail["type"], "flame");
    assert_eq!(detail["total"], 970);
    let frames_out = detail["frames"].as_array().expect("frames");
    assert_eq!(frames_out.len(), 1);
    assert_eq!(frames_out[0]["name"], "f0");
    assert_eq!(frames_out[0]["value"], 970);
}

#[test]
fn incident_scene_tool_counts_a_duplicated_span_once() {
    // INV-86: the input assembly dedupes, so an export whose
    // error span arrives twice composes the same incident as
    // the clean export, the counts the CLI reports.
    let clean = workspace_file("tests/fixtures/otel-demo-checkout.json");
    let mut doc: Value = serde_json::from_str(&clean).expect("the fixture must parse");
    let mut duplicated_error = false;
    for resource in doc["resourceSpans"]
        .as_array_mut()
        .expect("the fixture carries resource spans")
    {
        for scope in resource["scopeSpans"]
            .as_array_mut()
            .expect("the fixture carries scope spans")
        {
            let spans = scope["spans"]
                .as_array_mut()
                .expect("the fixture carries a spans array");
            if let Some(error_span) = spans.iter().find(|s| s["status"]["code"] == 2).cloned() {
                spans.push(error_span);
                duplicated_error = true;
            }
        }
    }
    assert!(duplicated_error, "the fixture carries an error span");
    let duplicated = serde_json::to_string(&doc).expect("the export must serialize");
    let plain = call_tool(13, "incident_scene", json!({"traces": clean}));
    let doubled = call_tool(14, "incident_scene", json!({"traces": duplicated}));
    assert_eq!(doubled["isError"], false);
    assert_eq!(doubled["content"], plain["content"]);
    assert_eq!(doubled["structuredContent"], plain["structuredContent"]);
}

#[test]
fn render_scene_draws_the_agent_state_panel() {
    let result = call_tool(
        12,
        "render_scene",
        json!({
            "scene": workspace_file("tests/fixtures/scenes/agent_state_scene.json"),
            "agent_state": workspace_file("tests/fixtures/agents/agent-state.json")
        }),
    );
    assert_eq!(result["isError"], false);
    let svg = result["_meta"]["telorama/svg"]
        .as_str()
        .expect("svg in _meta");
    assert!(svg.contains("Agent state"), "the panel heading renders");
    assert!(
        svg.contains("builder · running the workspace test suite"),
        "the action line renders"
    );
    assert!(
        svg.contains("Why · the checkpoint rule needs an independent pass"),
        "the rationale renders"
    );
    assert!(
        svg.contains("Waiting on · the builder to push the final head"),
        "the waiting line renders"
    );
    assert_eq!(
        result["structuredContent"]["panels"][0]["entries"], 2,
        "the summary counts the entries"
    );
}

#[test]
fn session_frames_tool_returns_the_frame_stream() {
    let result = call_tool(
        13,
        "session_frames",
        json!({
            "logs": workspace_file("tests/fixtures/agents/claude-code-session-logs.json"),
            "spans": workspace_file("tests/fixtures/agents/claude-code-session-spans.json")
        }),
    );
    assert_eq!(result["isError"], false);
    let text = result["content"][0]["text"].as_str().expect("text");
    assert!(text.contains("Frame 1 of 2"), "{text}");
    assert!(text.contains("Frame 2 of 2"), "{text}");
    let frames = result["structuredContent"]["frames"]
        .as_array()
        .expect("frames");
    assert_eq!(frames.len(), 2);
    assert_eq!(frames[0]["turns"], 1);
    assert_eq!(frames[1]["turns"], 2);
    let meta_frames = result["_meta"]["telorama/frames"]
        .as_array()
        .expect("meta frames");
    assert_eq!(meta_frames.len(), 2);
    assert_eq!(
        result["_meta"]["telorama/svg"], meta_frames[1],
        "the view payload is the final frame"
    );
}
