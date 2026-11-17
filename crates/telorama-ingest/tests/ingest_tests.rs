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
