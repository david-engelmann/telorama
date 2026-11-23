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
    assert_eq!(pricing.table_as_of, "2026-11-23");
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
    assert!(
        (turn1.cost_usd - 0.027125).abs() < 1e-9,
        "{}",
        turn1.cost_usd
    );
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
    assert!(
        (turn2.cost_usd - 0.001725).abs() < 1e-9,
        "{}",
        turn2.cost_usd
    );
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

#[test]
fn genai_session_maps_spans_to_a_run() {
    use telorama_scene::AgentStep;
    let spans = telorama_ingest::parse_otlp_traces(&fixture_text(
        "tests/fixtures/agents/genai-session-spans.json",
    ))
    .expect("genai fixture must parse");
    let run = telorama_ingest::parse_genai_session(&spans).expect("spans must adapt");
    assert_eq!(run.session_id, "conv-genai-demo");
    assert_eq!(run.dialect, "genai");
    assert_eq!(run.turns.len(), 1);
    assert_eq!(run.coverage.log_events, 0);
    assert_eq!(run.coverage.spans_seen, 6);
    assert_eq!(run.coverage.spans_merged, 6);
    let pricing = run.pricing.as_ref().expect("span runs are priced");
    assert_eq!(pricing.priced_models, vec!["gpt-5.1".to_string()]);
    assert!(pricing.unpriced_models.is_empty());

    let turn = &run.turns[0];
    assert_eq!(turn.steps.len(), 4);
    // Gross input 5000 with 2000 cached enters as 3000, and the
    // subagent's call counts in the turn totals, as in every dialect.
    assert_eq!(turn.input_tokens, 6200);
    assert_eq!(turn.output_tokens, 1500);
    assert_eq!(turn.cache_read_tokens, 3000);
    assert!((turn.cost_usd - 0.023125).abs() < 1e-9, "{}", turn.cost_usd);
    let AgentStep::ToolCall(tool) = &turn.steps[1] else {
        panic!("step 2 must be the web_search tool call");
    };
    assert_eq!(tool.name, "web_search");
    let AgentStep::Subagent(sub) = &turn.steps[2] else {
        panic!("step 3 must be the researcher subagent");
    };
    assert_eq!(sub.agent_id, "agent-researcher");
    assert_eq!(sub.steps.len(), 1);
}

#[test]
fn genai_session_reads_the_legacy_cache_attribute_names() {
    let spans = telorama_ingest::parse_otlp_traces(&fixture_text(
        "tests/fixtures/agents/genai-session-legacy-spans.json",
    ))
    .expect("legacy genai fixture must parse");
    let run = telorama_ingest::parse_genai_session(&spans).expect("spans must adapt");
    assert_eq!(run.session_id, "conv-genai-legacy");
    assert_eq!(run.turns.len(), 1);
    let turn = &run.turns[0];
    // The legacy spellings carry the same figures as the dotted
    // ones. Gross input enters net of cache read, and cache
    // creation is counted in the totals even though the pricing
    // table never prices it.
    assert_eq!(turn.input_tokens, 6200);
    assert_eq!(turn.output_tokens, 1500);
    assert_eq!(turn.cache_read_tokens, 3000);
    assert_eq!(turn.cache_creation_tokens, 400);
    assert!((turn.cost_usd - 0.023125).abs() < 1e-9, "{}", turn.cost_usd);
}

#[test]
fn genai_session_reads_the_cache_write_attribute_name() {
    let spans = telorama_ingest::parse_otlp_traces(&fixture_text(
        "tests/fixtures/agents/genai-session-cachewrite-spans.json",
    ))
    .expect("cache-write genai fixture must parse");
    let run = telorama_ingest::parse_genai_session(&spans).expect("spans must adapt");
    assert_eq!(run.session_id, "conv-genai-cachewrite");
    assert_eq!(run.turns.len(), 1);
    let turn = &run.turns[0];
    // The current spelling carries the same figures the legacy
    // fixture carries under its own. Before this read existed the
    // creation count silently landed at zero on current emitters.
    assert_eq!(turn.input_tokens, 6200);
    assert_eq!(turn.output_tokens, 1500);
    assert_eq!(turn.cache_read_tokens, 3000);
    assert_eq!(turn.cache_creation_tokens, 400);
    assert!((turn.cost_usd - 0.023125).abs() < 1e-9, "{}", turn.cost_usd);
}

#[test]
fn genai_session_stages_a_workflow_root() {
    use telorama_scene::AgentStep;
    let spans = telorama_ingest::parse_otlp_traces(&fixture_text(
        "tests/fixtures/agents/genai-session-workflow-spans.json",
    ))
    .expect("workflow genai fixture must parse");
    let run = telorama_ingest::parse_genai_session(&spans).expect("spans must adapt");
    assert_eq!(run.session_id, "conv-genai-workflow");
    assert_eq!(run.turns.len(), 1);
    let pricing = run.pricing.as_ref().expect("span runs are priced");
    assert_eq!(
        pricing.priced_models,
        vec!["claude-sonnet-5".to_string(), "gpt-5.2".to_string()]
    );
    assert!(pricing.unpriced_models.is_empty());
    let turn = &run.turns[0];
    // The workflow root is structure, so the turn's one step is the
    // triage agent, holding a chat call, a generate_content call
    // read as a model call, and a tool call.
    assert_eq!(turn.steps.len(), 1);
    let AgentStep::Subagent(sub) = &turn.steps[0] else {
        panic!("the turn's step must be the triage subagent");
    };
    assert_eq!(sub.agent_id, "agent-triage");
    assert_eq!(sub.steps.len(), 3);
    assert!(matches!(sub.steps[0], AgentStep::ModelCall(_)));
    assert!(matches!(sub.steps[1], AgentStep::ModelCall(_)));
    let AgentStep::ToolCall(tool) = &sub.steps[2] else {
        panic!("step 3 must be the doc_lookup tool call");
    };
    assert_eq!(tool.name, "doc_lookup");
    // Net input 3000 at sonnet prices plus 2000 at gpt-5.2 prices.
    assert_eq!(turn.input_tokens, 5000);
    assert_eq!(turn.output_tokens, 1100);
    assert_eq!(turn.cache_read_tokens, 1000);
    assert!((turn.cost_usd - 0.0227).abs() < 1e-9, "{}", turn.cost_usd);
}

#[test]
fn genai_session_refuses_two_conversations() {
    let spans = telorama_ingest::parse_otlp_traces(&fixture_text(
        "tests/fixtures/agents/genai-session-spans.json",
    ))
    .expect("genai fixture must parse");
    let mut two = spans.clone();
    for span in two.iter_mut().filter(|s| s.span_id == "00000000000000a4") {
        span.attributes.insert(
            "gen_ai.conversation.id".to_string(),
            "conv-other".to_string(),
        );
    }
    let err = telorama_ingest::parse_genai_session(&two).unwrap_err();
    assert!(err.message.contains("2 sessions"), "{err}");
}

#[test]
fn openinference_session_maps_span_kinds() {
    use telorama_scene::AgentStep;
    let spans = telorama_ingest::parse_otlp_traces(&fixture_text(
        "tests/fixtures/agents/openinference-session-spans.json",
    ))
    .expect("openinference fixture must parse");
    let run = telorama_ingest::parse_openinference_session(&spans).expect("spans must adapt");
    assert_eq!(run.session_id, "sess-oi-demo");
    assert_eq!(run.dialect, "openinference");
    assert_eq!(run.coverage.spans_seen, 5);
    let turn = &run.turns[0];
    // The CHAIN span is structure: its LLM child lifts into the turn.
    assert_eq!(turn.steps.len(), 3);
    assert_eq!(turn.input_tokens, 2900);
    assert_eq!(turn.output_tokens, 570);
    assert_eq!(turn.cache_read_tokens, 600);
    assert!((turn.cost_usd - 0.0094).abs() < 1e-9, "{}", turn.cost_usd);
    let AgentStep::ToolCall(tool) = &turn.steps[1] else {
        panic!("step 2 must be the calculator tool call");
    };
    assert_eq!(tool.name, "calculator");
}

#[test]
fn openinference_session_maps_a_reranker_step() {
    use telorama_scene::AgentStep;
    let spans = telorama_ingest::parse_otlp_traces(&fixture_text(
        "tests/fixtures/agents/openinference-session-reranker-spans.json",
    ))
    .expect("reranker fixture must parse");
    let run = telorama_ingest::parse_openinference_session(&spans).expect("spans must adapt");
    assert_eq!(run.session_id, "sess-oi-rerank");
    assert_eq!(run.turns.len(), 1);
    let turn = &run.turns[0];
    // The chain is structure. The retriever and the reranker are
    // both tool steps, the reranker named by its span name, and
    // the model call closes the turn. Before the reranker kind
    // was classified, its step vanished from the run.
    assert_eq!(turn.steps.len(), 3);
    let AgentStep::ToolCall(retriever) = &turn.steps[0] else {
        panic!("step 1 must be the retriever tool call");
    };
    assert_eq!(retriever.name, "vector search");
    let AgentStep::ToolCall(reranker) = &turn.steps[1] else {
        panic!("step 2 must be the reranker tool call");
    };
    assert_eq!(reranker.name, "rerank results");
    assert!(matches!(turn.steps[2], AgentStep::ModelCall(_)));
    assert_eq!(turn.input_tokens, 1500);
    assert_eq!(turn.output_tokens, 250);
}

#[test]
fn agent_state_parse_enforces_the_rationale_rule() {
    let ok = telorama_ingest::parse_agent_state(
        r#"[{"agent":"a","doing":"working","why":"it matters"}]"#,
    )
    .expect("a complete entry parses");
    assert_eq!(ok.len(), 1);
    assert_eq!(ok[0].agent, "a");
    let err = telorama_ingest::parse_agent_state(r#"[{"agent":"a","doing":"working","why":"  "}]"#)
        .expect_err("an empty rationale is refused");
    assert!(err.message.contains("empty why"), "{err}");
    let err = telorama_ingest::parse_agent_state("[]").expect_err("an empty report is refused");
    assert!(err.message.contains("no entries"), "{err}");
}

#[test]
fn turn_coverages_sum_to_the_run_coverage() {
    let logs = include_str!("../../../tests/fixtures/agents/claude-code-session-logs.json");
    let spans = parse_otlp_traces(include_str!(
        "../../../tests/fixtures/agents/claude-code-session-spans.json"
    ))
    .expect("spans must parse");
    let run = telorama_ingest::parse_claude_code_session(logs, &spans).expect("run must parse");
    assert_eq!(run.turns.len(), 2);
    let events: u64 = run.turns.iter().map(|t| t.coverage.log_events).sum();
    assert_eq!(events, run.coverage.log_events);
    let merged: u64 = run.turns.iter().map(|t| t.coverage.spans_merged).sum();
    assert_eq!(merged, run.coverage.spans_merged);
    let seen: u64 = run.turns.iter().map(|t| t.coverage.spans_seen).sum();
    assert_eq!(seen, run.coverage.spans_seen);
    assert_eq!(run.turns[0].coverage.spans_seen, 3);
    assert_eq!(run.turns[1].coverage.spans_seen, 3);
    assert!(run.turns[0].coverage.log_events > 0);
    assert!(run.turns[1].coverage.log_events > 0);
}

#[test]
fn claude_decision_only_prompt_carries_its_events_forward() {
    let logs = r#"{"resourceLogs":[{"scopeLogs":[{"logRecords":[
      {"attributes":[{"key":"event.name","value":{"stringValue":"claude_code.tool_decision"}},{"key":"session.id","value":{"stringValue":"s-carry"}},{"key":"prompt.id","value":{"stringValue":"p-0"}},{"key":"tool_name","value":{"stringValue":"Bash"}},{"key":"decision","value":{"stringValue":"accept"}}],"timeUnixNano":"1000000000"},
      {"attributes":[{"key":"event.name","value":{"stringValue":"claude_code.user_prompt"}},{"key":"session.id","value":{"stringValue":"s-carry"}},{"key":"prompt.id","value":{"stringValue":"p-1"}}],"timeUnixNano":"2000000000"},
      {"attributes":[{"key":"event.name","value":{"stringValue":"claude_code.api_request"}},{"key":"session.id","value":{"stringValue":"s-carry"}},{"key":"prompt.id","value":{"stringValue":"p-1"}},{"key":"model","value":{"stringValue":"claude-sonnet-4-5"}},{"key":"input_tokens","value":{"intValue":"10"}},{"key":"output_tokens","value":{"intValue":"2"}},{"key":"cost_usd","value":{"doubleValue":0.001}},{"key":"duration_ms","value":{"intValue":"100"}}],"timeUnixNano":"3000000000"}
    ]}]}]}"#;
    let run = telorama_ingest::parse_claude_code_session(logs, &[]).expect("run must parse");
    assert_eq!(run.turns.len(), 1, "a decision-only prompt is not a turn");
    assert_eq!(run.turns[0].prompt_id, "p-1");
    assert_eq!(
        run.turns[0].coverage.log_events, 3,
        "the dropped prompt's event carries forward"
    );
    assert_eq!(run.coverage.log_events, 3);
}

#[test]
fn claude_promptless_records_count_toward_the_first_turn() {
    let logs = r#"{"resourceLogs":[{"scopeLogs":[{"logRecords":[
      {"attributes":[{"key":"event.name","value":{"stringValue":"claude_code.api_request"}},{"key":"session.id","value":{"stringValue":"s-stray"}},{"key":"model","value":{"stringValue":"claude-sonnet-4-5"}},{"key":"input_tokens","value":{"intValue":"1"}},{"key":"output_tokens","value":{"intValue":"1"}},{"key":"cost_usd","value":{"doubleValue":0.001}},{"key":"duration_ms","value":{"intValue":"10"}}],"timeUnixNano":"1000000000"},
      {"attributes":[{"key":"event.name","value":{"stringValue":"claude_code.user_prompt"}},{"key":"session.id","value":{"stringValue":"s-stray"}},{"key":"prompt.id","value":{"stringValue":"p-1"}}],"timeUnixNano":"2000000000"},
      {"attributes":[{"key":"event.name","value":{"stringValue":"claude_code.api_request"}},{"key":"session.id","value":{"stringValue":"s-stray"}},{"key":"prompt.id","value":{"stringValue":"p-1"}},{"key":"model","value":{"stringValue":"claude-sonnet-4-5"}},{"key":"input_tokens","value":{"intValue":"10"}},{"key":"output_tokens","value":{"intValue":"2"}},{"key":"cost_usd","value":{"doubleValue":0.001}},{"key":"duration_ms","value":{"intValue":"100"}}],"timeUnixNano":"3000000000"}
    ]}]}]}"#;
    let run = telorama_ingest::parse_claude_code_session(logs, &[]).expect("run must parse");
    assert_eq!(run.turns.len(), 1);
    assert_eq!(
        run.turns[0].steps.len(),
        1,
        "a prompt-less record folds into no step"
    );
    assert_eq!(
        run.turns[0].coverage.log_events, 3,
        "the prompt-less record counts toward the first turn"
    );
    assert_eq!(run.coverage.log_events, 3);
}

#[test]
fn codex_pre_prompt_events_land_in_the_first_turns_coverage() {
    let logs = include_str!("../../../tests/fixtures/agents/codex-session-logs.json");
    let run = telorama_ingest::parse_codex_session(logs, &[]).expect("run must parse");
    assert_eq!(run.turns.len(), 2);
    assert_eq!(
        run.turns[0].coverage.log_events, 10,
        "nine folded events plus the conversation start that arrived before the first prompt"
    );
    assert_eq!(run.turns[1].coverage.log_events, 6);
    let sum: u64 = run.turns.iter().map(|t| t.coverage.log_events).sum();
    assert_eq!(sum, run.coverage.log_events);
}

#[test]
fn claude_blocked_child_merges_once_per_turn_however_many_steps_it_prices() {
    let logs = r#"{"resourceLogs":[{"scopeLogs":[{"logRecords":[
      {"attributes":[{"key":"event.name","value":{"stringValue":"claude_code.user_prompt"}},{"key":"session.id","value":{"stringValue":"s-fuse"}},{"key":"prompt.id","value":{"stringValue":"p-1"}}],"timeUnixNano":"1000000000"},
      {"attributes":[{"key":"event.name","value":{"stringValue":"claude_code.tool_result"}},{"key":"session.id","value":{"stringValue":"s-fuse"}},{"key":"prompt.id","value":{"stringValue":"p-1"}},{"key":"tool_name","value":{"stringValue":"Bash"}},{"key":"success","value":{"boolValue":true}},{"key":"duration_ms","value":{"intValue":"500"}}],"timeUnixNano":"10000000000"},
      {"attributes":[{"key":"event.name","value":{"stringValue":"claude_code.tool_result"}},{"key":"session.id","value":{"stringValue":"s-fuse"}},{"key":"prompt.id","value":{"stringValue":"p-1"}},{"key":"tool_name","value":{"stringValue":"Bash"}},{"key":"success","value":{"boolValue":true}},{"key":"duration_ms","value":{"intValue":"500"}}],"timeUnixNano":"20000000000"}
    ]}]}]}"#;
    let spans_text = r#"{"resourceSpans":[{"scopeSpans":[{"spans":[
      {"traceId":"f00df00df00df00df00df00df00df00d","spanId":"1111111111111111","name":"claude_code.tool","startTimeUnixNano":"9000000000","endTimeUnixNano":"21000000000"},
      {"traceId":"f00df00df00df00df00df00df00df00d","spanId":"2222222222222222","parentSpanId":"1111111111111111","name":"claude_code.tool.blocked_on_user","startTimeUnixNano":"9500000000","endTimeUnixNano":"14500000000"}
    ]}]}]}"#;
    let spans = telorama_ingest::parse_otlp_traces(spans_text).expect("spans must parse");
    let run = telorama_ingest::parse_claude_code_session(logs, &spans).expect("run must parse");
    assert_eq!(run.turns.len(), 1);
    assert_eq!(
        run.turns[0].coverage.spans_merged, 1,
        "one blocked child folded into the turn counts once, not once per step it priced"
    );
    assert_eq!(run.coverage.spans_merged, 1);
    let waits: Vec<i64> = run.turns[0]
        .steps
        .iter()
        .map(|s| match s {
            telorama_scene::AgentStep::ToolCall(t) => t.wait_nanos,
            _ => 0,
        })
        .collect();
    assert_eq!(waits, vec![5_000_000_000, 5_000_000_000]);
}

#[test]
fn claude_decision_only_export_folds_to_zero_coverage() {
    let logs = r#"{"resourceLogs":[{"scopeLogs":[{"logRecords":[
      {"attributes":[{"key":"event.name","value":{"stringValue":"claude_code.tool_decision"}},{"key":"session.id","value":{"stringValue":"s-zero"}},{"key":"prompt.id","value":{"stringValue":"p-0"}},{"key":"tool_name","value":{"stringValue":"Bash"}},{"key":"decision","value":{"stringValue":"accept"}}],"timeUnixNano":"1000000000"}
    ]}]}]}"#;
    let run = telorama_ingest::parse_claude_code_session(logs, &[]).expect("run must parse");
    assert_eq!(run.turns.len(), 0, "a decision-only export has no turns");
    assert_eq!(run.coverage.log_events, 0);
    assert_eq!(run.coverage.spans_seen, 0);
    assert_eq!(run.coverage.spans_merged, 0);
}

#[test]
fn agent_state_blank_optional_fields_normalize_away() {
    let entries = telorama_ingest::parse_agent_state(
        r#"[{"agent":"builder","doing":"building","why":"the gate needs it","waiting_on":"  ","next":""}]"#,
    )
    .expect("state must parse");
    assert_eq!(entries[0].waiting_on, None);
    assert_eq!(entries[0].next, None);
}
