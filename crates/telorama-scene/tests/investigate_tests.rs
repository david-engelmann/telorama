use telorama_scene::{
    AgentRun, AgentStep, AgentTurn, Coverage, ModelCall, Pricing, Span, SpanKind, SubagentRun,
    ToolCall, investigate, red_by_service,
};

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

fn checkout_like_spans() -> Vec<Span> {
    vec![
        span("t1", "01", None, "web", 0, 100, false),
        span("t1", "02", Some("01"), "pay", 10, 90, true),
        span("t1", "03", Some("01"), "email", 20, 30, true),
        span("t1", "04", Some("01"), "pay", 30, 40, true),
        span("t1", "05", Some("01"), "pay", 40, 50, false),
    ]
}

#[test]
fn investigate_ranks_errors_before_latency_and_matches_red() {
    let spans = checkout_like_spans();
    let findings = investigate(&spans, None, 1.0);
    assert_eq!(findings.len(), 2);
    let errors = &findings[0];
    assert_eq!(errors.rank, 1);
    assert_eq!(errors.kind, "errors");
    assert_eq!(errors.subject, "pay");
    // INV-49: the finding's numbers are the RED derivation's numbers.
    let reds = red_by_service(&spans, 1.0);
    let pay = reds.iter().find(|r| r.service == "pay").expect("pay red");
    assert_eq!(pay.errors, 2);
    assert!(
        errors
            .statement
            .contains("pay carries 66.7% of the errors in the window, 2 of 3 errors"),
        "{}",
        errors.statement
    );
    assert!(errors.statement.contains("above the standing 5%"));
    // INV-50: the exemplars exist in the input.
    assert!(
        spans
            .iter()
            .any(|s| s.span_id == errors.exemplars[0] && s.is_error)
    );
    assert_eq!(errors.exemplars[1], "t1");
    let latency = &findings[1];
    assert_eq!(latency.rank, 2);
    assert_eq!(latency.kind, "latency");
    assert!(
        latency
            .statement
            .contains("pay op in pay holds the largest self time at 80 ms"),
        "{}",
        latency.statement
    );
    assert!(latency.statement.contains("on the critical path"));
}

#[test]
fn investigate_is_deterministic_under_input_reordering() {
    let spans = checkout_like_spans();
    let mut shuffled = spans.clone();
    shuffled.reverse();
    let a = investigate(&spans, None, 1.0);
    let b = investigate(&shuffled, None, 1.0);
    assert_eq!(a, b);
}

fn model_call(cost: f64) -> AgentStep {
    AgentStep::ModelCall(ModelCall {
        model: "gpt-test".to_string(),
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
}

fn tool(name: &str, wait_s: i64) -> AgentStep {
    AgentStep::ToolCall(ToolCall {
        name: name.to_string(),
        start_unix_nano: 0,
        end_unix_nano: wait_s * 1_000_000_000,
        wait_nanos: wait_s * 1_000_000_000,
        is_error: false,
        decision: None,
    })
}

fn run_with_wait_and_cost() -> AgentRun {
    AgentRun {
        session_id: "sess-test".to_string(),
        dialect: "claude_code".to_string(),
        turns: vec![
            AgentTurn {
                prompt_id: "p1".to_string(),
                start_unix_nano: 0,
                end_unix_nano: 100_000_000_000,
                steps: vec![
                    model_call(0.30),
                    tool("shell", 20),
                    AgentStep::Subagent(SubagentRun {
                        agent_id: "sub-1".to_string(),
                        start_unix_nano: 0,
                        end_unix_nano: 5_000_000_000,
                        steps: vec![tool("read", 5)],
                    }),
                ],
                input_tokens: 100,
                output_tokens: 10,
                cache_read_tokens: 0,
                cache_creation_tokens: 0,
                cost_usd: 0.50,
                coverage: Coverage::default(),
            },
            AgentTurn {
                prompt_id: "p2".to_string(),
                start_unix_nano: 100_000_000_000,
                end_unix_nano: 100_000_000_000,
                steps: vec![model_call(0.10)],
                input_tokens: 50,
                output_tokens: 5,
                cache_read_tokens: 0,
                cache_creation_tokens: 0,
                cost_usd: 0.10,
                coverage: Coverage::default(),
            },
        ],
        coverage: Coverage {
            log_events: 4,
            spans_seen: 0,
            spans_merged: 0,
        },
        pricing: None,
    }
}

#[test]
fn investigate_wait_and_cost_findings_from_a_run() {
    let run = run_with_wait_and_cost();
    let findings = investigate(&[], Some(&run), 100.0);
    assert_eq!(findings.len(), 2);
    let wait = &findings[0];
    assert_eq!(wait.kind, "wait");
    assert!(
        wait.statement
            .contains("spent 25 s waiting on the user (25.0% of the session)"),
        "{}",
        wait.statement
    );
    assert!(wait.statement.contains("longest 20 s on shell"));
    assert_eq!(wait.exemplars, vec!["p1".to_string()]);
    let cost = &findings[1];
    assert_eq!(cost.kind, "cost");
    assert!(
        cost.statement.contains(
            "most expensive turn cost $0.5000 (83.3% of the session priced cost $0.6000)"
        ),
        "{}",
        cost.statement
    );
    assert_eq!(cost.exemplars, vec!["p1".to_string()]);
}

#[test]
fn investigate_unpriced_only_run_names_the_models() {
    let mut run = run_with_wait_and_cost();
    for turn in &mut run.turns {
        turn.cost_usd = 0.0;
    }
    run.pricing = Some(Pricing {
        table_as_of: "2026-11-16".to_string(),
        priced_models: Vec::new(),
        unpriced_models: vec!["gpt-unknown-9".to_string()],
    });
    let findings = investigate(&[], Some(&run), 100.0);
    let cost = findings
        .iter()
        .find(|f| f.kind == "cost")
        .expect("an unpriced cost finding");
    assert!(cost.statement.contains("No priced cost in this session"));
    assert!(cost.statement.contains("gpt-unknown-9"));
}

#[test]
fn investigate_empty_input_is_empty() {
    assert!(investigate(&[], None, 60.0).is_empty());
}
