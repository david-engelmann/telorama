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
fn golden_wide_names_scene() {
    golden(
        "wide_names_scene.svg",
        "wide_names_scene.json",
        "wide-names-trace.json",
    );
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
fn unsupported_panel_type_is_a_specific_error() {
    // INV-20: a panel type with no renderer yet is refused
    // at parse time with the error naming the type at its
    // path, so nothing renders silently wrong.
    let text = workspace_file("tests/fixtures/scenes/checkout_scene.json")
        .replace("\"type\":\"waterfall\"", "\"type\":\"treemap\"");
    let errors = match telorama_scene::parse_scene(&text) {
        Ok(_) => panic!("a scene naming an unrenderable panel type must fail"),
        Err(errors) => errors,
    };
    assert_eq!(errors.len(), 1, "got: {errors:?}");
    assert_eq!(errors[0].path, "$.panels[0]");
    assert_eq!(errors[0].message, "unknown panel type \"treemap\"");
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
fn golden_agent_run_openinference_scene() {
    let scene = telorama_scene::parse_scene(&workspace_file(
        "tests/fixtures/scenes/agent_run_openinference_scene.json",
    ))
    .expect("openinference scene must validate");
    let spans = telorama_ingest::parse_otlp_traces(&workspace_file(
        "tests/fixtures/agents/openinference-session-spans.json",
    ))
    .expect("openinference fixture must parse");
    let run = telorama_ingest::parse_openinference_session(&spans).expect("spans must adapt");
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
    .expect("openinference scene must render");
    assert_golden("agent_run_openinference_scene.svg", &svg);
}

#[test]
fn golden_agent_run_genai_current_scene() {
    let scene = telorama_scene::parse_scene(&workspace_file(
        "tests/fixtures/scenes/agent_run_genai_current_scene.json",
    ))
    .expect("genai current scene must validate");
    let spans = telorama_ingest::parse_otlp_traces(&workspace_file(
        "tests/fixtures/agents/genai-session-current-spans.json",
    ))
    .expect("genai current fixture must parse");
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
    .expect("genai current scene must render");
    assert_golden("agent_run_genai_current_scene.svg", &svg);
}

#[test]
fn golden_agent_run_openinference_fable_scene() {
    let scene = telorama_scene::parse_scene(&workspace_file(
        "tests/fixtures/scenes/agent_run_openinference_fable_scene.json",
    ))
    .expect("openinference fable scene must validate");
    let spans = telorama_ingest::parse_otlp_traces(&workspace_file(
        "tests/fixtures/agents/openinference-session-fable-spans.json",
    ))
    .expect("openinference fable fixture must parse");
    let run = telorama_ingest::parse_openinference_session(&spans).expect("spans must adapt");
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
    .expect("openinference fable scene must render");
    assert_golden("agent_run_openinference_fable_scene.svg", &svg);
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

// ----------------------------------------------------------------
// Round 4 scale proof. The recipes mirror the ones in the scene
// crate's derive tests, fixed arithmetic so the shapes are the
// same on every run. Hash pins are the first hosted run's
// renders, and each ceiling is ten times that run's measured
// figure, per docs/SCALE-DESIGN.md and INV-69 to INV-71.

fn scale_span(id: &str, parent: Option<&str>, service: &str, start_ms: i64, end_ms: i64) -> Span {
    Span {
        trace_id: "t1".to_string(),
        span_id: id.to_string(),
        parent_span_id: parent.map(|p| p.to_string()),
        name: format!("op-{id}"),
        kind: telorama_scene::SpanKind::Server,
        start_unix_nano: start_ms * 1_000_000,
        end_unix_nano: end_ms * 1_000_000,
        is_error: false,
        service: service.to_string(),
        attributes: std::collections::BTreeMap::new(),
    }
}

fn scale_spans() -> Vec<Span> {
    const SERVICES: [&str; 8] = [
        "web", "api", "db", "cache", "queue", "worker", "auth", "search",
    ];
    let mut spans = Vec::with_capacity(10_000);
    for i in 0..1000 {
        let parent = (i > 0).then(|| format!("c{:04}", i - 1));
        spans.push(scale_span(
            &format!("c{i:04}"),
            parent.as_deref(),
            SERVICES[i % 8],
            i as i64,
            i as i64 + 5,
        ));
    }
    for h in 0..100 {
        let hub = format!("h{h:03}");
        spans.push(scale_span(&hub, None, SERVICES[h % 8], 0, 500));
        for j in 0..80 {
            spans.push(scale_span(
                &format!("k{h:03}-{j:02}"),
                Some(&hub),
                SERVICES[(h + j) % 8],
                j as i64,
                j as i64 + 3,
            ));
        }
    }
    for o in 0..900 {
        spans.push(scale_span(
            &format!("o{o:03}"),
            Some("absent"),
            SERVICES[o % 8],
            0,
            2,
        ));
    }
    assert_eq!(spans.len(), 10_000);
    spans
}

fn scale_scene(panel: &str) -> telorama_scene::Scene {
    let text = format!(
        "{{\"context\":{{\"filter\":{{}},\"provenance\":{{\"generated_by\":\"test\",\"question\":\"Does it hold at scale?\"}},\"range\":{{\"from_unix_ms\":0,\"to_unix_ms\":20000}}}},\"data\":[{{\"id\":\"trace\",\"sampled\":\"unknown\",\"source\":{{\"kind\":\"otlp_file\",\"trace_id\":\"t1\"}}}}],\"headline\":\"Scale proof\",\"narrative\":[{{\"panel\":0,\"text\":\"Scale proof.\"}}],\"panels\":[{panel}],\"scene\":\"0.1\",\"title\":\"Scale trace\"}}"
    );
    telorama_scene::parse_scene(&text).expect("scale scene must validate")
}

#[test]
fn scale_trace_renders_pin_by_hash() {
    let spans = scale_spans();
    let started = std::time::Instant::now();
    let mut hashes: Vec<String> = Vec::new();
    // The flame panel over a trace declares the tree's total
    // self time as its sample count, computed from the same
    // spans the panel renders.
    let flame_total = telorama_scene::flame_from_spans(&spans, "t1").value;
    let panels: Vec<String> = vec![
        "{\"data\":\"trace\",\"type\":\"waterfall\"}".to_string(),
        "{\"data\":\"trace\",\"type\":\"service_map\"}".to_string(),
        "{\"data\":\"trace\",\"residual\":\"other\",\"type\":\"sankey\"}".to_string(),
    ];
    for panel in &panels {
        let scene = scale_scene(panel);
        let svg = telorama_svg::render_scene(&scene, &spans)
            .unwrap_or_else(|e| panic!("a panel must render at scale: {e}"));
        assert!(svg.len() > 1_000, "a render is {} bytes", svg.len());
        hashes.push(telorama_scene::content_hash(svg.as_bytes()));
    }
    assert_eq!(
        hashes,
        vec![
            "74414c45532594f0".to_string(),
            "7c8f942722799c15".to_string(),
            "92b9037620a3a948".to_string()
        ]
    );
    // The service map hash moved in Round 8, and only that
    // one: the scale fixture's services form a ring along the
    // chain, so the old hash pinned the layering the pre-INV-82
    // relaxation inflated on a cyclic graph. The condensation
    // layers the ring at one layer, and this hash pins the
    // collapsed layout. The waterfall and sankey hashes, over
    // derivations whose inputs are trees and staged flows,
    // stand unchanged.
    // The flame over this trace refuses by name. The chain
    // makes the span tree a thousand levels deep, beyond
    // the renderer's 48-level flame limit, a standing bound
    // recorded in docs/SCALE-DESIGN.md. The wide fixture
    // below proves the flame render itself at this count.
    let deep_flame = scale_scene(&format!(
        "{{\"data\":\"trace\",\"min_samples\":1,\"sample_count\":{flame_total},\"sample_type\":\"cpu\",\"type\":\"flame\"}}"
    ));
    match telorama_svg::render_scene(&deep_flame, &spans) {
        Ok(_) => panic!("the deep flame must refuse"),
        Err(e) => assert!(
            e.to_string().contains("beyond the 48-level render limit"),
            "{e}"
        ),
    }
    // Measured at 306 milliseconds on the first hosted run
    // of the final code. The ceiling is ten times that,
    // rounded up to the nearest hundred milliseconds.
    assert!(
        started.elapsed() < std::time::Duration::from_millis(3100),
        "trace renders at scale took {:?}",
        started.elapsed()
    );
    eprintln!("scale trace renders {:?}", started.elapsed());
}

#[test]
fn cyclic_spans_at_scale_render_and_terminate() {
    // INV-82 at the scale suite's width, in two shapes. The
    // service ring: 10,000 spans with unique ids whose services
    // cycle an eight-service ring, so the layering faces a
    // cycle. The id ring: 10,000 spans whose ids cycle a
    // three-id ring, so the waterfall walk faces a cycle. The
    // shapes are separate because the service graph resolves
    // parents by span id, and repeated ids would scramble the
    // ring's edges.
    const SERVICES: [&str; 8] = [
        "web", "api", "db", "cache", "queue", "worker", "auth", "search",
    ];
    let mut ring_spans = Vec::with_capacity(10_000);
    for i in 0..10_000usize {
        let parent = (i > 0).then(|| format!("s{:05}", i - 1));
        ring_spans.push(scale_span(
            &format!("s{i:05}"),
            parent.as_deref(),
            SERVICES[i % 8],
            (i % 100) as i64,
            (i % 100) as i64 + 5,
        ));
    }
    let ring = ["a", "b", "c"];
    let mut id_spans = Vec::with_capacity(10_000);
    for i in 0..10_000usize {
        let parent = (i > 0).then(|| ring[(i - 1) % 3]);
        id_spans.push(scale_span(
            ring[i % 3],
            parent,
            SERVICES[i % 8],
            (i % 100) as i64,
            (i % 100) as i64 + 5,
        ));
    }
    let started = std::time::Instant::now();
    let map_scene = scale_scene("{\"data\":\"trace\",\"type\":\"service_map\"}");
    let svg = telorama_svg::render_scene(&map_scene, &ring_spans)
        .unwrap_or_else(|e| panic!("the service map must render a cyclic graph: {e}"));
    assert!(svg.len() > 1_000, "a render is {} bytes", svg.len());
    let fall_scene = scale_scene("{\"data\":\"trace\",\"type\":\"waterfall\"}");
    let svg = telorama_svg::render_scene(&fall_scene, &id_spans)
        .unwrap_or_else(|e| panic!("the waterfall must render a cyclic trace: {e}"));
    assert!(svg.len() > 1_000, "a render is {} bytes", svg.len());
    // The whole service ring is one strongly connected
    // component, so every service layers at zero.
    let graph = telorama_scene::service_graph(&ring_spans, 20.0);
    let layers = telorama_scene::service_layers(&graph);
    assert_eq!(layers.len(), 8);
    assert!(layers.values().all(|layer| *layer == 0));
    // The ceiling matches the acyclic trace renders' ten-times
    // bound; a walk that loops would never reach the assert.
    assert!(
        started.elapsed() < std::time::Duration::from_millis(3100),
        "cyclic renders at scale took {:?}",
        started.elapsed()
    );
    eprintln!("scale cyclic renders {:?}", started.elapsed());
}

#[test]
fn wide_trace_flame_renders_pin_by_hash() {
    // Ten thousand spans at depth two, one hundred roots
    // with ninety-nine children each, so the span-derived
    // flame renders at the same count the deep fixture
    // refuses. The recipe mirrors the scale family's
    // arithmetic, fixed so the shape is the same every run.
    const SERVICES: [&str; 8] = [
        "web", "api", "db", "cache", "queue", "worker", "auth", "search",
    ];
    let mut spans = Vec::with_capacity(10_000);
    for r in 0..100 {
        let root = format!("r{r:03}");
        spans.push(scale_span(&root, None, SERVICES[r % 8], 0, 500));
        for j in 0..99 {
            spans.push(scale_span(
                &format!("w{r:03}-{j:02}"),
                Some(&root),
                SERVICES[(r + j) % 8],
                j as i64,
                j as i64 + 3,
            ));
        }
    }
    assert_eq!(spans.len(), 10_000);
    let flame_total = telorama_scene::flame_from_spans(&spans, "t1").value;
    let scene = scale_scene(&format!(
        "{{\"data\":\"trace\",\"min_samples\":1,\"sample_count\":{flame_total},\"sample_type\":\"cpu\",\"type\":\"flame\"}}"
    ));
    let started = std::time::Instant::now();
    let svg = telorama_svg::render_scene(&scene, &spans).expect("the wide flame must render");
    eprintln!("scale wide flame render {:?}", started.elapsed());
    assert!(svg.len() > 1_000, "the render is {} bytes", svg.len());
    assert_eq!(
        telorama_scene::content_hash(svg.as_bytes()),
        "3337414d89172e42"
    );
}

#[test]
fn deep_flame_refusal_is_reached_at_extreme_depth() {
    // INV-86: the depth verdict is iterative, so a flame panel
    // over a chain far deeper than the call stack refuses by
    // name instead of overflowing inside the verdict itself.
    use std::collections::BTreeMap;
    use telorama_scene::SpanKind;
    let depth: i64 = 20_000;
    let mut spans = Vec::new();
    for i in 0..depth {
        spans.push(Span {
            trace_id: "t1".to_string(),
            span_id: format!("s{i}"),
            parent_span_id: (i > 0).then(|| format!("s{}", i - 1)),
            name: format!("op-{i}"),
            kind: SpanKind::Server,
            start_unix_nano: i * 1_000_000,
            end_unix_nano: (2 * depth - i) * 1_000_000,
            is_error: false,
            service: "svc".to_string(),
            attributes: BTreeMap::new(),
        });
    }
    let mut tree = telorama_scene::flame_from_spans(&spans, "t1");
    let flame_total = tree.value;
    // Dismantle iteratively, as the derive suite's deep-chain
    // test does: dropping this tree would recurse.
    let mut pending: Vec<telorama_scene::FlameNode> = std::mem::take(&mut tree.children);
    while let Some(mut node) = pending.pop() {
        pending.append(&mut node.children);
    }
    let scene = scale_scene(&format!(
        "{{\"data\":\"trace\",\"min_samples\":1,\"sample_count\":{flame_total},\"sample_type\":\"cpu\",\"type\":\"flame\"}}"
    ));
    match telorama_svg::render_scene(&scene, &spans) {
        Ok(_) => panic!("the deep flame must refuse"),
        Err(e) => assert!(
            e.to_string().contains("beyond the 48-level render limit"),
            "{e}"
        ),
    }
}

#[test]
fn profile_flame_renders_at_a_hundred_thousand_samples() {
    // The stack recipe mirrors the scene crate's profile test,
    // counts summing to exactly one hundred thousand.
    let mut stacks: Vec<telorama_scene::StackSample> = Vec::new();
    let mut assigned: u64 = 0;
    for i in 0..199 {
        let count = (i * 37 % 400) + 100;
        assigned += count;
        stacks.push(telorama_scene::StackSample {
            frames: vec![
                "root".to_string(),
                format!("svc-{}", i % 10),
                format!("fn-{i}"),
            ],
            count,
        });
    }
    stacks.push(telorama_scene::StackSample {
        frames: vec![
            "root".to_string(),
            "svc-0".to_string(),
            "fn-last".to_string(),
        ],
        count: 100_000 - assigned,
    });
    let text = "{\"context\":{\"filter\":{},\"provenance\":{\"generated_by\":\"test\",\"question\":\"Does it hold at scale?\"},\"range\":{\"from_unix_ms\":0,\"to_unix_ms\":20000}},\"data\":[{\"id\":\"profile\",\"sampled\":\"unsampled\",\"source\":{\"kind\":\"folded_profile\",\"query\":\"scale profile\"}}],\"headline\":\"Scale proof\",\"narrative\":[{\"panel\":0,\"text\":\"Scale proof.\"}],\"panels\":[{\"data\":\"profile\",\"min_samples\":100,\"sample_count\":100000,\"sample_type\":\"cpu\",\"type\":\"flame\"}],\"scene\":\"0.1\",\"title\":\"Scale profile\"}";
    let scene = telorama_scene::parse_scene(text).expect("flame scene must validate");
    let started = std::time::Instant::now();
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
    .expect("flame must render at scale");
    // Measured at 1.5 milliseconds on the first hosted run
    // of the final code. Ten times that is under the one
    // hundred millisecond floor INV-70 sets for the
    // smallest figures, so the floor is the ceiling.
    assert!(
        started.elapsed() < std::time::Duration::from_millis(100),
        "flame render at scale took {:?}",
        started.elapsed()
    );
    eprintln!("scale flame render {:?}", started.elapsed());
    assert!(svg.len() > 1_000, "flame render is {} bytes", svg.len());
    assert_eq!(
        telorama_scene::content_hash(svg.as_bytes()),
        "345ec9af1621aa72"
    );
}

#[test]
fn frame_stream_totals_hold_at_five_hundred_turns() {
    use telorama_scene::{AgentRun, AgentStep, AgentTurn, Coverage, ModelCall, Pricing};
    let call = |cost: f64| {
        AgentStep::ModelCall(ModelCall {
            model: "gpt-5.1".to_string(),
            start_unix_nano: 0,
            end_unix_nano: 1_000_000_000,
            ttft_nanos: None,
            attempts: 1,
            is_error: false,
            input_tokens: 100,
            output_tokens: 10,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
            cost_usd: cost,
        })
    };
    let mut turns = Vec::with_capacity(500);
    for k in 0..500i64 {
        turns.push(AgentTurn {
            prompt_id: format!("t{}", k + 1),
            start_unix_nano: k * 10_000_000_000,
            end_unix_nano: k * 10_000_000_000 + 9_000_000_000,
            steps: vec![call(0.50), call(0.50)],
            input_tokens: 200,
            output_tokens: 20,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
            cost_usd: 1.0,
            coverage: Coverage {
                log_events: 0,
                spans_seen: 2,
                spans_merged: 2,
            },
        });
    }
    let run = AgentRun {
        session_id: "sess-scale".to_string(),
        dialect: "genai".to_string(),
        turns,
        coverage: Coverage {
            log_events: 0,
            spans_seen: 1000,
            spans_merged: 1000,
        },
        pricing: Some(Pricing {
            table_as_of: "2026-11-23".to_string(),
            priced_models: vec!["gpt-5.1".to_string()],
            unpriced_models: vec![],
        }),
    };
    let started = std::time::Instant::now();
    let frames = telorama_scene::session_frames(&run, "test");
    assert_eq!(frames.len(), 500);
    let mut total_bytes = 0usize;
    let mut last_svg = String::new();
    for frame in &frames {
        let svg = telorama_svg::render_scene_with_data(
            &frame.scene,
            &telorama_svg::SceneData {
                spans: &[],
                stacks: &[],
                agent_run: Some(&frame.run),
                agent_state: &[],
                findings: &frame.findings,
            },
        )
        .expect("frame must render");
        total_bytes += svg.len();
        last_svg = svg;
    }
    // Measured at 2.5 seconds on the first hosted run of
    // the final code. The ceiling is ten times that,
    // rounded up.
    assert!(
        started.elapsed() < std::time::Duration::from_secs(26),
        "the rendered stream at 500 turns took {:?}",
        started.elapsed()
    );
    eprintln!("scale rendered stream {:?}", started.elapsed());
    // The first hosted run measured the stream at 104,428,401
    // bytes. The total is deterministic, so the ceiling sits
    // just above the measured figure and any growth fails
    // until it is re-measured and re-recorded, per INV-71.
    assert!(
        total_bytes <= 105_000_000,
        "the rendered stream totals {total_bytes} bytes"
    );
    assert_eq!(
        telorama_scene::content_hash(last_svg.as_bytes()),
        "49f00fd8a1ffe9eb"
    );
}

#[test]
fn frame_stream_at_five_thousand_turns() {
    // Round 14: the Round 4 session recipe at ten times the
    // turns. The stream's construction and its final frame are
    // what this test proves. The aggregate byte total of a
    // full render is recorded in DECISIONS from the
    // measurement run, because it grows with the square of
    // the turn count by construction (INV-71).
    use telorama_scene::{AgentRun, AgentStep, AgentTurn, Coverage, ModelCall, Pricing};
    let call = |cost: f64| {
        AgentStep::ModelCall(ModelCall {
            model: "gpt-5.1".to_string(),
            start_unix_nano: 0,
            end_unix_nano: 1_000_000_000,
            ttft_nanos: None,
            attempts: 1,
            is_error: false,
            input_tokens: 100,
            output_tokens: 10,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
            cost_usd: cost,
        })
    };
    let mut turns = Vec::with_capacity(5_000);
    for k in 0..5_000i64 {
        turns.push(AgentTurn {
            prompt_id: format!("t{}", k + 1),
            start_unix_nano: k * 10_000_000_000,
            end_unix_nano: k * 10_000_000_000 + 9_000_000_000,
            steps: vec![call(0.50), call(0.50)],
            input_tokens: 200,
            output_tokens: 20,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
            cost_usd: 1.0,
            coverage: Coverage {
                log_events: 0,
                spans_seen: 2,
                spans_merged: 2,
            },
        });
    }
    let run = AgentRun {
        session_id: "sess-long".to_string(),
        dialect: "genai".to_string(),
        turns,
        coverage: Coverage {
            log_events: 0,
            spans_seen: 10_000,
            spans_merged: 10_000,
        },
        pricing: Some(Pricing {
            table_as_of: "2026-11-23".to_string(),
            priced_models: vec!["gpt-5.1".to_string()],
            unpriced_models: vec![],
        }),
    };
    let started = std::time::Instant::now();
    let frames = telorama_scene::session_frames(&run, "test");
    let built = started.elapsed();
    eprintln!("long session stream build {:?}", built);
    assert_eq!(frames.len(), 5_000);
    let last = frames.last().expect("the stream has a final frame");
    assert_eq!(
        last.scene.headline,
        "After turn 5000: 1000000 input tokens, 100000 output tokens, $5000.0000 so far"
    );
    let svg = telorama_svg::render_scene_with_data(
        &last.scene,
        &telorama_svg::SceneData {
            spans: &[],
            stacks: &[],
            agent_run: Some(&last.run),
            agent_state: &[],
            findings: &last.findings,
        },
    )
    .expect("the final frame must render");
    let hash = telorama_scene::content_hash(svg.as_bytes());
    eprintln!("long session final frame hash {hash}");
    eprintln!("long session final frame bytes {}", svg.len());
    assert_eq!(hash, "c5ed954ad4390b9f");
    // A mid-stream spot render, so the proof is not only
    // about the stream's two ends.
    let mid = &frames[2_499];
    let mid_svg = telorama_svg::render_scene_with_data(
        &mid.scene,
        &telorama_svg::SceneData {
            spans: &[],
            stacks: &[],
            agent_run: Some(&mid.run),
            agent_state: &[],
            findings: &mid.findings,
        },
    )
    .expect("the mid-stream frame must render");
    assert_eq!(
        telorama_scene::content_hash(mid_svg.as_bytes()),
        "de36a42155768390"
    );
    // The first hosted measurement built the stream in
    // 13.06 seconds. The ceiling is ten times that,
    // rounded up, per INV-70.
    assert!(
        built < std::time::Duration::from_secs(131),
        "building the 5000-turn stream took {:?}",
        built
    );
}

#[test]
fn many_trace_sankey_renders_pin_by_hash() {
    // The thousand-trace fixture rendered whole, through a
    // sankey panel whose data names no single trace, so the
    // render covers every trace in the set. The recipe
    // mirrors the derive suite's, with each trace's service
    // rotation offset so the traces differ.
    const SERVICES: [&str; 8] = [
        "web", "api", "db", "cache", "queue", "worker", "auth", "search",
    ];
    let mut spans = Vec::with_capacity(10_000);
    for t in 0..1000 {
        for i in 0..10 {
            let mut s = scale_span(
                &format!("s{i}"),
                (i > 0).then(|| format!("s{}", i - 1)).as_deref(),
                SERVICES[(t + i) % 8],
                i as i64,
                i as i64 + 2,
            );
            s.trace_id = format!("mt{t:04}");
            spans.push(s);
        }
    }
    assert_eq!(spans.len(), 10_000);
    let text = "{\"context\":{\"filter\":{},\"provenance\":{\"generated_by\":\"test\",\"question\":\"Does it hold at scale?\"},\"range\":{\"from_unix_ms\":0,\"to_unix_ms\":20000}},\"data\":[{\"id\":\"all\",\"sampled\":\"unknown\",\"source\":{\"kind\":\"otlp_file\",\"query\":\"all traces\"}}],\"headline\":\"Scale proof\",\"narrative\":[{\"panel\":0,\"text\":\"Scale proof.\"}],\"panels\":[{\"data\":\"all\",\"residual\":\"other\",\"type\":\"sankey\"}],\"scene\":\"0.1\",\"title\":\"Scale traces\"}";
    let scene = telorama_scene::parse_scene(text).expect("scale scene must validate");
    let started = std::time::Instant::now();
    let svg = telorama_svg::render_scene(&scene, &spans).expect("the sankey must render at scale");
    eprintln!("scale many trace sankey render {:?}", started.elapsed());
    assert!(svg.len() > 1_000, "the render is {} bytes", svg.len());
    assert_eq!(
        telorama_scene::content_hash(svg.as_bytes()),
        "3bbc3b37c3a4caa1"
    );
}

#[test]
fn deep_subagent_nesting_renders_with_an_empty_label_budget() {
    // INV-73's zero budget guard, exercised through the one
    // caller that can still reach it. The agent step label
    // budget falls with nesting depth and saturates to zero
    // at depth twelve, where truncation must return an empty
    // label instead of failing.
    use telorama_scene::{AgentRun, AgentStep, AgentTurn, Coverage, ModelCall, SubagentRun};
    fn nest(depth: usize, leaf: AgentStep) -> AgentStep {
        if depth == 0 {
            leaf
        } else {
            AgentStep::Subagent(SubagentRun {
                agent_id: format!("agent-{depth}"),
                start_unix_nano: 0,
                end_unix_nano: 9_000_000_000,
                steps: vec![nest(depth - 1, leaf)],
            })
        }
    }
    let leaf = AgentStep::ModelCall(ModelCall {
        model: "gpt-5.1".to_string(),
        start_unix_nano: 0,
        end_unix_nano: 1_000_000_000,
        ttft_nanos: None,
        attempts: 1,
        is_error: false,
        input_tokens: 100,
        output_tokens: 10,
        cache_read_tokens: 0,
        cache_creation_tokens: 0,
        cost_usd: 0.5,
    });
    let run = AgentRun {
        session_id: "sess-deep".to_string(),
        dialect: "genai".to_string(),
        turns: vec![AgentTurn {
            prompt_id: "t1".to_string(),
            start_unix_nano: 0,
            end_unix_nano: 9_000_000_000,
            steps: vec![nest(12, leaf)],
            input_tokens: 100,
            output_tokens: 10,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
            cost_usd: 0.5,
            coverage: Coverage {
                log_events: 0,
                spans_seen: 1,
                spans_merged: 1,
            },
        }],
        coverage: Coverage {
            log_events: 0,
            spans_seen: 1,
            spans_merged: 1,
        },
        pricing: None,
    };
    let frames = telorama_scene::session_frames(&run, "test");
    let svg = telorama_svg::render_scene_with_data(
        &frames[0].scene,
        &telorama_svg::SceneData {
            spans: &[],
            stacks: &[],
            agent_run: Some(&frames[0].run),
            agent_state: &[],
            findings: &frames[0].findings,
        },
    )
    .expect("deep nesting must render");
    assert!(svg.contains("<svg"));
}

// ----------------------------------------------------------------
// Round 5, the dark theme. The light goldens above are the
// regression pin for the whole round. They must not move.

fn darkened(scene: &str, traces: &str) -> (telorama_scene::Scene, Vec<Span>) {
    let (mut scene_obj, spans) = load(scene, traces);
    scene_obj.theme = telorama_scene::Theme::Dark;
    (scene_obj, spans)
}

#[test]
fn golden_checkout_scene_dark() {
    let (scene_obj, spans) = darkened("checkout_scene.json", "otel-demo-checkout.json");
    let svg = telorama_svg::render_scene(&scene_obj, &spans).expect("dark scene must render");
    assert_golden("checkout_scene_dark.svg", &svg);
}

#[test]
fn golden_sankey_scene_dark() {
    let (scene_obj, spans) = darkened("sankey_scene.json", "otel-demo-checkout.json");
    let svg = telorama_svg::render_scene(&scene_obj, &spans).expect("dark scene must render");
    assert_golden("sankey_scene_dark.svg", &svg);
}

#[test]
fn golden_wide_names_scene_dark() {
    let (scene_obj, spans) = darkened("wide_names_scene.json", "wide-names-trace.json");
    let svg = telorama_svg::render_scene(&scene_obj, &spans).expect("dark scene must render");
    assert_golden("wide_names_scene_dark.svg", &svg);
}

#[test]
fn golden_agent_run_scene_dark() {
    let mut scene = telorama_scene::parse_scene(&workspace_file(
        "tests/fixtures/scenes/agent_run_scene.json",
    ))
    .expect("agent run scene must validate");
    scene.theme = telorama_scene::Theme::Dark;
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
    .expect("dark agent run scene must render");
    assert_golden("agent_run_scene_dark.svg", &svg);
}

#[test]
fn golden_agent_run_openinference_fable_scene_dark() {
    let mut scene = telorama_scene::parse_scene(&workspace_file(
        "tests/fixtures/scenes/agent_run_openinference_fable_scene.json",
    ))
    .expect("openinference fable scene must validate");
    scene.theme = telorama_scene::Theme::Dark;
    let spans = telorama_ingest::parse_otlp_traces(&workspace_file(
        "tests/fixtures/agents/openinference-session-fable-spans.json",
    ))
    .expect("openinference fable fixture must parse");
    let run = telorama_ingest::parse_openinference_session(&spans).expect("spans must adapt");
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
    .expect("dark openinference fable scene must render");
    assert_golden("agent_run_openinference_fable_scene_dark.svg", &svg);
}

#[test]
fn golden_agent_run_genai_current_scene_dark() {
    let mut scene = telorama_scene::parse_scene(&workspace_file(
        "tests/fixtures/scenes/agent_run_genai_current_scene.json",
    ))
    .expect("genai current scene must validate");
    scene.theme = telorama_scene::Theme::Dark;
    let spans = telorama_ingest::parse_otlp_traces(&workspace_file(
        "tests/fixtures/agents/genai-session-current-spans.json",
    ))
    .expect("genai current fixture must parse");
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
    .expect("dark genai current scene must render");
    assert_golden("agent_run_genai_current_scene_dark.svg", &svg);
}

#[test]
fn golden_flame_scene_dark() {
    // The flame root box is the one dark fill a label lands
    // on, so this golden pins the dark label rule as well
    // as the scene.
    let mut scene =
        telorama_scene::parse_scene(&workspace_file("tests/fixtures/scenes/flame_scene.json"))
            .expect("flame scene must validate");
    scene.theme = telorama_scene::Theme::Dark;
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
    .expect("dark flame scene must render");
    assert_golden("flame_scene_dark.svg", &svg);
}

#[test]
fn golden_incident_findings_scene_dark() {
    let spans = telorama_ingest::parse_otlp_traces(&workspace_file(
        "tests/fixtures/otel-demo-checkout.json",
    ))
    .expect("checkout fixture must parse");
    let scene = telorama_scene::incident_scene_with_theme(
        &spans,
        None,
        "agent",
        telorama_scene::Theme::Dark,
    )
    .expect("composition must succeed");
    let window_seconds =
        (scene.context.range.to_unix_ms - scene.context.range.from_unix_ms) as f64 / 1000.0;
    let findings = telorama_scene::investigate(&spans, None, window_seconds);
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
    .expect("dark incident scene must render");
    assert_golden("incident_findings_scene_dark.svg", &svg);
}

#[test]
fn golden_api_scene_dark() {
    let (scene_obj, spans) = darkened("api_scene.json", "single-service-api.json");
    let svg = telorama_svg::render_scene(&scene_obj, &spans).expect("dark scene must render");
    assert_golden("api_scene_dark.svg", &svg);
}

#[test]
fn golden_heatmap_scene_dark() {
    let (scene_obj, spans) = darkened("heatmap_scene.json", "otel-demo-checkout.json");
    let svg = telorama_svg::render_scene(&scene_obj, &spans).expect("dark scene must render");
    assert_golden("heatmap_scene_dark.svg", &svg);
}

#[test]
fn golden_service_map_scene_dark() {
    let (scene_obj, spans) = darkened("service_map_scene.json", "otel-demo-checkout.json");
    let svg = telorama_svg::render_scene(&scene_obj, &spans).expect("dark scene must render");
    assert_golden("service_map_scene_dark.svg", &svg);
}

#[test]
fn golden_agent_run_codex_scene_dark() {
    let mut scene = telorama_scene::parse_scene(&workspace_file(
        "tests/fixtures/scenes/agent_run_codex_scene.json",
    ))
    .expect("codex scene must validate");
    scene.theme = telorama_scene::Theme::Dark;
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
    .expect("dark codex scene must render");
    assert_golden("agent_run_codex_scene_dark.svg", &svg);
}

#[test]
fn golden_agent_run_codex_unpriced_scene_dark() {
    let mut scene = telorama_scene::parse_scene(&workspace_file(
        "tests/fixtures/scenes/agent_run_codex_unpriced_scene.json",
    ))
    .expect("unpriced scene must validate");
    scene.theme = telorama_scene::Theme::Dark;
    let logs = workspace_file("tests/fixtures/agents/codex-session-unpriced-logs.json");
    let run = telorama_ingest::parse_codex_session(&logs, &[]).expect("fixture must adapt");
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
    .expect("dark unpriced scene must render");
    assert_golden("agent_run_codex_unpriced_scene_dark.svg", &svg);
}

fn rel_luminance(hex: &str) -> f64 {
    let hex = hex.trim_start_matches('#');
    let channel = |i: usize| {
        let c = u32::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0) as f64 / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(0) + 0.7152 * channel(2) + 0.0722 * channel(4)
}

fn contrast(a: &str, b: &str) -> f64 {
    let (la, lb) = (rel_luminance(a), rel_luminance(b));
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

#[test]
fn dark_palette_pairs_clear_their_contrast_floors() {
    // INV-75. Every pair the dark renderer can emit, computed
    // from the palette constants themselves. Text clears four
    // and a half to one on both grounds, data marks clear
    // three to one, bar labels (the ground color on a palette
    // fill, the dark bar_text_fill rule) clear the text floor,
    // and the grid chrome clears its own quiet floor.
    let p = telorama_svg::DARK_PALETTE;
    let mut text_colors: Vec<&str> = vec![p.ink, p.muted, p.error, p.fallback, p.residual];
    text_colors.extend(p.service.iter());
    for color in &text_colors {
        for ground in [p.background, p.surface] {
            assert!(
                contrast(color, ground) >= 4.5,
                "text {color} on {ground} is {:.2}",
                contrast(color, ground)
            );
        }
    }
    let mut marks: Vec<&str> = vec![p.error, p.waiting, p.residual, p.fallback, p.heatmap_base];
    marks.extend(p.service.iter());
    for color in &marks {
        assert!(
            contrast(color, p.background) >= 3.0,
            "mark {color} on {} is {:.2}",
            p.background,
            contrast(color, p.background)
        );
    }
    for fill in p.service {
        assert!(
            contrast(p.background, fill) >= 4.5,
            "bar label on {fill} is {:.2}",
            contrast(p.background, fill)
        );
    }
    // The dark label rule picks the better of the ground and
    // the ink for the fill it lands on. Every fill the
    // renderer labels, the service fills and the flame
    // root's surface, must clear the text floor under the
    // better of the two.
    let mut labelled: Vec<&str> = vec![p.surface];
    labelled.extend(p.service.iter());
    for fill in labelled {
        let best = contrast(p.background, fill).max(contrast(p.ink, fill));
        assert!(best >= 4.5, "best label on {fill} is {best:.2}",);
    }
    assert!(
        contrast(p.grid, p.background) >= 1.4,
        "grid on {} is {:.2}",
        p.background,
        contrast(p.grid, p.background)
    );
}

fn strip_colors(svg: &str) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut chars = svg.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '#' {
            let mut hex = String::new();
            while hex.len() < 6 {
                match chars.peek() {
                    Some(n) if n.is_ascii_hexdigit() => hex.push(chars.next().unwrap()),
                    _ => break,
                }
            }
            if hex.len() == 6 {
                out.push_str("#XXXXXX");
            } else {
                out.push('#');
                out.push_str(&hex);
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[test]
fn themes_differ_only_in_color() {
    // INV-76. Geometry, text, and ordering are the same bytes
    // in both themes once color attributes are normalized out.
    for (scene, traces) in [
        ("checkout_scene.json", "otel-demo-checkout.json"),
        ("sankey_scene.json", "otel-demo-checkout.json"),
        ("heatmap_scene.json", "otel-demo-checkout.json"),
        ("service_map_scene.json", "otel-demo-checkout.json"),
    ] {
        let (light_scene, spans) = load(scene, traces);
        let mut dark_scene = light_scene.clone();
        dark_scene.theme = telorama_scene::Theme::Dark;
        let light = telorama_svg::render_scene(&light_scene, &spans).expect("light must render");
        let dark = telorama_svg::render_scene(&dark_scene, &spans).expect("dark must render");
        assert_ne!(light, dark, "{scene} must actually change under dark");
        assert_eq!(
            strip_colors(&light),
            strip_colors(&dark),
            "{scene} differs between themes by more than color"
        );
    }
    // The flagship scenes that render from data rather than
    // spans get the same proof through their own paths.
    let logs = workspace_file("tests/fixtures/agents/claude-code-session-logs.json");
    let session_spans = telorama_ingest::parse_otlp_traces(&workspace_file(
        "tests/fixtures/agents/claude-code-session-spans.json",
    ))
    .expect("session spans must parse");
    let run = telorama_ingest::parse_claude_code_session(&logs, &session_spans)
        .expect("session fixture must adapt");
    let mut agent_scene = telorama_scene::parse_scene(&workspace_file(
        "tests/fixtures/scenes/agent_run_scene.json",
    ))
    .expect("agent run scene must validate");
    let agent_data = telorama_svg::SceneData {
        spans: &[],
        stacks: &[],
        agent_run: Some(&run),
        agent_state: &[],
        findings: &[],
    };
    let light = telorama_svg::render_scene_with_data(&agent_scene, &agent_data)
        .expect("light agent run must render");
    agent_scene.theme = telorama_scene::Theme::Dark;
    let dark = telorama_svg::render_scene_with_data(&agent_scene, &agent_data)
        .expect("dark agent run must render");
    assert_ne!(light, dark);
    assert_eq!(strip_colors(&light), strip_colors(&dark));

    let spans = telorama_ingest::parse_otlp_traces(&workspace_file(
        "tests/fixtures/otel-demo-checkout.json",
    ))
    .expect("checkout fixture must parse");
    let light_scene =
        telorama_scene::incident_scene(&spans, None, "agent").expect("composition must succeed");
    let dark_scene = telorama_scene::incident_scene_with_theme(
        &spans,
        None,
        "agent",
        telorama_scene::Theme::Dark,
    )
    .expect("dark composition must succeed");
    let window_seconds = (light_scene.context.range.to_unix_ms
        - light_scene.context.range.from_unix_ms) as f64
        / 1000.0;
    let findings = telorama_scene::investigate(&spans, None, window_seconds);
    let incident_data = telorama_svg::SceneData {
        spans: &spans,
        stacks: &[],
        agent_run: None,
        agent_state: &[],
        findings: &findings,
    };
    let light = telorama_svg::render_scene_with_data(&light_scene, &incident_data)
        .expect("light incident must render");
    let dark = telorama_svg::render_scene_with_data(&dark_scene, &incident_data)
        .expect("dark incident must render");
    assert_ne!(light, dark);
    assert_eq!(strip_colors(&light), strip_colors(&dark));

    let mut state_scene = telorama_scene::parse_scene(&workspace_file(
        "tests/fixtures/scenes/agent_state_scene.json",
    ))
    .expect("agent state scene must validate");
    let state = telorama_ingest::parse_agent_state(&workspace_file(
        "tests/fixtures/agents/agent-state.json",
    ))
    .expect("state fixture must parse");
    let state_data = telorama_svg::SceneData {
        spans: &[],
        stacks: &[],
        agent_run: None,
        agent_state: &state,
        findings: &[],
    };
    let light = telorama_svg::render_scene_with_data(&state_scene, &state_data)
        .expect("light agent state must render");
    state_scene.theme = telorama_scene::Theme::Dark;
    let dark = telorama_svg::render_scene_with_data(&state_scene, &state_data)
        .expect("dark agent state must render");
    assert_ne!(light, dark);
    assert_eq!(strip_colors(&light), strip_colors(&dark));
}

#[test]
fn dark_frames_carry_the_theme() {
    // The frame builder's theme argument reaches every
    // frame's scene, and the light builder stays light.
    let logs = workspace_file("tests/fixtures/agents/claude-code-session-logs.json");
    let spans = telorama_ingest::parse_otlp_traces(&workspace_file(
        "tests/fixtures/agents/claude-code-session-spans.json",
    ))
    .expect("session spans must parse");
    let run = telorama_ingest::parse_claude_code_session(&logs, &spans)
        .expect("session fixture must adapt");
    let dark_frames =
        telorama_scene::session_frames_with_theme(&run, "agent", telorama_scene::Theme::Dark);
    assert!(!dark_frames.is_empty());
    assert!(
        dark_frames
            .iter()
            .all(|f| f.scene.theme == telorama_scene::Theme::Dark),
        "every dark frame scene must carry the dark theme"
    );
    let last = dark_frames.last().unwrap();
    let svg = telorama_svg::render_scene_with_data(
        &last.scene,
        &telorama_svg::SceneData {
            spans: &[],
            stacks: &[],
            agent_run: Some(&last.run),
            agent_state: &[],
            findings: &last.findings,
        },
    )
    .expect("dark frame must render");
    assert!(svg.contains("fill=\"#121212\""), "dark frame ground");
    let light_frames = telorama_scene::session_frames(&run, "agent");
    assert!(
        light_frames
            .iter()
            .all(|f| f.scene.theme == telorama_scene::Theme::Light),
        "the plain builder must stay light"
    );
}

#[test]
fn extreme_range_scene_renders() {
    // Round 15 (INV-88): a scene whose range spans most of
    // the i64 millisecond range renders a defined picture
    // through the waterfall and service map windows, where
    // the window arithmetic once overflowed.
    let spans = vec![
        scale_span("a", None, "web", 0, 5),
        scale_span("b", Some("a"), "api", 1, 4),
    ];
    for panel in [
        "{\"data\":\"trace\",\"type\":\"waterfall\"}",
        "{\"data\":\"trace\",\"type\":\"service_map\"}",
    ] {
        let mut scene = scale_scene(panel);
        scene.context.range.from_unix_ms = i64::MIN / 2;
        scene.context.range.to_unix_ms = i64::MAX / 2;
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
        .expect("an extreme range must still render");
        assert!(svg.contains("Scale trace"));
    }
    // The heatmap window carries the same subtraction in
    // the renderer, exercised through the committed scene.
    let mut heat =
        telorama_scene::parse_scene(&workspace_file("tests/fixtures/scenes/heatmap_scene.json"))
            .expect("heatmap scene must validate");
    heat.context.range.from_unix_ms = i64::MIN;
    heat.context.range.to_unix_ms = i64::MAX;
    let checkout = telorama_ingest::parse_otlp_traces(&workspace_file(
        "tests/fixtures/otel-demo-checkout.json",
    ))
    .expect("checkout fixture must parse");
    let svg = telorama_svg::render_scene_with_data(
        &heat,
        &telorama_svg::SceneData {
            spans: &checkout,
            stacks: &[],
            agent_run: None,
            agent_state: &[],
            findings: &[],
        },
    )
    .expect("an extreme range must still render a heatmap");
    assert!(svg.contains("Checkout latency heatmap"));
}

#[test]
fn rootless_cycle_names_itself_in_the_render_error() {
    // Round 15: a trace whose spans form a pure cycle has
    // spans, and the render error says so, instead of
    // claiming none were found.
    let spans = vec![
        scale_span("a", Some("b"), "web", 0, 5),
        scale_span("b", Some("a"), "api", 1, 4),
    ];
    let scene = scale_scene("{\"data\":\"trace\",\"type\":\"waterfall\"}");
    let err = telorama_svg::render_scene_with_data(
        &scene,
        &telorama_svg::SceneData {
            spans: &spans,
            stacks: &[],
            agent_run: None,
            agent_state: &[],
            findings: &[],
        },
    )
    .expect_err("a rootless cycle cannot render a waterfall");
    assert!(err.to_string().contains("parent chain cycles"), "{err}");
    let flame_scene = scale_scene(
        "{\"data\":\"trace\",\"min_samples\":1,\"sample_count\":1,\"sample_type\":\"cpu\",\"type\":\"flame\"}",
    );
    let flame_err = telorama_svg::render_scene_with_data(
        &flame_scene,
        &telorama_svg::SceneData {
            spans: &spans,
            stacks: &[],
            agent_run: None,
            agent_state: &[],
            findings: &[],
        },
    )
    .expect_err("a rootless cycle cannot render a flame graph");
    assert!(
        flame_err.to_string().contains("parent chain cycles"),
        "{flame_err}"
    );
}

#[test]
fn agent_run_renders_steps_at_time_extremes() {
    // Round 15 review repair (INV-88): a dialect session
    // whose spans sit at both ends of the i64 time range
    // renders its agent run, where the step detail's
    // duration subtraction once overflowed.
    let mut root = scale_span("r", None, "agent", 0, 1);
    root.start_unix_nano = i64::MIN;
    root.end_unix_nano = i64::MAX;
    root.attributes.insert(
        "gen_ai.operation.name".to_string(),
        "invoke_workflow".to_string(),
    );
    root.attributes.insert(
        "gen_ai.conversation.id".to_string(),
        "conv-extremes".to_string(),
    );
    let mut call = scale_span("m", Some("r"), "agent", 0, 1);
    call.start_unix_nano = i64::MIN;
    call.end_unix_nano = i64::MAX;
    call.attributes
        .insert("gen_ai.operation.name".to_string(), "chat".to_string());
    call.attributes
        .insert("gen_ai.request.model".to_string(), "gpt-5.1".to_string());
    let run = telorama_ingest::parse_genai_session(&[root, call]).expect("the session parses");
    let scene = telorama_scene::parse_scene(&workspace_file(
        "tests/fixtures/scenes/agent_run_scene.json",
    ))
    .expect("agent run scene must validate");
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
    .expect("an extreme session must still render");
    assert!(svg.contains("<svg"));
}

#[test]
fn waterfall_renders_both_copies_of_a_shared_id() {
    // Round 16 (INV-89): the client and server copies of
    // one call share an id, and the waterfall draws a bar
    // for each, labelled from its own copy.
    let spans = vec![
        scale_span("r", None, "web", 0, 100),
        scale_span("x", Some("r"), "client", 10, 40),
        scale_span("x", Some("r"), "server", 50, 90),
    ];
    let scene = scale_scene("{\"data\":\"trace\",\"type\":\"waterfall\"}");
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
    .expect("a shared-id trace must render");
    assert!(svg.contains("client · op-x"), "the client copy is drawn");
    assert!(svg.contains("server · op-x"), "the server copy is drawn");
}
