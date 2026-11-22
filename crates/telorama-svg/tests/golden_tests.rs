use std::path::PathBuf;
use telorama_scene::Span;

fn workspace_file(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

fn load(scene: &str, traces: &str) -> (telorama_scene::Scene, Vec<Span>) {
    let scene =
        telorama_scene::parse_scene(&workspace_file(&format!("tests/fixtures/scenes/{scene}")))
            .expect("fixture scene must validate");
    let spans =
        telorama_ingest::parse_otlp_traces(&workspace_file(&format!("tests/fixtures/{traces}")))
            .expect("fixture traces must parse");
    (scene, spans)
}

fn golden(name: &str, scene: &str, traces: &str) {
    let (scene_obj, spans) = load(scene, traces);
    let svg = telorama_svg::render_scene(&scene_obj, &spans).expect("scene must render");
    let golden_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(format!("tests/golden/{name}"));
    if std::env::var_os("UPDATE_GOLDENS").is_some() {
        std::fs::write(&golden_path, &svg).expect("golden must be writable");
        return;
    }
    let expected = std::fs::read_to_string(&golden_path)
        .unwrap_or_else(|e| panic!("cannot read golden {}: {e}", golden_path.display()));
    assert_eq!(
        svg, expected,
        "rendered SVG differs from the golden file; if the change is intended, rerun with UPDATE_GOLDENS=1 and review the diff"
    );
}

fn assert_golden(name: &str, svg: &str) {
    let golden_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(format!("tests/golden/{name}"));
    if std::env::var_os("UPDATE_GOLDENS").is_some() {
        std::fs::write(&golden_path, svg).expect("golden must be writable");
        return;
    }
    let expected = std::fs::read_to_string(&golden_path)
        .unwrap_or_else(|e| panic!("cannot read golden {}: {e}", golden_path.display()));
    assert_eq!(
        svg, expected,
        "rendered SVG differs from the golden file; if the change is intended, rerun with UPDATE_GOLDENS=1 and review the diff"
    );
}

#[test]
fn golden_checkout_scene() {
    golden(
        "checkout_scene.svg",
        "checkout_scene.json",
        "otel-demo-checkout.json",
    );
}

#[test]
fn golden_api_scene() {
    golden("api_scene.svg", "api_scene.json", "single-service-api.json");
}

#[test]
fn golden_heatmap_scene() {
    golden(
        "heatmap_scene.svg",
        "heatmap_scene.json",
        "otel-demo-checkout.json",
    );
}

#[test]
fn golden_service_map_scene() {
    golden(
        "service_map_scene.svg",
        "service_map_scene.json",
        "otel-demo-checkout.json",
    );
}

#[test]
fn golden_agent_run_scene() {
    let scene = telorama_scene::parse_scene(&workspace_file(
        "tests/fixtures/scenes/agent_run_scene.json",
    ))
    .expect("agent run scene must validate");
    let logs = workspace_file("tests/fixtures/agents/claude-code-session-logs.json");
    let spans = telorama_ingest::parse_otlp_traces(&workspace_file(
        "tests/fixtures/agents/claude-code-session-spans.json",
    ))
    .expect("session spans must parse");
    let run = telorama_ingest::parse_claude_code_session(&logs, &spans)
        .expect("session fixture must adapt");
    let svg = telorama_svg::render_scene_with_data(
        &scene,
        &telorama_svg::SceneData {
            spans: &[],
            stacks: &[],
            agent_run: Some(&run),
            agent_state: &[],
            findings: &[],
        },
    )
    .expect("agent run scene must render");
    assert_golden("agent_run_scene.svg", &svg);
}

#[test]
fn golden_flame_scene() {
    let scene =
        telorama_scene::parse_scene(&workspace_file("tests/fixtures/scenes/flame_scene.json"))
            .expect("flame scene must validate");
    let stacks = telorama_ingest::parse_folded_stacks(&workspace_file(
        "tests/fixtures/profiles/checkout-cpu.txt",
    ))
    .expect("profile fixture must parse");
    let svg = telorama_svg::render_scene_with_data(
        &scene,
        &telorama_svg::SceneData {
            spans: &[],
            stacks: &stacks,
            agent_run: None,
            agent_state: &[],
            findings: &[],
        },
    )
    .expect("flame scene must render");
    assert_golden("flame_scene.svg", &svg);
}

#[test]
fn rendering_is_byte_identical_across_runs_and_input_orders() {
    let (scene, spans) = load("checkout_scene.json", "otel-demo-checkout.json");
    let first = telorama_svg::render_scene(&scene, &spans).unwrap();
    let second = telorama_svg::render_scene(&scene, &spans).unwrap();
    assert_eq!(first, second);
    let mut reversed = spans.clone();
    reversed.reverse();
    let third = telorama_svg::render_scene(&scene, &reversed).unwrap();
    assert_eq!(first, third, "input order must not leak into the render");
}

#[test]
fn sankey_scene_matches_golden() {
    golden(
        "sankey_scene.svg",
        "sankey_scene.json",
        "otel-demo-checkout.json",
    );
    // The standing determinism pattern: input order must not leak
    // into the render.
    let (scene, spans) = load("sankey_scene.json", "otel-demo-checkout.json");
    let first = telorama_svg::render_scene(&scene, &spans).unwrap();
    let mut reversed = spans.clone();
    reversed.reverse();
    let second = telorama_svg::render_scene(&scene, &reversed).unwrap();
    assert_eq!(first, second, "input order must not leak into the render");
}

#[test]
fn sankey_orphans_are_named_under_the_diagram() {
    use std::collections::BTreeMap;
    use telorama_scene::SpanKind;
    let scene = telorama_scene::parse_scene(
        r#"{"context":{"filter":{"env":"prod"},"provenance":{"generated_by":"agent","question":"Where does this flow enter?"},"range":{"from_unix_ms":1794841200000,"to_unix_ms":1794841201000}},"data":[{"id":"trace","sampled":"unknown","source":{"kind":"otlp_file","trace_id":"t1"}}],"headline":"A flow with a gap in its evidence","narrative":[{"panel":0,"text":"One span names a parent the data does not contain."}],"panels":[{"data":"trace","residual":"other","type":"sankey"}],"scene":"0.1","title":"Orphan flow"}"#,
    )
    .expect("the orphan scene must validate");
    let span = |id: &str, parent: Option<&str>, service: &str| Span {
        trace_id: "t1".to_string(),
        span_id: id.to_string(),
        parent_span_id: parent.map(|p| p.to_string()),
        name: format!("op-{id}"),
        kind: SpanKind::Server,
        start_unix_nano: 0,
        end_unix_nano: 1_000_000,
        is_error: false,
        service: service.to_string(),
        attributes: BTreeMap::new(),
    };
    let spans = vec![
        span("root", None, "web"),
        span("child", Some("root"), "api"),
        span("orphan", Some("missing"), "db"),
    ];
    let svg = telorama_svg::render_scene(&scene, &spans).expect("the scene must render");
    assert!(
        svg.contains(
            "1 span entered through the other bucket because their parent chain could not stage them"
        ),
        "the residual entry must be named under the diagram"
    );
    assert!(
        svg.contains("other · 1 span"),
        "the residual bucket must be drawn as its own block"
    );
    // The same caption must stay true when the entry came from a
    // cycle, whose parents are in the data: the wording names the
    // unstageable chain, not a missing parent.
    let cycle = vec![span("x", Some("y"), "api"), span("y", Some("x"), "db")];
    let svg = telorama_svg::render_scene(&scene, &cycle).expect("the cycle scene must render");
    assert!(
        svg.contains(
            "2 spans entered through the other bucket because their parent chain could not stage them"
        ),
        "the caption must describe cycle entry truthfully"
    );
}

#[test]
fn service_colors_follow_sorted_name_order() {
    let (_, spans) = load("checkout_scene.json", "otel-demo-checkout.json");
    let colors = telorama_svg::service_colors(&spans);
    assert_eq!(colors.len(), 8, "the checkout fixture has eight services");
    let mut names: Vec<&str> = colors.keys().map(String::as_str).collect();
    names.sort_unstable();
    for (i, name) in names.iter().enumerate() {
        assert_eq!(
            colors[*name],
            telorama_svg::SERVICE_PALETTE[i],
            "service {name} must take palette slot {i} in sorted order"
        );
    }
    // The error channel is text as well as color: an error span's row in
    // the render carries the marker, so color is never the only signal.
    let (scene, spans) = load("checkout_scene.json", "otel-demo-checkout.json");
    let svg = telorama_svg::render_scene(&scene, &spans).unwrap();
    assert!(
        svg.contains("· error"),
        "error spans must carry a text marker in the render"
    );
}

#[test]
fn formatting_is_canonical() {
    assert_eq!(telorama_svg::fmt_num(960.0), "960");
    assert_eq!(telorama_svg::fmt_num(12.5), "12.5");
    assert_eq!(telorama_svg::fmt_num(0.25), "0.25");
    assert_eq!(telorama_svg::fmt_duration(500), "500 ns");
    assert_eq!(telorama_svg::fmt_duration(1_500), "1.5 µs");
    assert_eq!(telorama_svg::fmt_duration(420_000_000), "420 ms");
    assert_eq!(telorama_svg::fmt_duration(2_500_000_000), "2.5 s");
    assert_eq!(
        telorama_svg::fmt_time_ms(1_794_839_400_000),
        "2026-11-16T14:30:00Z"
    );
    assert_eq!(telorama_svg::fmt_time_ms(0), "1970-01-01T00:00:00Z");
}

#[test]
fn flame_declared_count_mismatch_is_a_render_error() {
    let text = workspace_file("tests/fixtures/scenes/flame_scene.json")
        .replace("\"sample_count\":970", "\"sample_count\":971");
    let scene = telorama_scene::parse_scene(&text).expect("scene must validate");
    let stacks = telorama_ingest::parse_folded_stacks(&workspace_file(
        "tests/fixtures/profiles/checkout-cpu.txt",
    ))
    .expect("profile fixture must parse");
    let err = telorama_svg::render_scene_with_data(
        &scene,
        &telorama_svg::SceneData {
            spans: &[],
            stacks: &stacks,
            agent_run: None,
            agent_state: &[],
            findings: &[],
        },
    )
    .unwrap_err();
    assert!(
        err.message
            .contains("declared sample count 971 does not match the profile data total 970"),
        "got: {}",
        err.message
    );
}

#[test]
fn flame_span_fallback_is_labelled_on_its_face() {
    let text = "{\"context\":{\"filter\":{\"env\":\"prod\"},\"provenance\":{\"generated_by\":\"test\",\"question\":\"Where does checkout time go?\"},\"range\":{\"from_unix_ms\":1794839400000,\"to_unix_ms\":1794839401000}},\"data\":[{\"id\":\"trace\",\"sampled\":\"unknown\",\"source\":{\"kind\":\"otlp_file\",\"trace_id\":\"4bf92f3577b34da6a3ce929d0e0e4736\"}}],\"headline\":\"Checkout flame derived from spans\",\"narrative\":[],\"panels\":[{\"data\":\"trace\",\"min_samples\":1,\"sample_count\":1,\"sample_type\":\"cpu\",\"type\":\"flame\"}],\"scene\":\"0.1\",\"title\":\"Checkout span flame\"}";
    let scene = telorama_scene::parse_scene(text).expect("fallback scene must validate");
    let spans = telorama_ingest::parse_otlp_traces(&workspace_file(
        "tests/fixtures/otel-demo-checkout.json",
    ))
    .expect("checkout fixture must parse");
    let svg = telorama_svg::render_scene(&scene, &spans).expect("fallback must render");
    assert!(
        svg.contains(
            "Derived from span self time, not sampled profiles. The trace holds 500 ms of self time across its spans."
        ),
        "the fallback must label itself on its face"
    );
}

/// Position of a service map node label in the rendered SVG.
fn node_pos(svg: &str, name: &str) -> (f64, f64) {
    let needle = format!(">{name}<");
    let at = svg.find(&needle).expect("node label must render");
    let start = svg[..at].rfind("<text").expect("label must be a text prim");
    let tag = &svg[start..at];
    let coord = |attr: &str| -> f64 {
        let key = format!("{attr}=\"");
        let from = tag.find(&key).expect("coord must exist") + key.len();
        tag[from..]
            .find('"')
            .map(|end| {
                tag[from..from + end]
                    .parse::<f64>()
                    .expect("coord is a number")
            })
            .expect("coord must terminate")
    };
    (coord("x"), coord("y"))
}

#[test]
fn service_map_layout_layers_follow_longest_path() {
    let (scene, spans) = load("service_map_scene.json", "otel-demo-checkout.json");
    let svg = telorama_svg::render_scene(&scene, &spans).expect("map must render");
    let (fx, _) = node_pos(&svg, "frontend");
    let (cx, _) = node_pos(&svg, "checkout");
    let (ux, _) = node_pos(&svg, "currency");
    assert!(fx < cx, "frontend is layer 0, left of checkout");
    assert!(cx < ux, "checkout is layer 1, left of its callees");
    for name in [
        "email",
        "inventory",
        "payment",
        "product-catalog",
        "shipping",
    ] {
        let (x, _) = node_pos(&svg, name);
        assert_eq!(x, ux, "{name} shares the layer 2 column");
    }
    // Within layer 2 every service has the same single predecessor, so
    // the barycenter ties and the order falls to names.
    let mut last_y = f64::MIN;
    for name in [
        "currency",
        "email",
        "inventory",
        "payment",
        "product-catalog",
        "shipping",
    ] {
        let (_, y) = node_pos(&svg, name);
        assert!(
            y > last_y,
            "{name} must sit below its predecessor in name order"
        );
        last_y = y;
    }
}

#[test]
fn golden_agent_run_codex_scene() {
    let scene = telorama_scene::parse_scene(&workspace_file(
        "tests/fixtures/scenes/agent_run_codex_scene.json",
    ))
    .expect("codex scene must validate");
    let logs = workspace_file("tests/fixtures/agents/codex-session-logs.json");
    let run = telorama_ingest::parse_codex_session(&logs, &[]).expect("codex fixture must adapt");
    let svg = telorama_svg::render_scene_with_data(
        &scene,
        &telorama_svg::SceneData {
            spans: &[],
            stacks: &[],
            agent_run: Some(&run),
            agent_state: &[],
            findings: &[],
        },
    )
    .expect("codex scene must render");
    assert_golden("agent_run_codex_scene.svg", &svg);
}

#[test]
fn html_export_wraps_the_svg_without_external_references() {
    let (scene, spans) = load("checkout_scene.json", "otel-demo-checkout.json");
    let data = telorama_svg::SceneData {
        spans: &spans,
        stacks: &[],
        agent_run: None,
        agent_state: &[],
        findings: &[],
    };
    let html = telorama_svg::render_scene_html(&scene, &data).expect("html must render");
    let again = telorama_svg::render_scene_html(&scene, &data).expect("html must render");
    assert_eq!(html, again, "the export is deterministic");
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<title>Checkout trace 4bf92f35</title>"));
    let svg = telorama_svg::render_scene(&scene, &spans).expect("svg must render");
    assert!(html.contains(&svg), "the document carries the SVG itself");
    assert!(
        !html.contains("<script"),
        "no scripts in a standalone export"
    );
    assert!(!html.contains("href="), "no external links");
    assert!(!html.contains("src="), "no external assets");
}

#[test]
fn golden_agent_run_codex_unpriced_scene() {
    let scene = telorama_scene::parse_scene(&workspace_file(
        "tests/fixtures/scenes/agent_run_codex_unpriced_scene.json",
    ))
    .expect("unpriced scene must validate");
    let logs = workspace_file("tests/fixtures/agents/codex-session-unpriced-logs.json");
    let run = telorama_ingest::parse_codex_session(&logs, &[]).expect("fixture must adapt");
    // The split at the run level: one model priced, one named unpriced,
    // and only the priced call's 0.008875 in the total.
    let pricing = run.pricing.as_ref().expect("codex runs are priced");
    assert_eq!(pricing.priced_models, vec!["gpt-5.1-codex".to_string()]);
    assert_eq!(pricing.unpriced_models, vec!["gpt-unknown-9".to_string()]);
    assert_eq!(run.turns[0].input_tokens, 5000);
    assert!((run.turns[0].cost_usd - 0.008875).abs() < 1e-9);
    let svg = telorama_svg::render_scene_with_data(
        &scene,
        &telorama_svg::SceneData {
            spans: &[],
            stacks: &[],
            agent_run: Some(&run),
            agent_state: &[],
            findings: &[],
        },
    )
    .expect("unpriced scene must render");
    assert_golden("agent_run_codex_unpriced_scene.svg", &svg);
}

#[test]
fn golden_agent_run_genai_scene() {
    let scene = telorama_scene::parse_scene(&workspace_file(
        "tests/fixtures/scenes/agent_run_genai_scene.json",
    ))
    .expect("genai scene must validate");
    let spans = telorama_ingest::parse_otlp_traces(&workspace_file(
        "tests/fixtures/agents/genai-session-spans.json",
    ))
    .expect("genai fixture must parse");
    let run = telorama_ingest::parse_genai_session(&spans).expect("spans must adapt");
    let svg = telorama_svg::render_scene_with_data(
        &scene,
        &telorama_svg::SceneData {
            spans: &spans,
            stacks: &[],
            agent_run: Some(&run),
            agent_state: &[],
            findings: &[],
        },
    )
    .expect("genai scene must render");
    assert_golden("agent_run_genai_scene.svg", &svg);
}

#[test]
fn golden_incident_checkout_scene() {
    let spans = telorama_ingest::parse_otlp_traces(&workspace_file(
        "tests/fixtures/otel-demo-checkout.json",
    ))
    .expect("checkout fixture must parse");
    let scene =
        telorama_scene::incident_scene(&spans, None, "agent").expect("composition must succeed");
    assert_eq!(scene.title, "Incident: email errors");
    let svg = telorama_svg::render_scene_with_data(
        &scene,
        &telorama_svg::SceneData {
            spans: &spans,
            stacks: &[],
            agent_run: None,
            agent_state: &[],
            findings: &[],
        },
    )
    .expect("incident scene must render");
    assert_golden("incident_checkout_scene.svg", &svg);
}

#[test]
fn golden_incident_findings_scene() {
    let spans = telorama_ingest::parse_otlp_traces(&workspace_file(
        "tests/fixtures/otel-demo-checkout.json",
    ))
    .expect("checkout fixture must parse");
    let scene =
        telorama_scene::incident_scene(&spans, None, "agent").expect("composition must succeed");
    let window_seconds =
        (scene.context.range.to_unix_ms - scene.context.range.from_unix_ms) as f64 / 1000.0;
    let findings = telorama_scene::investigate(&spans, None, window_seconds);
    assert_eq!(findings.len(), 2);
    assert_eq!(findings[0].kind, "errors");
    assert_eq!(findings[0].subject, "email");
    assert_eq!(findings[1].kind, "latency");
    let svg = telorama_svg::render_scene_with_data(
        &scene,
        &telorama_svg::SceneData {
            spans: &spans,
            stacks: &[],
            agent_run: None,
            agent_state: &[],
            findings: &findings,
        },
    )
    .expect("incident scene with findings must render");
    assert!(svg.contains("Look here first"));
    assert_golden("incident_findings_scene.svg", &svg);
}

#[test]
fn golden_agent_state_scene() {
    let scene = telorama_scene::parse_scene(&workspace_file(
        "tests/fixtures/scenes/agent_state_scene.json",
    ))
    .expect("agent state scene must validate");
    let state = telorama_ingest::parse_agent_state(&workspace_file(
        "tests/fixtures/agents/agent-state.json",
    ))
    .expect("state fixture must parse");
    let svg = telorama_svg::render_scene_with_data(
        &scene,
        &telorama_svg::SceneData {
            spans: &[],
            stacks: &[],
            agent_run: None,
            agent_state: &state,
            findings: &[],
        },
    )
    .expect("agent state scene must render");
    assert!(svg.contains("builder · running the workspace test suite"));
    assert!(svg.contains("Waiting on · the builder to push the final head"));
    assert_golden("agent_state_scene.svg", &svg);
}

#[test]
fn frames_from_a_truncated_export_equal_the_leading_frames() {
    let spans = telorama_ingest::parse_otlp_traces(&workspace_file(
        "tests/fixtures/agents/claude-code-session-spans.json",
    ))
    .expect("spans must parse");
    let full = telorama_ingest::parse_claude_code_session(
        &workspace_file("tests/fixtures/agents/claude-code-session-logs.json"),
        &spans,
    )
    .expect("full run must parse");
    let cut_spans: Vec<_> = spans
        .iter()
        .filter(|s| {
            s.start_unix_nano + (s.end_unix_nano - s.start_unix_nano) / 2
                <= full.turns[0].end_unix_nano
        })
        .cloned()
        .collect();
    assert_eq!(cut_spans.len(), 3, "turn 1 of the demo carries three spans");
    let cut = telorama_ingest::parse_claude_code_session(
        &workspace_file("tests/fixtures/agents/claude-code-session-turn1-logs.json"),
        &cut_spans,
    )
    .expect("truncated run must parse");
    let full_frames = telorama_scene::session_frames(&full, "test");
    let cut_frames = telorama_scene::session_frames(&cut, "test");
    assert_eq!(full_frames.len(), 2);
    assert_eq!(cut_frames.len(), 1);
    assert_eq!(cut_frames[0].run, full_frames[0].run);
    assert_eq!(cut_frames[0].scene, full_frames[0].scene);
    assert_eq!(cut_frames[0].findings, full_frames[0].findings);
    let render = |frame: &telorama_scene::Frame| {
        telorama_svg::render_scene_with_data(
            &frame.scene,
            &telorama_svg::SceneData {
                spans: &[],
                stacks: &[],
                agent_run: Some(&frame.run),
                agent_state: &[],
                findings: &frame.findings,
            },
        )
        .expect("frame must render")
    };
    assert_eq!(render(&cut_frames[0]), render(&full_frames[0]));
}
