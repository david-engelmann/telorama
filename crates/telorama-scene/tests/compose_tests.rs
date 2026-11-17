use telorama_scene::{Coverage, Span, SpanKind, incident_scene, parse_scene};

fn span(
    trace: &str,
    id: &str,
    parent: Option<&str>,
    service: &str,
    start_ms: i64,
    end_ms: i64,
    error: bool,
) -> Span {
    Span {
        trace_id: trace.to_string(),
        span_id: id.to_string(),
        parent_span_id: parent.map(str::to_string),
        name: format!("{service} op"),
        kind: SpanKind::Server,
        start_unix_nano: start_ms * 1_000_000,
        end_unix_nano: end_ms * 1_000_000,
        is_error: error,
        service: service.to_string(),
        attributes: Default::default(),
    }
}

#[test]
fn incident_scene_names_the_failing_service_and_exemplar() {
    let spans = vec![
        span("t1", "01", None, "web", 0, 100, false),
        span("t1", "02", Some("01"), "pay", 10, 90, true),
        span("t2", "03", None, "web", 0, 50, false),
    ];
    let scene = incident_scene(&spans, None, "test").expect("composition must succeed");
    assert_eq!(scene.title, "Incident: pay errors");
    assert!(
        scene.headline.contains("pay is failing at 100.0%"),
        "{}",
        scene.headline
    );
    assert_eq!(scene.panels.len(), 4);
    assert_eq!(scene.data[1].source.trace_id.as_deref(), Some("t1"));
    let text = serde_json::to_string(&scene).expect("scene must serialize");
    parse_scene(&text).expect("the composed scene must pass the validator");
}

#[test]
fn incident_scene_without_errors_takes_the_slowest_trace() {
    let spans = vec![
        span("t1", "01", None, "web", 0, 40, false),
        span("t2", "02", None, "web", 0, 90, false),
    ];
    let scene = incident_scene(&spans, None, "test").expect("composition must succeed");
    assert_eq!(scene.title, "Incident: no failing service");
    assert_eq!(scene.data[1].source.trace_id.as_deref(), Some("t2"));
}

#[test]
fn incident_scene_attaches_a_run_as_the_last_panel() {
    let spans = vec![span("t1", "01", None, "web", 0, 40, false)];
    let run = telorama_scene::AgentRun {
        session_id: "sess-1".to_string(),
        dialect: "codex".to_string(),
        turns: Vec::new(),
        coverage: Coverage {
            log_events: 0,
            spans_seen: 0,
            spans_merged: 0,
        },
        pricing: None,
    };
    let scene = incident_scene(&spans, Some(&run), "test").expect("composition must succeed");
    assert_eq!(scene.panels.len(), 5);
    assert_eq!(scene.data[2].source.kind, "codex_session");
    let text = serde_json::to_string(&scene).expect("scene must serialize");
    parse_scene(&text).expect("the composed scene must pass the validator");
}

#[test]
fn incident_scene_refuses_empty_spans() {
    let err = incident_scene(&[], None, "test").unwrap_err();
    assert!(err.contains("no spans"), "{err}");
}
