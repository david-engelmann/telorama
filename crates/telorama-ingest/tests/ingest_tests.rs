use telorama_ingest::{parse_otlp_traces, summarize};
use telorama_scene::SpanKind;

const CHECKOUT: &str = include_str!("../../../tests/fixtures/otel-demo-checkout.json");
const API: &str = include_str!("../../../tests/fixtures/single-service-api.json");

#[test]
fn checkout_fixture_parses_to_nine_spans() {
    let spans = parse_otlp_traces(CHECKOUT).expect("checkout fixture must parse");
    assert_eq!(spans.len(), 9);
    let summary = summarize(&spans);
    assert_eq!(summary.trace_count, 1);
    assert_eq!(summary.services.len(), 8);
    let email = summary
        .services
        .iter()
        .find(|s| s.service == "email")
        .unwrap();
    assert_eq!(email.spans, 1);
    assert_eq!(email.errors, 1);
    let client = spans
        .iter()
        .find(|s| s.span_id == "0000000000000005")
        .unwrap();
    assert_eq!(client.kind, SpanKind::Client);
    assert_eq!(client.service, "checkout");
    let root = spans
        .iter()
        .find(|s| s.span_id == "0000000000000001")
        .unwrap();
    assert_eq!(root.parent_span_id, None);
    assert_eq!(root.duration_nanos(), 420_000_000);
    assert_eq!(
        root.attributes.get("http.route").map(|v| v.as_str()),
        Some("/checkout")
    );
    assert_eq!(
        root.attributes
            .get("resource.service.name")
            .map(|v| v.as_str()),
        Some("frontend")
    );
}

#[test]
fn api_fixture_parses() {
    let spans = parse_otlp_traces(API).expect("api fixture must parse");
    assert_eq!(spans.len(), 3);
    assert!(spans.iter().all(|s| s.service == "api"));
    assert!(spans.iter().all(|s| !s.is_error));
}

#[test]
fn json_lines_input_parses_each_document() {
    let text = format!("{API}\n{API}\n");
    let spans = parse_otlp_traces(&text).expect("jsonl must parse");
    assert_eq!(spans.len(), 6);
}

#[test]
fn empty_and_garbage_inputs_are_errors_not_empty_results() {
    assert!(parse_otlp_traces("").is_err());
    assert!(parse_otlp_traces("   \n").is_err());
    assert!(parse_otlp_traces("not json").is_err());
    assert!(
        parse_otlp_traces("{}").is_err(),
        "a document with no spans is an error"
    );
}

#[test]
fn missing_start_time_is_a_specific_error() {
    let doc = r#"{"resourceSpans":[{"resource":{"attributes":[{"key":"service.name","value":{"stringValue":"x"}}]},"scopeSpans":[{"spans":[{"traceId":"t","spanId":"s1","name":"op","kind":2,"endTimeUnixNano":"100"}]}]}]}"#;
    let err = parse_otlp_traces(doc).unwrap_err();
    assert!(
        err.message.contains("startTimeUnixNano"),
        "got: {}",
        err.message
    );
}

#[test]
fn parsing_is_deterministic() {
    let a = parse_otlp_traces(CHECKOUT).unwrap();
    let b = parse_otlp_traces(CHECKOUT).unwrap();
    assert_eq!(a, b);
}

#[test]
fn folded_stacks_parse_with_counts() {
    let text = "main;a;b 30\nmain;a 20\n\ngc 10\n";
    let stacks = telorama_ingest::parse_folded_stacks(text).expect("folded stacks must parse");
    assert_eq!(stacks.len(), 3);
    assert_eq!(stacks[0].frames, vec!["main", "a", "b"]);
    assert_eq!(stacks[0].count, 30);
    let total: u64 = stacks.iter().map(|s| s.count).sum();
    assert_eq!(total, 60);
}

#[test]
fn folded_stacks_reject_bad_lines() {
    let no_count = telorama_ingest::parse_folded_stacks("main;a;b").unwrap_err();
    assert!(no_count.message.contains("no sample count"), "{no_count}");
    let bad_count = telorama_ingest::parse_folded_stacks("main;a many").unwrap_err();
    assert!(bad_count.message.contains("non-numeric"), "{bad_count}");
    let zero = telorama_ingest::parse_folded_stacks("main;a 0").unwrap_err();
    assert!(zero.message.contains("zero"), "{zero}");
    let empty_frame = telorama_ingest::parse_folded_stacks("main;;a 5").unwrap_err();
    assert!(empty_frame.message.contains("empty frame"), "{empty_frame}");
    let empty = telorama_ingest::parse_folded_stacks("  \n").unwrap_err();
    assert!(empty.message.contains("empty"), "{empty}");
}

fn fixture_text(rel: &str) -> String {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

#[test]
fn claude_code_session_fuses_logs_and_spans() {
    use telorama_scene::AgentStep;
    let logs = fixture_text("tests/fixtures/agents/claude-code-session-logs.json");
    let spans = telorama_ingest::parse_otlp_traces(&fixture_text(
        "tests/fixtures/agents/claude-code-session-spans.json",
    ))
    .expect("span fixture must parse");
    let run = telorama_ingest::parse_claude_code_session(&logs, &spans)
        .expect("session fixture must adapt");
    assert_eq!(run.session_id, "sess-2026-1116-demo");
    assert_eq!(run.dialect, "claude_code");
    assert_eq!(run.turns.len(), 2);
    assert_eq!(run.coverage.log_events, 11);
    assert_eq!(run.coverage.spans_seen, 6);
    // The blocked_on_user span and the agent span entered the model.
    assert_eq!(run.coverage.spans_merged, 2);

    let turn1 = &run.turns[0];
    assert_eq!(turn1.input_tokens, 3000);
    assert_eq!(turn1.output_tokens, 750);
    assert_eq!(turn1.cache_read_tokens, 11200);
    assert!((turn1.cost_usd - 0.027).abs() < 1e-9, "{}", turn1.cost_usd);
    assert_eq!(turn1.steps.len(), 3);
    let AgentStep::ToolCall(bash) = &turn1.steps[1] else {
        panic!("turn 1 step 2 must be the Bash call");
    };
    assert_eq!(bash.name, "Bash");
    // The wait comes from the span, not the logs: 1.2 s of the 2 s.
    assert_eq!(bash.wait_nanos, 1_200_000_000);
    assert_eq!(bash.decision.as_deref(), Some("accept"));

    let turn2 = &run.turns[1];
    assert_eq!(turn2.input_tokens, 3000);
    assert_eq!(turn2.output_tokens, 580);
    assert!((turn2.cost_usd - 0.016).abs() < 1e-9, "{}", turn2.cost_usd);
    assert_eq!(turn2.steps.len(), 4);
    let AgentStep::ModelCall(failed) = &turn2.steps[0] else {
        panic!("turn 2 step 1 must be the failed model call");
    };
    assert!(failed.is_error);
    let AgentStep::ModelCall(retried) = &turn2.steps[1] else {
        panic!("turn 2 step 2 must be the retried model call");
    };
    assert_eq!(retried.attempts, 2);
    let AgentStep::Subagent(sub) = &turn2.steps[3] else {
        panic!("turn 2 step 4 must be the subagent");
    };
    assert_eq!(sub.agent_id, "agent-7");
    assert_eq!(sub.steps.len(), 1);
    let AgentStep::ModelCall(inner) = &sub.steps[0] else {
        panic!("the subagent's step must be its model call");
    };
    assert_eq!(inner.model, "claude-haiku-4-5");
}

#[test]
fn claude_code_session_needs_exactly_one_session() {
    let two = r#"{"resourceLogs":[{"scopeLogs":[{"logRecords":[
      {"timeUnixNano":"1000000000","attributes":[{"key":"event.name","value":{"stringValue":"claude_code.user_prompt"}},{"key":"session.id","value":{"stringValue":"s-1"}},{"key":"prompt.id","value":{"stringValue":"p-1"}}]},
      {"timeUnixNano":"2000000000","attributes":[{"key":"event.name","value":{"stringValue":"claude_code.user_prompt"}},{"key":"session.id","value":{"stringValue":"s-2"}},{"key":"prompt.id","value":{"stringValue":"p-1"}}]}
    ]}]}]}"#;
    let err = telorama_ingest::parse_claude_code_session(two, &[]).unwrap_err();
    assert!(err.message.contains("2 sessions"), "{err}");
    let none = telorama_ingest::parse_claude_code_session("{}", &[]).unwrap_err();
    assert!(none.message.contains("session.id"), "{none}");
}

#[test]
fn codex_session_prices_tokens_and_pairs_tools() {
    use telorama_scene::AgentStep;
    let logs = fixture_text("tests/fixtures/agents/codex-session-logs.json");
    let run = telorama_ingest::parse_codex_session(&logs, &[]).expect("codex fixture must adapt");
    assert_eq!(run.session_id, "conv-2026-1116-demo");
    assert_eq!(run.dialect, "codex");
    assert_eq!(run.turns.len(), 2);
    assert_eq!(run.coverage.log_events, 16);
    assert_eq!(run.coverage.spans_seen, 0);
    assert_eq!(run.coverage.spans_merged, 0);
    let pricing = run.pricing.as_ref().expect("codex runs are priced");
    assert_eq!(pricing.table_as_of, "2026-11-16");
    assert_eq!(
        pricing.priced_models,
        vec![
            "gpt-5.1-codex".to_string(),
            "gpt-5.1-codex-mini".to_string()
        ]
    );
    assert!(pricing.unpriced_models.is_empty());

    let turn1 = &run.turns[0];
    // Codex reports input gross: 8000 with 6000 cached, then 9500 with
    // 8000 cached. The model stores non-cache input only.
    assert_eq!(turn1.input_tokens, 3500);
    assert_eq!(turn1.output_tokens, 2100);
    assert_eq!(turn1.cache_read_tokens, 14000);
    assert!((turn1.cost_usd - 0.027125).abs() < 1e-9, "{}", turn1.cost_usd);
    assert_eq!(turn1.steps.len(), 3);
    let AgentStep::ModelCall(first) = &turn1.steps[0] else {
        panic!("turn 1 step 1 must be the first model call");
    };
    assert_eq!(first.ttft_nanos, Some(900_000_000));
    assert_eq!(first.input_tokens, 2000);
    assert_eq!(first.cache_read_tokens, 6000);
    let AgentStep::ToolCall(exec) = &turn1.steps[1] else {
        panic!("turn 1 step 2 must be the exec_command call");
    };
    assert_eq!(exec.name, "exec_command");
    assert_eq!(exec.decision.as_deref(), Some("approved"));
    // Codex merges no waits: the span side of this dialect is unproven.
    assert_eq!(exec.wait_nanos, 0);

    let turn2 = &run.turns[1];
    assert_eq!(turn2.input_tokens, 2000);
    assert_eq!(turn2.output_tokens, 600);
    assert!((turn2.cost_usd - 0.001725).abs() < 1e-9, "{}", turn2.cost_usd);
    let AgentStep::ModelCall(failed) = &turn2.steps[0] else {
        panic!("turn 2 step 1 must be the failed model call");
    };
    assert!(failed.is_error);
    let AgentStep::ModelCall(retried) = &turn2.steps[1] else {
        panic!("turn 2 step 2 must be the retried model call");
    };
    assert_eq!(retried.attempts, 2);
}

#[test]
fn codex_session_names_unpriced_models() {
    let logs = r#"{"resourceLogs":[{"scopeLogs":[{"logRecords":[
      {"timeUnixNano":"1000000000","attributes":[{"key":"event.name","value":{"stringValue":"codex.user_prompt"}},{"key":"conversation.id","value":{"stringValue":"c-1"}},{"key":"turn.id","value":{"stringValue":"t-1"}}]},
      {"timeUnixNano":"3000000000","attributes":[{"key":"event.name","value":{"stringValue":"codex.api_request"}},{"key":"conversation.id","value":{"stringValue":"c-1"}},{"key":"turn.id","value":{"stringValue":"t-1"}},{"key":"model","value":{"stringValue":"gpt-unknown-9"}},{"key":"duration_ms","value":{"intValue":"2000"}},{"key":"status","value":{"stringValue":"success"}}]},
      {"timeUnixNano":"3000000000","attributes":[{"key":"event.name","value":{"stringValue":"codex.sse_event"}},{"key":"conversation.id","value":{"stringValue":"c-1"}},{"key":"turn.id","value":{"stringValue":"t-1"}},{"key":"kind","value":{"stringValue":"response.completed"}},{"key":"input_tokens","value":{"intValue":"100"}},{"key":"output_tokens","value":{"intValue":"10"}}]}
    ]}]}]}"#;
    let run = telorama_ingest::parse_codex_session(logs, &[]).expect("must adapt");
    let pricing = run.pricing.as_ref().expect("codex runs are priced");
    assert_eq!(pricing.unpriced_models, vec!["gpt-unknown-9".to_string()]);
    assert!(pricing.priced_models.is_empty());
    // The tokens are in the run; only the cost is excluded.
    assert_eq!(run.turns[0].input_tokens, 100);
    assert_eq!(run.turns[0].cost_usd, 0.0);
}

#[test]
fn codex_session_needs_exactly_one_conversation() {
    let none = telorama_ingest::parse_codex_session("{}", &[]).unwrap_err();
    assert!(none.message.contains("conversation.id"), "{none}");
}
