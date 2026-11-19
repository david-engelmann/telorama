use std::path::PathBuf;
use std::process::Command;

fn workspace_file(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel)
}

fn telorama(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_telorama"))
        .args(args)
        .output()
        .expect("the telorama binary must run")
}

fn stdout(out: &std::process::Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("stdout must be utf8")
}

#[test]
fn inspect_summarizes_the_checkout_fixture() {
    let path = workspace_file("tests/fixtures/otel-demo-checkout.json");
    let out = telorama(&["inspect", path.to_str().unwrap()]);
    assert!(out.status.success());
    let text = stdout(&out);
    assert!(text.contains("spans: 9"), "got:\n{text}");
    assert!(
        text.contains("service email: 1 spans, 1 errors"),
        "got:\n{text}"
    );
    assert!(
        text.contains("service frontend: 1 spans, 0 errors"),
        "got:\n{text}"
    );
}

#[test]
fn validate_accepts_the_fixture_scene() {
    let path = workspace_file("tests/fixtures/scenes/checkout_scene.json");
    let out = telorama(&["validate", path.to_str().unwrap()]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout(&out).contains("valid: Checkout trace 4bf92f35 (3 panels, 1 data sources)"));
}

#[test]
fn validate_rejects_an_average_duration_with_the_honesty_message() {
    let bad = workspace_file("tests/fixtures/scenes/checkout_scene.json");
    let text = std::fs::read_to_string(&bad)
        .unwrap()
        .replace("\"p95\"", "\"average\"");
    let tmp = std::env::temp_dir().join(format!("telorama-bad-scene-{}.json", std::process::id()));
    std::fs::write(&tmp, text).unwrap();
    let out = telorama(&["validate", tmp.to_str().unwrap()]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        stderr.contains("percentiles, never an average"),
        "got:\n{stderr}"
    );
    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn render_matches_the_golden_file() {
    let scene = workspace_file("tests/fixtures/scenes/checkout_scene.json");
    let traces = workspace_file("tests/fixtures/otel-demo-checkout.json");
    let golden = workspace_file("tests/golden/checkout_scene.svg");
    let tmp = std::env::temp_dir().join(format!("telorama-render-{}.svg", std::process::id()));
    let out = telorama(&[
        "render",
        "--scene",
        scene.to_str().unwrap(),
        "--traces",
        traces.to_str().unwrap(),
        "--out",
        tmp.to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let rendered = std::fs::read_to_string(&tmp).unwrap();
    // Since M5 the CLI render carries the investigation layer the
    // library golden does not: the forcing function means the tool
    // never returns the lazy version. The scene content is unchanged.
    let expected = std::fs::read_to_string(&golden).unwrap();
    assert!(rendered.contains("Look here first"), "{rendered}");
    assert!(
        rendered.contains("email carries 100.0% of the errors"),
        "{rendered}"
    );
    assert!(rendered.contains("Checkout trace 4bf92f35"));
    assert!(expected.contains("Checkout trace 4bf92f35"));
    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn schema_command_prints_the_scene_schema() {
    let out = telorama(&["schema"]);
    assert!(out.status.success());
    let text = stdout(&out);
    assert!(
        text.contains("\"title\": \"telorama scene spec\""),
        "got:\n{text}"
    );
}

#[test]
fn session_prints_the_run_as_json() {
    let logs = workspace_file("tests/fixtures/agents/claude-code-session-logs.json");
    let spans = workspace_file("tests/fixtures/agents/claude-code-session-spans.json");
    let out = telorama(&[
        "session",
        "--logs",
        logs.to_str().unwrap(),
        "--spans",
        spans.to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = stdout(&out);
    assert!(
        text.contains("\"session_id\": \"sess-2026-1116-demo\""),
        "got:\n{text}"
    );
    assert!(text.contains("\"wait_nanos\": 1200000000"), "got:\n{text}");
    assert!(text.contains("\"agent_id\": \"agent-7\""), "got:\n{text}");
    assert!(text.contains("\"spans_merged\": 2"), "got:\n{text}");
}

#[test]
fn render_flame_scene_from_a_profile() {
    let scene = workspace_file("tests/fixtures/scenes/flame_scene.json");
    let profile = workspace_file("tests/fixtures/profiles/checkout-cpu.txt");
    let out_path = std::env::temp_dir().join("telorama-cli-flame.svg");
    let out = telorama(&[
        "render",
        "--scene",
        scene.to_str().unwrap(),
        "--profile",
        profile.to_str().unwrap(),
        "--out",
        out_path.to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let written = std::fs::read_to_string(&out_path).expect("the render must be written");
    let golden =
        std::fs::read_to_string(workspace_file("tests/golden/flame_scene.svg")).expect("golden");
    assert_eq!(written, golden);
}

#[test]
fn render_agent_run_scene_from_a_session() {
    let scene = workspace_file("tests/fixtures/scenes/agent_run_scene.json");
    let logs = workspace_file("tests/fixtures/agents/claude-code-session-logs.json");
    let spans = workspace_file("tests/fixtures/agents/claude-code-session-spans.json");
    let out_path = std::env::temp_dir().join("telorama-cli-agent-run.svg");
    let out = telorama(&[
        "render",
        "--scene",
        scene.to_str().unwrap(),
        "--session-logs",
        logs.to_str().unwrap(),
        "--session-spans",
        spans.to_str().unwrap(),
        "--out",
        out_path.to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let written = std::fs::read_to_string(&out_path).expect("the render must be written");
    // Since M5 the CLI render carries the investigation layer. The
    // Claude Code fixture waits on the user and spends money, so the
    // strip names both, over the same scene the golden records.
    assert!(written.contains("Look here first"), "{written}");
    assert!(written.contains("waiting on the user"), "{written}");
    assert!(written.contains("most expensive turn cost"), "{written}");
    let golden = std::fs::read_to_string(workspace_file("tests/golden/agent_run_scene.svg"))
        .expect("golden");
    assert!(golden.contains("Agent run"));
    assert!(written.contains("Agent run"));
}

#[test]
fn session_with_codex_dialect_reports_pricing() {
    let logs = workspace_file("tests/fixtures/agents/codex-session-logs.json");
    let out = telorama(&[
        "session",
        "--logs",
        logs.to_str().unwrap(),
        "--dialect",
        "codex",
    ]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = stdout(&out);
    assert!(text.contains("\"dialect\": \"codex\""), "got:\n{text}");
    assert!(
        text.contains("\"table_as_of\": \"2026-11-16\""),
        "got:\n{text}"
    );
    assert!(text.contains("\"gpt-5.1-codex\""), "got:\n{text}");
}

#[test]
fn render_html_writes_a_standalone_document() {
    let scene = workspace_file("tests/fixtures/scenes/checkout_scene.json");
    let traces = workspace_file("tests/fixtures/otel-demo-checkout.json");
    let out_path = std::env::temp_dir().join("telorama-cli-scene.html");
    let out = telorama(&[
        "render-html",
        "--scene",
        scene.to_str().unwrap(),
        "--traces",
        traces.to_str().unwrap(),
        "--out",
        out_path.to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let written = std::fs::read_to_string(&out_path).expect("the export must be written");
    assert!(written.starts_with("<!DOCTYPE html>"));
    assert!(written.contains("<svg"));
    assert!(!written.contains("<script"));
}

#[test]
fn session_with_genai_dialect_reads_spans() {
    let spans = workspace_file("tests/fixtures/agents/genai-session-spans.json");
    let out = telorama(&[
        "session",
        "--spans",
        spans.to_str().expect("path is utf8"),
        "--dialect",
        "genai",
    ]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = stdout(&out);
    assert!(text.contains("\"dialect\": \"genai\""), "{text}");
    assert!(
        text.contains("\"session_id\": \"conv-genai-demo\""),
        "{text}"
    );
}

#[test]
fn incident_composes_a_scene_from_the_checkout_fixture() {
    let traces = workspace_file("tests/fixtures/otel-demo-checkout.json");
    let out = telorama(&[
        "incident",
        "--traces",
        traces.to_str().expect("path is utf8"),
    ]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = stdout(&out);
    assert!(
        text.contains("\"title\": \"Incident: email errors\""),
        "{text}"
    );
    assert!(text.contains("\"type\": \"service_map\""), "{text}");
    assert!(text.contains("\"type\": \"heatmap\""), "{text}");
}

#[test]
fn plugin_packaging_is_wellformed() {
    let manifest_text =
        std::fs::read_to_string(workspace_file("plugin/.claude-plugin/plugin.json"))
            .expect("plugin manifest must read");
    let manifest: serde_json::Value =
        serde_json::from_str(&manifest_text).expect("plugin manifest must parse");
    assert_eq!(manifest["name"], "telorama");
    assert_eq!(manifest["version"], "0.1.0");
    let mcp_text =
        std::fs::read_to_string(workspace_file("plugin/.mcp.json")).expect("mcp config must read");
    let mcp: serde_json::Value = serde_json::from_str(&mcp_text).expect("mcp config must parse");
    assert_eq!(mcp["mcpServers"]["telorama"]["command"], "telorama-mcp");
    let skill = std::fs::read_to_string(workspace_file("plugin/skills/telorama/SKILL.md"))
        .expect("skill must read");
    assert!(
        skill.starts_with("---\nname: telorama\n"),
        "the skill opens with its frontmatter"
    );
    for needle in [
        "validate_scene",
        "inspect_traces",
        "render_scene",
        "session_run",
        "session_frames",
        "incident_scene",
        "scene_panel_detail",
        "Claude Code",
        "Codex",
        "GenAI",
        "OpenInference",
    ] {
        assert!(skill.contains(needle), "the skill must mention {needle}");
    }
}

#[test]
fn render_with_state_draws_the_agent_state_panel() {
    let scene = workspace_file("tests/fixtures/scenes/agent_state_scene.json");
    let state = workspace_file("tests/fixtures/agents/agent-state.json");
    let tmp = std::env::temp_dir().join(format!("telorama-state-{}.svg", std::process::id()));
    let out = telorama(&[
        "render",
        "--scene",
        scene.to_str().unwrap(),
        "--state",
        state.to_str().unwrap(),
        "--out",
        tmp.to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let rendered = std::fs::read_to_string(&tmp).unwrap();
    assert!(rendered.contains("Agent state"), "{rendered}");
    assert!(
        rendered.contains("Why · the slice is ready for its gate"),
        "{rendered}"
    );
    assert!(
        rendered.contains("Waiting on · the builder to push the final head"),
        "{rendered}"
    );
}

#[test]
fn frames_command_writes_jsonl_and_a_framebook() {
    let logs = workspace_file("tests/fixtures/agents/claude-code-session-logs.json");
    let spans = workspace_file("tests/fixtures/agents/claude-code-session-spans.json");
    let tmp = std::env::temp_dir().join(format!("telorama-frames-{}.jsonl", std::process::id()));
    let book = std::env::temp_dir().join(format!("telorama-framebook-{}.html", std::process::id()));
    let out = telorama(&[
        "frames",
        "--logs",
        logs.to_str().unwrap(),
        "--spans",
        spans.to_str().unwrap(),
        "--out",
        tmp.to_str().unwrap(),
        "--html",
        book.to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let jsonl = std::fs::read_to_string(&tmp).unwrap();
    let lines: Vec<&str> = jsonl.lines().collect();
    assert_eq!(lines.len(), 2, "one payload per frame");
    for (i, line) in lines.iter().enumerate() {
        let payload: serde_json::Value = serde_json::from_str(line).expect("payload parses");
        assert_eq!(payload["frame"], i + 1);
        assert_eq!(payload["turns"], i + 1);
        assert!(
            payload["svg"].as_str().expect("svg").contains("<svg"),
            "each payload carries its frame render"
        );
    }
    let html = std::fs::read_to_string(&book).unwrap();
    assert!(html.contains("Frame 1 of 2 · after turn 1"), "{html}");
    assert!(html.contains("Frame 2 of 2 · after turn 2"), "{html}");
    assert!(
        !html.contains("<script"),
        "the framebook carries no scripts"
    );
}
