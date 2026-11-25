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
    assert_eq!(pricing.table_as_of, "2026-11-25");
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
fn genai_session_terminates_on_a_cyclic_span_tree() {
    // INV-82 in the dialect builder: the fixture's fourth span
    // repeats the second span's id under the third, so the
    // children map cycles. The walk terminates because each
    // occurrence is visited once and the children of the
    // repeated id hang from its first copy alone (INV-89), so
    // the repeated span emits its own subagent step carrying
    // no children, and the turn counts all four occurrences.
    use telorama_scene::AgentStep;
    let spans = telorama_ingest::parse_otlp_traces(&fixture_text(
        "tests/fixtures/agents/genai-session-cyclic-spans.json",
    ))
    .expect("cyclic fixture must parse");
    assert_eq!(spans.len(), 4);
    let run = telorama_ingest::parse_genai_session(&spans).expect("spans must adapt");
    assert_eq!(run.turns.len(), 1);
    let turn = &run.turns[0];
    assert_eq!(turn.coverage.spans_seen, 4);
    assert_eq!(turn.steps.len(), 1);
    let AgentStep::Subagent(stage) = &turn.steps[0] else {
        panic!("the turn's one step must be the stage subagent");
    };
    assert_eq!(stage.steps.len(), 1);
    let AgentStep::Subagent(step) = &stage.steps[0] else {
        panic!("the stage's one step must be the step subagent");
    };
    assert_eq!(step.steps.len(), 1);
    let AgentStep::Subagent(repeated) = &step.steps[0] else {
        panic!("the step's one step must be the repeated span's subagent");
    };
    assert!(repeated.steps.is_empty());
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
fn genai_session_prices_the_current_model_lines() {
    // INV-93: every id this fixture names carries a row behind a
    // vendor-published price, so the whole run prices and the
    // total is the sum at the Round 44 rows.
    let spans = telorama_ingest::parse_otlp_traces(&fixture_text(
        "tests/fixtures/agents/genai-session-current-spans.json",
    ))
    .expect("current genai fixture must parse");
    let run = telorama_ingest::parse_genai_session(&spans).expect("spans must adapt");
    assert_eq!(run.session_id, "conv-genai-current");
    assert_eq!(run.dialect, "genai");
    assert_eq!(run.turns.len(), 1);
    assert_eq!(run.coverage.spans_seen, 6);
    let pricing = run.pricing.as_ref().expect("span runs are priced");
    assert_eq!(pricing.table_as_of, "2026-11-25");
    assert_eq!(
        pricing.priced_models,
        vec![
            "gpt-5.4".to_string(),
            "gpt-5.4-mini".to_string(),
            "gpt-5.4-nano".to_string(),
            "gpt-6-astra".to_string(),
            "gpt-6.1-sol".to_string()
        ]
    );
    assert!(pricing.unpriced_models.is_empty());

    let turn = &run.turns[0];
    assert_eq!(turn.steps.len(), 5);
    // Gross input enters net of cache reads, as in every dialect.
    assert_eq!(turn.input_tokens, 9000);
    assert_eq!(turn.output_tokens, 1850);
    assert_eq!(turn.cache_read_tokens, 4500);
    assert!(
        (turn.cost_usd - 0.0534125).abs() < 1e-9,
        "{}",
        turn.cost_usd
    );
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
fn genai_session_refuses_nesting_past_the_budget() {
    // INV-87: a generated chain of agent spans nesting past
    // MAX_AGENT_NESTING is refused with the named error, and
    // a chain whose steps nest exactly to the budget builds.
    use std::collections::BTreeMap;
    use telorama_scene::{MAX_AGENT_NESTING, Span};
    let build = |spans: usize| -> Vec<Span> {
        (0..spans)
            .map(|i| Span {
                trace_id: "t1".to_string(),
                span_id: format!("s{i}"),
                parent_span_id: (i > 0).then(|| format!("s{}", i - 1)),
                name: "agent".to_string(),
                kind: SpanKind::Internal,
                start_unix_nano: 0,
                end_unix_nano: 1_000,
                is_error: false,
                service: "svc".to_string(),
                attributes: BTreeMap::from([(
                    "gen_ai.operation.name".to_string(),
                    "invoke_agent".to_string(),
                )]),
            })
            .collect()
    };
    // The root span is the turn and never emits as a step, so
    // a chain of MAX + 2 spans nests its steps one past the
    // budget.
    let deep = build(MAX_AGENT_NESTING + 2);
    let err = telorama_ingest::parse_genai_session(&deep)
        .expect_err("a session past the budget must refuse");
    assert!(
        format!("{err}").contains("512"),
        "the refusal names the budget: {err}"
    );
    let at_budget = build(MAX_AGENT_NESTING + 1);
    let run = telorama_ingest::parse_genai_session(&at_budget)
        .expect("a session at the budget must build");
    assert_eq!(run.turns.len(), 1);
}

#[test]
fn genai_session_parses_ten_thousand_spans() {
    // Round 14: a generated session of one thousand turns,
    // ten spans each, through the GenAI adapter. Turn counts
    // and token totals are the assertions. The elapsed time
    // prints for the record and stands under a ceiling.
    use std::collections::BTreeMap;
    use telorama_scene::Span;
    let mut spans: Vec<Span> = Vec::with_capacity(10_000);
    let mk = |id: String, parent: Option<String>, op: &str, attrs: Vec<(&str, &str)>| -> Span {
        let mut attributes: BTreeMap<String, String> = attrs
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        attributes.insert("gen_ai.operation.name".to_string(), op.to_string());
        Span {
            trace_id: "t-long".to_string(),
            span_id: id,
            parent_span_id: parent,
            name: op.to_string(),
            kind: SpanKind::Internal,
            start_unix_nano: 0,
            end_unix_nano: 10_000,
            is_error: false,
            service: "svc".to_string(),
            attributes,
        }
    };
    for t in 0..1_000 {
        let root = format!("r{t}");
        spans.push(mk(root.clone(), None, "invoke_workflow", vec![]));
        let agent = format!("a{t}");
        spans.push(mk(
            agent.clone(),
            Some(root.clone()),
            "invoke_agent",
            vec![],
        ));
        for c in 0..4 {
            spans.push(mk(
                format!("c{t}-{c}"),
                Some(agent.clone()),
                "chat",
                vec![
                    ("gen_ai.request.model", "gpt-5.1"),
                    ("gen_ai.usage.input_tokens", "500"),
                    ("gen_ai.usage.output_tokens", "50"),
                ],
            ));
        }
        for x in 0..4 {
            spans.push(mk(
                format!("x{t}-{x}"),
                Some(root.clone()),
                "execute_tool",
                vec![("gen_ai.tool.name", "search")],
            ));
        }
    }
    assert_eq!(spans.len(), 10_000);
    let started = std::time::Instant::now();
    let run = telorama_ingest::parse_genai_session(&spans).expect("the session parses");
    let elapsed = started.elapsed();
    eprintln!("long genai session parse {:?}", elapsed);
    // The first hosted measurement parsed in 47
    // milliseconds. The ceiling is ten times that,
    // per INV-70.
    assert!(
        elapsed < std::time::Duration::from_millis(470),
        "the ten thousand span parse took {:?}",
        elapsed
    );
    assert_eq!(run.turns.len(), 1_000);
    let input: u64 = run.turns.iter().map(|t| t.input_tokens).sum();
    let output: u64 = run.turns.iter().map(|t| t.output_tokens).sum();
    assert_eq!(input, 2_000_000);
    assert_eq!(output, 200_000);
    // The honest emission count is the sum of the
    // per-turn coverage, which counts the spans the
    // walk actually reached. The run-level figure is
    // copied from the input length and proves nothing.
    let seen: u64 = run.turns.iter().map(|t| t.coverage.spans_seen).sum();
    assert_eq!(seen, 10_000);
}

#[test]
fn claude_session_parses_ten_thousand_events() {
    // Round 14: a generated Claude Code log of two thousand
    // prompts, five events each, in one OTLP document.
    use serde_json::json;
    fn record(at: i64, name: &str, prompt: &str, extra: &[(&str, serde_json::Value)]) -> String {
        let mut attributes = vec![
            json!({"key": "event.name", "value": {"stringValue": name}}),
            json!({"key": "session.id", "value": {"stringValue": "sess-long"}}),
            json!({"key": "prompt.id", "value": {"stringValue": prompt}}),
        ];
        for (key, value) in extra {
            attributes.push(json!({"key": key, "value": value}));
        }
        json!({
            "timeUnixNano": at.to_string(),
            "body": {"stringValue": name},
            "attributes": attributes,
        })
        .to_string()
    }
    let mut records: Vec<String> = Vec::with_capacity(10_000);
    for p in 0..2_000 {
        let base = 1_794_841_200_000_000_000i64 + (p as i64) * 10_000_000_000;
        let prompt = format!("p-{p}");
        records.push(record(base, "claude_code.user_prompt", &prompt, &[]));
        for e in 0..2 {
            let at = base + 1_000_000_000 + (e as i64) * 1_000_000_000;
            records.push(record(
                at,
                "claude_code.api_request",
                &prompt,
                &[
                    ("model", json!({"stringValue": "claude-sonnet-4-5"})),
                    ("input_tokens", json!({"intValue": "1000"})),
                    ("output_tokens", json!({"intValue": "100"})),
                    ("cost_usd", json!({"doubleValue": 0.01})),
                    ("duration_ms", json!({"intValue": "900"})),
                ],
            ));
        }
        records.push(record(
            base + 4_000_000_000,
            "claude_code.tool_decision",
            &prompt,
            &[],
        ));
        records.push(record(
            base + 5_000_000_000,
            "claude_code.tool_result",
            &prompt,
            &[],
        ));
    }
    assert_eq!(records.len(), 10_000);
    let events = format!(
        "{{\"resourceLogs\":[{{\"scopeLogs\":[{{\"logRecords\":[{}]}}]}}]}}",
        records.join(",")
    );
    let started = std::time::Instant::now();
    let run = telorama_ingest::parse_claude_code_session(&events, &[]).expect("the session parses");
    let elapsed = started.elapsed();
    eprintln!("long claude session parse {:?}", elapsed);
    // The first hosted measurement parsed in 176
    // milliseconds. The ceiling is ten times that,
    // per INV-70.
    assert!(
        elapsed < std::time::Duration::from_millis(1_760),
        "the ten thousand event parse took {:?}",
        elapsed
    );
    assert_eq!(run.turns.len(), 2_000);
    assert_eq!(run.coverage.log_events, 10_000);
    let input: u64 = run.turns.iter().map(|t| t.input_tokens).sum();
    let output: u64 = run.turns.iter().map(|t| t.output_tokens).sum();
    assert_eq!(input, 4_000_000);
    assert_eq!(output, 400_000);
}

#[test]
fn claude_session_refuses_nesting_past_the_budget() {
    // INV-87: the Claude adapter enforces the same budget
    // at its wrap. Generated agent spans nest their windows
    // around one model step, smallest first, so each wrap
    // deepens the run by one level.
    use std::collections::BTreeMap;
    use telorama_scene::{MAX_AGENT_NESTING, Span};
    let logs = concat!(
        "{\"resourceLogs\":[{\"resource\":{\"attributes\":[{\"key\":\"service.name\",",
        "\"value\":{\"stringValue\":\"claude-code\"}}]},\"scopeLogs\":[{\"logRecords\":[",
        "{\"attributes\":[{\"key\":\"event.name\",\"value\":{\"stringValue\":\"claude_code.user_prompt\"}},",
        "{\"key\":\"session.id\",\"value\":{\"stringValue\":\"sess-budget\"}},",
        "{\"key\":\"prompt.id\",\"value\":{\"stringValue\":\"p-1\"}}],",
        "\"timeUnixNano\":\"1794841200000000000\"},",
        "{\"attributes\":[{\"key\":\"event.name\",\"value\":{\"stringValue\":\"claude_code.api_request\"}},",
        "{\"key\":\"session.id\",\"value\":{\"stringValue\":\"sess-budget\"}},",
        "{\"key\":\"prompt.id\",\"value\":{\"stringValue\":\"p-1\"}},",
        "{\"key\":\"model\",\"value\":{\"stringValue\":\"claude-sonnet-4-5\"}},",
        "{\"key\":\"input_tokens\",\"value\":{\"intValue\":\"100\"}},",
        "{\"key\":\"output_tokens\",\"value\":{\"intValue\":\"10\"}},",
        "{\"key\":\"cache_read_tokens\",\"value\":{\"intValue\":\"0\"}},",
        "{\"key\":\"cache_creation_tokens\",\"value\":{\"intValue\":\"0\"}},",
        "{\"key\":\"cost_usd\",\"value\":{\"doubleValue\":0.001}},",
        "{\"key\":\"duration_ms\",\"value\":{\"intValue\":\"4000\"}},",
        "{\"key\":\"ttft_ms\",\"value\":{\"intValue\":\"100\"}},",
        "{\"key\":\"attempt\",\"value\":{\"intValue\":\"1\"}}],",
        "\"timeUnixNano\":\"1794841204000000000\"}],\"scope\":{\"name\":\"claude-code\"}}]}]}"
    );
    // The model step runs from 1s to 5s past the base; every
    // window holds its midpoint, and window lengths shrink
    // as i grows, so the smallest window wraps first.
    let base: i64 = 1794841203000000000;
    let build = |windows: usize| -> Vec<Span> {
        (0..windows)
            .map(|i| Span {
                trace_id: "t1".to_string(),
                span_id: format!("a{i}"),
                parent_span_id: None,
                name: "claude_code.interaction".to_string(),
                kind: SpanKind::Internal,
                start_unix_nano: base + i as i64 * 1_000_000,
                end_unix_nano: base + (12_000 - i as i64) * 1_000_000,
                is_error: false,
                service: "claude-code".to_string(),
                attributes: BTreeMap::from([
                    ("session.id".to_string(), "sess-budget".to_string()),
                    ("agent_id".to_string(), format!("agent-{i}")),
                ]),
            })
            .collect()
    };
    let deep = build(MAX_AGENT_NESTING + 1);
    let err = telorama_ingest::parse_claude_code_session(logs, &deep)
        .expect_err("a wrap past the budget must refuse");
    assert!(
        format!("{err}").contains("512"),
        "the refusal names the budget: {err}"
    );
    let at_budget = build(MAX_AGENT_NESTING);
    let run = telorama_ingest::parse_claude_code_session(logs, &at_budget)
        .expect("wraps at the budget must build");
    assert_eq!(run.turns.len(), 1);
}

#[test]
fn genai_session_reads_the_dotted_cache_creation_name() {
    // The 1.37 and 1.41 generation spelling of cache creation, the
    // middle step of the INV-65 fallback chain. The fixture mirrors
    // the legacy one figure for figure with only the spelling
    // changed, so the totals must match exactly.
    let spans = telorama_ingest::parse_otlp_traces(&fixture_text(
        "tests/fixtures/agents/genai-session-creation-spans.json",
    ))
    .expect("creation genai fixture must parse");
    let run = telorama_ingest::parse_genai_session(&spans).expect("spans must adapt");
    assert_eq!(run.session_id, "conv-genai-creation");
    let turn = &run.turns[0];
    assert_eq!(turn.input_tokens, 6200);
    assert_eq!(turn.output_tokens, 1500);
    assert_eq!(turn.cache_read_tokens, 3000);
    assert_eq!(turn.cache_creation_tokens, 400);
    assert!((turn.cost_usd - 0.023125).abs() < 1e-9, "{}", turn.cost_usd);
}

#[test]
fn genai_session_prefers_the_cache_write_spelling() {
    // When one span carries two creation spellings, the cache_write
    // spelling wins, per the INV-65 read order. The bogus dotted
    // value must not move the count.
    let mut spans = telorama_ingest::parse_otlp_traces(&fixture_text(
        "tests/fixtures/agents/genai-session-cachewrite-spans.json",
    ))
    .expect("cache-write genai fixture must parse");
    for span in spans.iter_mut() {
        if span.span_id == "00000000000000a2" {
            span.attributes.insert(
                "gen_ai.usage.cache_creation.input_tokens".to_string(),
                "999".to_string(),
            );
        }
    }
    let run = telorama_ingest::parse_genai_session(&spans).expect("spans must adapt");
    assert_eq!(run.turns[0].cache_creation_tokens, 400);
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
    assert_eq!(run.turns.len(), 2);
    assert_eq!(run.coverage.spans_seen, 10);
    let pricing = run.pricing.as_ref().expect("span runs are priced");
    assert_eq!(
        pricing.priced_models,
        vec![
            "claude-sonnet-5".to_string(),
            "gpt-5.1".to_string(),
            "gpt-5.2".to_string()
        ]
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

    // The second trace nests a workflow under an agent, where the
    // classification is observable: the workflow wraps its chat
    // call as a subagent step. The plan span beside it stays a
    // transparent container, so its chat call lifts into the turn.
    let turn = &run.turns[1];
    assert_eq!(turn.steps.len(), 2);
    let AgentStep::Subagent(sub) = &turn.steps[0] else {
        panic!("step 1 must be the nested workflow as a subagent");
    };
    assert_eq!(sub.agent_id, "workflow-intake");
    assert_eq!(sub.steps.len(), 1);
    assert!(matches!(sub.steps[0], AgentStep::ModelCall(_)));
    assert!(matches!(turn.steps[1], AgentStep::ModelCall(_)));
    assert_eq!(turn.input_tokens, 1500);
    assert_eq!(turn.output_tokens, 300);
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
fn openinference_session_prices_the_fable_and_mythos_lines() {
    // INV-93: the four Claude ids this fixture names carry rows
    // behind the vendor's published prices, so the whole run
    // prices and the total is the sum at the Round 44 rows.
    use telorama_scene::AgentStep;
    let spans = telorama_ingest::parse_otlp_traces(&fixture_text(
        "tests/fixtures/agents/openinference-session-fable-spans.json",
    ))
    .expect("fable openinference fixture must parse");
    let run = telorama_ingest::parse_openinference_session(&spans).expect("spans must adapt");
    assert_eq!(run.session_id, "sess-oi-fable");
    assert_eq!(run.dialect, "openinference");
    assert_eq!(run.coverage.spans_seen, 6);
    let pricing = run.pricing.as_ref().expect("span runs are priced");
    assert_eq!(pricing.table_as_of, "2026-11-25");
    assert_eq!(
        pricing.priced_models,
        vec![
            "claude-fable-5".to_string(),
            "claude-fable-5.1".to_string(),
            "claude-mythos-5".to_string(),
            "claude-mythos-5.1".to_string()
        ]
    );
    assert!(pricing.unpriced_models.is_empty());

    let turn = &run.turns[0];
    assert_eq!(turn.steps.len(), 5);
    // Prompt tokens enter net of cache reads, as in every dialect.
    assert_eq!(turn.input_tokens, 5600);
    assert_eq!(turn.output_tokens, 1250);
    assert_eq!(turn.cache_read_tokens, 1800);
    assert!((turn.cost_usd - 0.11955).abs() < 1e-9, "{}", turn.cost_usd);
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
fn openinference_decision_spans_stay_out_of_the_totals() {
    // A DECISION span stays a transparent container per INV-67:
    // it emits no step of its own, its child still walks, and its
    // decision token counts never enter the turn's model totals.
    use telorama_scene::AgentStep;
    let mut spans = telorama_ingest::parse_otlp_traces(&fixture_text(
        "tests/fixtures/agents/openinference-session-reranker-spans.json",
    ))
    .expect("reranker fixture must parse");
    let llm = spans
        .iter()
        .find(|s| s.span_id == "00000000000000d4")
        .expect("fixture carries the llm span")
        .clone();
    let mut decision = llm.clone();
    decision.span_id = "00000000000000d5".to_string();
    decision.parent_span_id = Some("00000000000000d1".to_string());
    decision.name = "route choice".to_string();
    decision.start_unix_nano = 1_794_852_311_200_000_000;
    decision.end_unix_nano = 1_794_852_311_600_000_000;
    decision.attributes.clear();
    decision.attributes.insert(
        "openinference.span.kind".to_string(),
        "DECISION".to_string(),
    );
    decision
        .attributes
        .insert("decision.token_count.input".to_string(), "777".to_string());
    decision
        .attributes
        .insert("decision.token_count.output".to_string(), "888".to_string());
    let mut child = llm.clone();
    child.span_id = "00000000000000d6".to_string();
    child.parent_span_id = Some("00000000000000d5".to_string());
    child.name = "llm call".to_string();
    child.start_unix_nano = 1_794_852_311_300_000_000;
    child.end_unix_nano = 1_794_852_311_500_000_000;
    child
        .attributes
        .insert("llm.token_count.prompt".to_string(), "100".to_string());
    child
        .attributes
        .insert("llm.token_count.completion".to_string(), "10".to_string());
    spans.push(decision);
    spans.push(child);
    let run = telorama_ingest::parse_openinference_session(&spans).expect("spans must adapt");
    assert_eq!(run.turns.len(), 1);
    let turn = &run.turns[0];
    // The three fixture steps plus the decision span's lifted child.
    assert_eq!(turn.steps.len(), 4);
    assert!(matches!(turn.steps[3], AgentStep::ModelCall(_)));
    // Only the child call's tokens join the totals, never the 777.
    assert_eq!(turn.input_tokens, 1600);
    assert_eq!(turn.output_tokens, 260);
}

#[test]
fn openinference_session_ignores_span_carried_cost() {
    // Span-carried cost is never read, per INV-68. The renderer's
    // table is the only pricing law, so a cost attribute on the
    // span must not move the priced figure.
    let mut spans = telorama_ingest::parse_otlp_traces(&fixture_text(
        "tests/fixtures/agents/openinference-session-reranker-spans.json",
    ))
    .expect("reranker fixture must parse");
    for span in spans.iter_mut() {
        if span.span_id == "00000000000000d4" {
            span.attributes
                .insert("llm.cost.total".to_string(), "99.99".to_string());
        }
    }
    let run = telorama_ingest::parse_openinference_session(&spans).expect("spans must adapt");
    let turn = &run.turns[0];
    // 1500 input and 250 output at the gpt-5.1 row.
    assert!((turn.cost_usd - 0.004375).abs() < 1e-9, "{}", turn.cost_usd);
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
fn claude_refusal_becomes_an_error_model_call() {
    // INV-83: a claude_code.api_refusal event is a failed model
    // call in its turn, with the error flag set and zero tokens,
    // exactly the shape an api_error call gets.
    let logs = include_str!("../../../tests/fixtures/agents/claude-code-refusal-logs.json");
    let run = telorama_ingest::parse_claude_code_session(logs, &[]).expect("run must parse");
    assert_eq!(run.session_id, "sess-2026-1116-refusal");
    assert_eq!(run.turns.len(), 1);
    let turn = &run.turns[0];
    assert_eq!(turn.steps.len(), 2);
    let refusal = match &turn.steps[1] {
        telorama_scene::AgentStep::ModelCall(call) => call,
        other => panic!("the second step must be the refusal model call, got {other:?}"),
    };
    assert!(refusal.is_error);
    assert_eq!(refusal.model, "claude-sonnet-4-5");
    assert_eq!(refusal.input_tokens, 0);
    assert_eq!(refusal.output_tokens, 0);
    assert_eq!(refusal.cost_usd, 0.0);
    // The turn totals come from the successful call alone.
    assert_eq!(turn.input_tokens, 1200);
    assert_eq!(turn.output_tokens, 300);
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

#[test]
fn claude_session_reads_hostile_figures_as_defined_defaults() {
    // Round 15 (INV-88): a negative token count reads as
    // zero, a non-finite cost reads as zero, and an extreme
    // duration saturates, where the adapter once cast,
    // propagated, and overflowed on the same shapes.
    use serde_json::json;
    fn record(at: i64, name: &str, extra: &[(&str, serde_json::Value)]) -> String {
        let mut attributes = vec![
            json!({"key": "event.name", "value": {"stringValue": name}}),
            json!({"key": "session.id", "value": {"stringValue": "sess-hostile"}}),
            json!({"key": "prompt.id", "value": {"stringValue": "p-1"}}),
        ];
        for (key, value) in extra {
            attributes.push(json!({"key": key, "value": value}));
        }
        json!({
            "timeUnixNano": at.to_string(),
            "body": {"stringValue": name},
            "attributes": attributes,
        })
        .to_string()
    }
    let records = [
        record(1_794_841_200_000_000_000, "claude_code.user_prompt", &[]),
        record(
            1_794_841_201_000_000_000,
            "claude_code.api_request",
            &[
                ("model", json!({"stringValue": "claude-sonnet-4-5"})),
                ("input_tokens", json!({"intValue": "-1"})),
                ("output_tokens", json!({"intValue": "100"})),
                ("cost_usd", json!({"stringValue": "NaN"})),
                ("duration_ms", json!({"intValue": "10000000000000"})),
            ],
        ),
    ];
    let events = format!(
        "{{\"resourceLogs\":[{{\"scopeLogs\":[{{\"logRecords\":[{}]}}]}}]}}",
        records.join(",")
    );
    let run = telorama_ingest::parse_claude_code_session(&events, &[]).expect("the session parses");
    assert_eq!(run.turns.len(), 1);
    assert_eq!(run.turns[0].input_tokens, 0);
    assert_eq!(run.turns[0].output_tokens, 100);
    assert_eq!(run.turns[0].cost_usd, 0.0);
    // The investigation over this run must not panic.
    let findings = telorama_scene::investigate(&[], Some(&run), 60.0);
    assert!(findings.iter().all(|f| f.kind != "cost"));
}

#[test]
fn otlp_spans_without_ids_are_refused() {
    // Round 15 (INV-88): a span carrying no spanId is
    // refused by name, not aliased to an empty id.
    let doc = r#"{"resourceSpans":[{"resource":{"attributes":[{"key":"service.name","value":{"stringValue":"web"}}]},"scopeSpans":[{"spans":[{"traceId":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","name":"op","kind":2,"startTimeUnixNano":"1000","endTimeUnixNano":"2000"}]}]}]}"#;
    let err = telorama_ingest::parse_otlp_traces(doc).expect_err("an id-less span must be refused");
    assert!(err.to_string().contains("traceId or spanId"), "{err}");
}

#[test]
fn logs_reader_accepts_a_numeric_int_value() {
    // Round 15 (INV-88): the logs reader takes intValue as
    // a raw JSON number as well as a string, the leniency
    // the traces reader already applied, so the two
    // readers cannot disagree on a document.
    use serde_json::json;
    let record = json!({
        "timeUnixNano": "1794841201000000000",
        "body": {"stringValue": "claude_code.api_request"},
        "attributes": [
            {"key": "event.name", "value": {"stringValue": "claude_code.api_request"}},
            {"key": "session.id", "value": {"stringValue": "sess-numeric"}},
            {"key": "prompt.id", "value": {"stringValue": "p-1"}},
            {"key": "model", "value": {"stringValue": "claude-sonnet-4-5"}},
            {"key": "input_tokens", "value": {"intValue": 700}},
            {"key": "output_tokens", "value": {"intValue": 70}},
            {"key": "cost_usd", "value": {"doubleValue": 0.01}},
            {"key": "duration_ms", "value": {"intValue": 900}},
        ],
    });
    let events = json!({"resourceLogs": [{"scopeLogs": [{"logRecords": [record]}]}]});
    let run = telorama_ingest::parse_claude_code_session(&events.to_string(), &[])
        .expect("the session parses");
    assert_eq!(run.turns.len(), 1);
    assert_eq!(run.turns[0].input_tokens, 700);
}

#[test]
fn claude_session_cost_sum_stays_finite() {
    // Round 15 review repair (INV-88): two finite costs
    // near the f64 ceiling sum past it, and the turn total
    // saturates at the largest finite value instead of
    // reaching the headlines as an infinity.
    use serde_json::json;
    let record = |at: i64, name: &str, cost: Option<f64>| {
        let mut attributes = vec![
            json!({"key": "event.name", "value": {"stringValue": name}}),
            json!({"key": "session.id", "value": {"stringValue": "sess-costly"}}),
            json!({"key": "prompt.id", "value": {"stringValue": "p-1"}}),
        ];
        if name == "claude_code.api_request" {
            attributes.push(json!({"key": "model", "value": {"stringValue": "claude-sonnet-4-5"}}));
            attributes.push(json!({"key": "duration_ms", "value": {"intValue": "900"}}));
        }
        if let Some(c) = cost {
            attributes.push(json!({"key": "cost_usd", "value": {"doubleValue": c}}));
        }
        json!({
            "timeUnixNano": at.to_string(),
            "body": {"stringValue": name},
            "attributes": attributes,
        })
        .to_string()
    };
    let records = [
        record(1_794_841_200_000_000_000, "claude_code.user_prompt", None),
        record(
            1_794_841_201_000_000_000,
            "claude_code.api_request",
            Some(1e308),
        ),
        record(
            1_794_841_202_000_000_000,
            "claude_code.api_request",
            Some(1e308),
        ),
    ];
    let events = format!(
        "{{\"resourceLogs\":[{{\"scopeLogs\":[{{\"logRecords\":[{}]}}]}}]}}",
        records.join(",")
    );
    let run = telorama_ingest::parse_claude_code_session(&events, &[]).expect("the session parses");
    assert_eq!(run.turns.len(), 1);
    assert_eq!(run.turns[0].cost_usd, f64::MAX);
}

#[test]
fn claude_session_wait_sum_saturates() {
    // Round 15 review repair (INV-88): two blocked_on_user
    // spans at the i64 extremes sum their waits into the
    // tool step with saturation, inside the parse itself.
    use telorama_scene::{Span, SpanKind};
    let span = |id: &str, parent: Option<&str>, name: &str, start: i64, end: i64| Span {
        trace_id: "t1".to_string(),
        span_id: id.to_string(),
        parent_span_id: parent.map(|p| p.to_string()),
        name: name.to_string(),
        kind: SpanKind::Internal,
        start_unix_nano: start,
        end_unix_nano: end,
        is_error: false,
        service: "claude".to_string(),
        attributes: std::collections::BTreeMap::new(),
    };
    let spans = vec![
        span("tool", None, "claude_code.tool", 0, i64::MAX),
        span(
            "w1",
            Some("tool"),
            "claude_code.tool.blocked_on_user",
            0,
            i64::MAX,
        ),
        span(
            "w2",
            Some("tool"),
            "claude_code.tool.blocked_on_user",
            0,
            i64::MAX,
        ),
    ];
    let logs = r#"{"resourceLogs":[{"scopeLogs":[{"logRecords":[
        {"timeUnixNano":"1794841200000000000","body":{"stringValue":"claude_code.user_prompt"},"attributes":[{"key":"event.name","value":{"stringValue":"claude_code.user_prompt"}},{"key":"session.id","value":{"stringValue":"sess-wait"}},{"key":"prompt.id","value":{"stringValue":"p-1"}}]},
        {"timeUnixNano":"1794841201000000000","body":{"stringValue":"claude_code.tool_result"},"attributes":[{"key":"event.name","value":{"stringValue":"claude_code.tool_result"}},{"key":"session.id","value":{"stringValue":"sess-wait"}},{"key":"prompt.id","value":{"stringValue":"p-1"}},{"key":"tool_name","value":{"stringValue":"shell"}},{"key":"duration_ms","value":{"intValue":"1000"}}]}
    ]}]}]}"#;
    let run = telorama_ingest::parse_claude_code_session(logs, &spans).expect("the session parses");
    let waits: Vec<i64> = run.turns[0]
        .steps
        .iter()
        .filter_map(|s| match s {
            telorama_scene::AgentStep::ToolCall(t) => Some(t.wait_nanos),
            _ => None,
        })
        .collect();
    assert_eq!(waits, vec![i64::MAX]);
}

#[test]
fn unmodelled_attribute_shapes_are_skipped() {
    // Round 15 (INV-88): an attribute whose value is an
    // array is skipped by the traces reader, the rule the
    // logs reader already applied, instead of entering
    // the span as an empty string.
    let doc = r#"{"resourceSpans":[{"resource":{"attributes":[{"key":"service.name","value":{"stringValue":"web"}}]},"scopeSpans":[{"spans":[{"traceId":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","spanId":"00000000000000aa","name":"op","kind":2,"startTimeUnixNano":"1000","endTimeUnixNano":"2000","attributes":[{"key":"tags","value":{"arrayValue":{"values":[{"stringValue":"x"}]}}},{"key":"http.route","value":{"stringValue":"/pay"}}]}]}]}]}"#;
    let spans = telorama_ingest::parse_otlp_traces(doc).expect("the export parses");
    assert_eq!(spans.len(), 1);
    assert!(!spans[0].attributes.contains_key("tags"));
    assert_eq!(spans[0].attributes["http.route"], "/pay");
}

#[test]
fn genai_session_gives_each_copy_of_a_shared_id_its_own_step() {
    // Round 17 (INV-89): two copies of one span id under one
    // root, each with children naming the shared id. Both
    // copies emit their own subagent step, the children walk
    // under the adopting copy alone, and the turn totals and
    // coverage count every occurrence once.
    use telorama_scene::AgentStep;
    let doc = r#"{"resourceSpans":[{"scopeSpans":[{"spans":[
      {"traceId":"aa11bb22cc33dd44ee55ff6677889900","spanId":"00000000000000a1","name":"invoke_agent planner","startTimeUnixNano":"1794852000000000000","endTimeUnixNano":"1794852100000000000","attributes":[{"key":"gen_ai.operation.name","value":{"stringValue":"invoke_agent"}},{"key":"gen_ai.agent.id","value":{"stringValue":"agent-root"}},{"key":"gen_ai.conversation.id","value":{"stringValue":"conv-dup"}}]},
      {"traceId":"aa11bb22cc33dd44ee55ff6677889900","spanId":"00000000000000b1","parentSpanId":"00000000000000a1","name":"invoke_agent worker-a","startTimeUnixNano":"1794852010000000000","endTimeUnixNano":"1794852040000000000","attributes":[{"key":"gen_ai.operation.name","value":{"stringValue":"invoke_agent"}},{"key":"gen_ai.agent.id","value":{"stringValue":"agent-a"}},{"key":"gen_ai.conversation.id","value":{"stringValue":"conv-dup"}}]},
      {"traceId":"aa11bb22cc33dd44ee55ff6677889900","spanId":"00000000000000c1","parentSpanId":"00000000000000b1","name":"chat gpt-5.1","startTimeUnixNano":"1794852012000000000","endTimeUnixNano":"1794852020000000000","attributes":[{"key":"gen_ai.operation.name","value":{"stringValue":"chat"}},{"key":"gen_ai.request.model","value":{"stringValue":"gpt-5.1"}},{"key":"gen_ai.usage.input_tokens","value":{"intValue":"1000"}},{"key":"gen_ai.usage.output_tokens","value":{"intValue":"100"}}]},
      {"traceId":"aa11bb22cc33dd44ee55ff6677889900","spanId":"00000000000000b1","parentSpanId":"00000000000000a1","name":"invoke_agent worker-b","startTimeUnixNano":"1794852050000000000","endTimeUnixNano":"1794852090000000000","attributes":[{"key":"gen_ai.operation.name","value":{"stringValue":"invoke_agent"}},{"key":"gen_ai.agent.id","value":{"stringValue":"agent-b"}},{"key":"gen_ai.conversation.id","value":{"stringValue":"conv-dup"}}]},
      {"traceId":"aa11bb22cc33dd44ee55ff6677889900","spanId":"00000000000000c2","parentSpanId":"00000000000000b1","name":"chat gpt-5.1","startTimeUnixNano":"1794852055000000000","endTimeUnixNano":"1794852060000000000","attributes":[{"key":"gen_ai.operation.name","value":{"stringValue":"chat"}},{"key":"gen_ai.request.model","value":{"stringValue":"gpt-5.1"}},{"key":"gen_ai.usage.input_tokens","value":{"intValue":"2000"}},{"key":"gen_ai.usage.output_tokens","value":{"intValue":"200"}}]}
    ]}]}]}"#;
    let spans = telorama_ingest::parse_otlp_traces(doc).expect("the export parses");
    assert_eq!(spans.len(), 5);
    let run = telorama_ingest::parse_genai_session(&spans).expect("spans must adapt");
    assert_eq!(run.turns.len(), 1);
    let turn = &run.turns[0];
    assert_eq!(turn.steps.len(), 2);
    let AgentStep::Subagent(first) = &turn.steps[0] else {
        panic!("the first step must be the first copy's subagent");
    };
    assert_eq!(first.agent_id, "agent-a");
    assert_eq!(
        first.steps.len(),
        2,
        "both children walk under the adopting copy"
    );
    let AgentStep::Subagent(second) = &turn.steps[1] else {
        panic!("the second step must be the second copy's subagent");
    };
    assert_eq!(second.agent_id, "agent-b");
    assert!(second.steps.is_empty());
    // Each model call counts once, under the adopting copy.
    assert_eq!(turn.input_tokens, 3000);
    assert_eq!(turn.output_tokens, 300);
    assert_eq!(turn.coverage.spans_seen, 5);
    assert_eq!(turn.coverage.spans_merged, 5);
}

#[test]
fn genai_session_two_root_copies_do_not_double_the_shared_subtree() {
    // Round 17 (INV-89): both copies of one span id are roots.
    // Each builds its own turn, and the child naming the
    // shared id walks in the adopting copy's turn alone, so
    // the session totals count it once.
    let doc = r#"{"resourceSpans":[{"scopeSpans":[{"spans":[
      {"traceId":"aa11bb22cc33dd44ee55ff6677889900","spanId":"00000000000000b1","name":"invoke_agent worker-a","startTimeUnixNano":"1794852010000000000","endTimeUnixNano":"1794852040000000000","attributes":[{"key":"gen_ai.operation.name","value":{"stringValue":"invoke_agent"}},{"key":"gen_ai.agent.id","value":{"stringValue":"agent-a"}},{"key":"gen_ai.conversation.id","value":{"stringValue":"conv-dup2"}}]},
      {"traceId":"aa11bb22cc33dd44ee55ff6677889900","spanId":"00000000000000b1","name":"invoke_agent worker-b","startTimeUnixNano":"1794852050000000000","endTimeUnixNano":"1794852090000000000","attributes":[{"key":"gen_ai.operation.name","value":{"stringValue":"invoke_agent"}},{"key":"gen_ai.agent.id","value":{"stringValue":"agent-b"}},{"key":"gen_ai.conversation.id","value":{"stringValue":"conv-dup2"}}]},
      {"traceId":"aa11bb22cc33dd44ee55ff6677889900","spanId":"00000000000000c1","parentSpanId":"00000000000000b1","name":"chat gpt-5.1","startTimeUnixNano":"1794852012000000000","endTimeUnixNano":"1794852020000000000","attributes":[{"key":"gen_ai.operation.name","value":{"stringValue":"chat"}},{"key":"gen_ai.request.model","value":{"stringValue":"gpt-5.1"}},{"key":"gen_ai.usage.input_tokens","value":{"intValue":"1000"}},{"key":"gen_ai.usage.output_tokens","value":{"intValue":"100"}}]}
    ]}]}]}"#;
    let spans = telorama_ingest::parse_otlp_traces(doc).expect("the export parses");
    let run = telorama_ingest::parse_genai_session(&spans).expect("spans must adapt");
    assert_eq!(run.turns.len(), 2);
    assert_eq!(run.turns[0].input_tokens, 1000);
    assert_eq!(run.turns[0].steps.len(), 1);
    assert_eq!(run.turns[0].coverage.spans_seen, 2);
    assert_eq!(run.turns[1].input_tokens, 0);
    assert!(run.turns[1].steps.is_empty());
    assert_eq!(run.turns[1].coverage.spans_seen, 1);
}

#[test]
fn claude_code_blocked_wait_stays_with_its_own_tool_occurrence() {
    // Round 17 (INV-89): two claude_code.tool spans share one
    // id with disjoint windows, each with a blocked_on_user
    // child of its own. Each step's wait is its own
    // occurrence's sum, never the pooled sum of both.
    use telorama_scene::AgentStep;
    let logs = r#"{"resourceLogs":[{"scopeLogs":[{"logRecords":[
      {"timeUnixNano":"1794841200000000000","attributes":[{"key":"event.name","value":{"stringValue":"claude_code.user_prompt"}},{"key":"session.id","value":{"stringValue":"sess-dup"}},{"key":"prompt.id","value":{"stringValue":"p-1"}}]},
      {"timeUnixNano":"1794841300000000000","attributes":[{"key":"event.name","value":{"stringValue":"claude_code.tool_result"}},{"key":"session.id","value":{"stringValue":"sess-dup"}},{"key":"prompt.id","value":{"stringValue":"p-1"}},{"key":"tool_name","value":{"stringValue":"Bash"}},{"key":"success","value":{"boolValue":true}},{"key":"duration_ms","value":{"intValue":"100000"}}]},
      {"timeUnixNano":"1794841500000000000","attributes":[{"key":"event.name","value":{"stringValue":"claude_code.tool_result"}},{"key":"session.id","value":{"stringValue":"sess-dup"}},{"key":"prompt.id","value":{"stringValue":"p-1"}},{"key":"tool_name","value":{"stringValue":"Edit"}},{"key":"success","value":{"boolValue":true}},{"key":"duration_ms","value":{"intValue":"100000"}}]}
    ]}]}]}"#;
    let span_doc = r#"{"resourceSpans":[{"scopeSpans":[{"spans":[
      {"traceId":"aa11bb22cc33dd44ee55ff6677889900","spanId":"00000000000000e1","name":"claude_code.tool","startTimeUnixNano":"1794841200000000000","endTimeUnixNano":"1794841300000000000"},
      {"traceId":"aa11bb22cc33dd44ee55ff6677889900","spanId":"00000000000000f1","parentSpanId":"00000000000000e1","name":"claude_code.tool.blocked_on_user","startTimeUnixNano":"1794841210000000000","endTimeUnixNano":"1794841260000000000"},
      {"traceId":"aa11bb22cc33dd44ee55ff6677889900","spanId":"00000000000000e1","name":"claude_code.tool","startTimeUnixNano":"1794841400000000000","endTimeUnixNano":"1794841500000000000"},
      {"traceId":"aa11bb22cc33dd44ee55ff6677889900","spanId":"00000000000000f2","parentSpanId":"00000000000000e1","name":"claude_code.tool.blocked_on_user","startTimeUnixNano":"1794841410000000000","endTimeUnixNano":"1794841420000000000"}
    ]}]}]}"#;
    let spans = telorama_ingest::parse_otlp_traces(span_doc).expect("the spans parse");
    let run =
        telorama_ingest::parse_claude_code_session(logs, &spans).expect("the session must adapt");
    assert_eq!(run.turns.len(), 1);
    let turn = &run.turns[0];
    assert_eq!(turn.steps.len(), 2);
    let AgentStep::ToolCall(bash) = &turn.steps[0] else {
        panic!("the first step must be the Bash call");
    };
    assert_eq!(bash.wait_nanos, 50_000_000_000);
    let AgentStep::ToolCall(edit) = &turn.steps[1] else {
        panic!("the second step must be the Edit call");
    };
    assert_eq!(edit.wait_nanos, 10_000_000_000);
    assert_eq!(run.coverage.spans_merged, 2);
}

#[test]
fn claude_code_decisions_pair_to_results_by_occurrence() {
    // Round 18 (INV-90): one turn carries two Bash decisions
    // before either Bash result, the batched interleaving of
    // concurrent tool calls. The first result reads the first
    // decision and the second reads the second, so no label
    // crosses between the two invocations.
    use telorama_scene::AgentStep;
    let logs = r#"{"resourceLogs":[{"scopeLogs":[{"logRecords":[
      {"timeUnixNano":"1794841200000000000","attributes":[{"key":"event.name","value":{"stringValue":"claude_code.user_prompt"}},{"key":"session.id","value":{"stringValue":"sess-pair"}},{"key":"prompt.id","value":{"stringValue":"p-1"}}]},
      {"timeUnixNano":"1794841201000000000","attributes":[{"key":"event.name","value":{"stringValue":"claude_code.tool_decision"}},{"key":"session.id","value":{"stringValue":"sess-pair"}},{"key":"prompt.id","value":{"stringValue":"p-1"}},{"key":"tool_name","value":{"stringValue":"Bash"}},{"key":"decision","value":{"stringValue":"accept"}}]},
      {"timeUnixNano":"1794841202000000000","attributes":[{"key":"event.name","value":{"stringValue":"claude_code.tool_decision"}},{"key":"session.id","value":{"stringValue":"sess-pair"}},{"key":"prompt.id","value":{"stringValue":"p-1"}},{"key":"tool_name","value":{"stringValue":"Bash"}},{"key":"decision","value":{"stringValue":"reject"}}]},
      {"timeUnixNano":"1794841210000000000","attributes":[{"key":"event.name","value":{"stringValue":"claude_code.tool_result"}},{"key":"session.id","value":{"stringValue":"sess-pair"}},{"key":"prompt.id","value":{"stringValue":"p-1"}},{"key":"tool_name","value":{"stringValue":"Bash"}},{"key":"success","value":{"boolValue":true}},{"key":"duration_ms","value":{"intValue":"5000"}}]},
      {"timeUnixNano":"1794841220000000000","attributes":[{"key":"event.name","value":{"stringValue":"claude_code.tool_result"}},{"key":"session.id","value":{"stringValue":"sess-pair"}},{"key":"prompt.id","value":{"stringValue":"p-1"}},{"key":"tool_name","value":{"stringValue":"Bash"}},{"key":"success","value":{"boolValue":true}},{"key":"duration_ms","value":{"intValue":"5000"}}]}
    ]}]}]}"#;
    let run =
        telorama_ingest::parse_claude_code_session(logs, &[]).expect("the session must adapt");
    assert_eq!(run.turns.len(), 1);
    let turn = &run.turns[0];
    assert_eq!(turn.steps.len(), 2);
    let AgentStep::ToolCall(first) = &turn.steps[0] else {
        panic!("the first step must be the first Bash call");
    };
    assert_eq!(first.decision.as_deref(), Some("accept"));
    let AgentStep::ToolCall(second) = &turn.steps[1] else {
        panic!("the second step must be the second Bash call");
    };
    assert_eq!(second.decision.as_deref(), Some("reject"));
}

#[test]
fn codex_decisions_pair_to_results_by_occurrence() {
    // Round 18 (INV-90): the same batched interleaving in the
    // Codex dialect. Two exec_command decisions land before
    // either result, and the results read them in order.
    use telorama_scene::AgentStep;
    let logs = r#"{"resourceLogs":[{"scopeLogs":[{"logRecords":[
      {"timeUnixNano":"1794841200000000000","attributes":[{"key":"event.name","value":{"stringValue":"codex.user_prompt"}},{"key":"conversation.id","value":{"stringValue":"c-pair"}},{"key":"turn.id","value":{"stringValue":"t-1"}}]},
      {"timeUnixNano":"1794841201000000000","attributes":[{"key":"event.name","value":{"stringValue":"codex.tool_decision"}},{"key":"conversation.id","value":{"stringValue":"c-pair"}},{"key":"turn.id","value":{"stringValue":"t-1"}},{"key":"tool_name","value":{"stringValue":"exec_command"}},{"key":"decision","value":{"stringValue":"approved"}}]},
      {"timeUnixNano":"1794841202000000000","attributes":[{"key":"event.name","value":{"stringValue":"codex.tool_decision"}},{"key":"conversation.id","value":{"stringValue":"c-pair"}},{"key":"turn.id","value":{"stringValue":"t-1"}},{"key":"tool_name","value":{"stringValue":"exec_command"}},{"key":"decision","value":{"stringValue":"denied"}}]},
      {"timeUnixNano":"1794841210000000000","attributes":[{"key":"event.name","value":{"stringValue":"codex.tool_result"}},{"key":"conversation.id","value":{"stringValue":"c-pair"}},{"key":"turn.id","value":{"stringValue":"t-1"}},{"key":"tool_name","value":{"stringValue":"exec_command"}},{"key":"success","value":{"boolValue":true}},{"key":"duration_ms","value":{"intValue":"5000"}}]},
      {"timeUnixNano":"1794841220000000000","attributes":[{"key":"event.name","value":{"stringValue":"codex.tool_result"}},{"key":"conversation.id","value":{"stringValue":"c-pair"}},{"key":"turn.id","value":{"stringValue":"t-1"}},{"key":"tool_name","value":{"stringValue":"exec_command"}},{"key":"success","value":{"boolValue":true}},{"key":"duration_ms","value":{"intValue":"5000"}}]}
    ]}]}]}"#;
    let run = telorama_ingest::parse_codex_session(logs, &[]).expect("the session must adapt");
    assert_eq!(run.turns.len(), 1);
    let turn = &run.turns[0];
    assert_eq!(turn.steps.len(), 2);
    let AgentStep::ToolCall(first) = &turn.steps[0] else {
        panic!("the first step must be the first exec_command call");
    };
    assert_eq!(first.decision.as_deref(), Some("approved"));
    let AgentStep::ToolCall(second) = &turn.steps[1] else {
        panic!("the second step must be the second exec_command call");
    };
    assert_eq!(second.decision.as_deref(), Some("denied"));
}

#[test]
fn claude_code_blocked_child_resolves_to_the_smallest_containing_occurrence() {
    // Round 18 (INV-89): two claude_code.tool spans share one
    // id with nested windows, the larger first in the export,
    // and one blocked_on_user child sits inside both. The
    // child resolves to the smallest containing occurrence, so
    // the step only the larger occurrence contains keeps a
    // zero wait.
    use telorama_scene::AgentStep;
    let logs = r#"{"resourceLogs":[{"scopeLogs":[{"logRecords":[
      {"timeUnixNano":"1794841200000000000","attributes":[{"key":"event.name","value":{"stringValue":"claude_code.user_prompt"}},{"key":"session.id","value":{"stringValue":"sess-dup"}},{"key":"prompt.id","value":{"stringValue":"p-1"}}]},
      {"timeUnixNano":"1794841380000000000","attributes":[{"key":"event.name","value":{"stringValue":"claude_code.tool_result"}},{"key":"session.id","value":{"stringValue":"sess-dup"}},{"key":"prompt.id","value":{"stringValue":"p-1"}},{"key":"tool_name","value":{"stringValue":"Bash"}},{"key":"success","value":{"boolValue":true}},{"key":"duration_ms","value":{"intValue":"60000"}}]},
      {"timeUnixNano":"1794841500000000000","attributes":[{"key":"event.name","value":{"stringValue":"claude_code.tool_result"}},{"key":"session.id","value":{"stringValue":"sess-dup"}},{"key":"prompt.id","value":{"stringValue":"p-1"}},{"key":"tool_name","value":{"stringValue":"Edit"}},{"key":"success","value":{"boolValue":true}},{"key":"duration_ms","value":{"intValue":"50000"}}]}
    ]}]}]}"#;
    let span_doc = r#"{"resourceSpans":[{"scopeSpans":[{"spans":[
      {"traceId":"aa11bb22cc33dd44ee55ff6677889900","spanId":"00000000000000e1","name":"claude_code.tool","startTimeUnixNano":"1794841200000000000","endTimeUnixNano":"1794841500000000000"},
      {"traceId":"aa11bb22cc33dd44ee55ff6677889900","spanId":"00000000000000e1","name":"claude_code.tool","startTimeUnixNano":"1794841300000000000","endTimeUnixNano":"1794841400000000000"},
      {"traceId":"aa11bb22cc33dd44ee55ff6677889900","spanId":"00000000000000f1","parentSpanId":"00000000000000e1","name":"claude_code.tool.blocked_on_user","startTimeUnixNano":"1794841320000000000","endTimeUnixNano":"1794841370000000000"}
    ]}]}]}"#;
    let spans = telorama_ingest::parse_otlp_traces(span_doc).expect("the spans parse");
    let run =
        telorama_ingest::parse_claude_code_session(logs, &spans).expect("the session must adapt");
    assert_eq!(run.turns.len(), 1);
    let turn = &run.turns[0];
    assert_eq!(turn.steps.len(), 2);
    let AgentStep::ToolCall(bash) = &turn.steps[0] else {
        panic!("the first step must be the Bash call");
    };
    assert_eq!(bash.wait_nanos, 50_000_000_000);
    let AgentStep::ToolCall(edit) = &turn.steps[1] else {
        panic!("the second step must be the Edit call");
    };
    assert_eq!(edit.wait_nanos, 0);
    assert_eq!(run.coverage.spans_merged, 1);
}
