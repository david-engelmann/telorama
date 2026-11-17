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
            "scene_panel_detail"
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
        html.contains("ui/notifications/tool-result"),
        "the view listens for results"
    );
    assert!(
        html.contains("telorama/svg"),
        "the view reads the svg from _meta"
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
    let golden = workspace_file("tests/golden/checkout_scene.svg");
    assert_eq!(
        result["structuredContent"]["svg_fnv1a64"],
        telorama_scene::content_hash(golden.as_bytes())
    );
    assert_eq!(
        result["structuredContent"]["panels"][0]["type"],
        "waterfall"
    );
    assert_eq!(result["structuredContent"]["panels"][0]["spans"], 9);
    assert_eq!(
        result["structuredContent"]["panels"][0]["critical_path_spans"],
        3
    );
    assert_eq!(
        result["_meta"]["telorama/svg"]
            .as_str()
            .expect("svg in _meta"),
        golden,
        "the view payload is the byte-identical golden render"
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
        "2026-11-16"
    );
    let text = result["content"][0]["text"].as_str().expect("text part");
    assert!(text.contains("$0.0289"), "got: {text}");
    assert!(text.contains("estimate"), "got: {text}");
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
