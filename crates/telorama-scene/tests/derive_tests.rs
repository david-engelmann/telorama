use std::collections::BTreeMap;
use telorama_scene::{
    Span, SpanKind, critical_path, percentile_nanos, red_by_service, waterfall_rows,
};

fn ms(v: i64) -> i64 {
    v * 1_000_000
}

fn span(
    id: &str,
    parent: Option<&str>,
    service: &str,
    kind: SpanKind,
    start: i64,
    end: i64,
    error: bool,
) -> Span {
    Span {
        trace_id: "t1".to_string(),
        span_id: id.to_string(),
        parent_span_id: parent.map(|p| p.to_string()),
        name: format!("op-{id}"),
        kind,
        start_unix_nano: ms(start),
        end_unix_nano: ms(end),
        is_error: error,
        service: service.to_string(),
        attributes: BTreeMap::new(),
    }
}

/// The checkout shape from the workspace fixture, rebuilt in code:
/// a root, a checkout span, and children including a parallel pair and a
/// short error span that finishes last.
fn checkout_spans() -> Vec<Span> {
    vec![
        span("01", None, "frontend", SpanKind::Server, 0, 420, false),
        span(
            "02",
            Some("01"),
            "checkout",
            SpanKind::Server,
            10,
            400,
            false,
        ),
        span(
            "03",
            Some("02"),
            "product-catalog",
            SpanKind::Server,
            20,
            60,
            false,
        ),
        span(
            "04",
            Some("02"),
            "currency",
            SpanKind::Server,
            65,
            85,
            false,
        ),
        span(
            "05",
            Some("02"),
            "checkout",
            SpanKind::Client,
            88,
            262,
            false,
        ),
        span(
            "06",
            Some("05"),
            "payment",
            SpanKind::Server,
            90,
            260,
            false,
        ),
        span(
            "07",
            Some("02"),
            "inventory",
            SpanKind::Server,
            100,
            180,
            false,
        ),
        span(
            "08",
            Some("02"),
            "shipping",
            SpanKind::Server,
            270,
            380,
            false,
        ),
        span("09", Some("02"), "email", SpanKind::Server, 385, 398, true),
    ]
}

#[test]
fn waterfall_rows_are_depth_first_with_depths() {
    let spans = checkout_spans();
    let rows = waterfall_rows(&spans, "t1");
    let order: Vec<&str> = rows.iter().map(|r| r.span_id.as_str()).collect();
    assert_eq!(
        order,
        vec!["01", "02", "03", "04", "05", "06", "07", "08", "09"]
    );
    let depths: Vec<usize> = rows.iter().map(|r| r.depth).collect();
    assert_eq!(depths, vec![0, 1, 2, 2, 2, 3, 2, 2, 2]);
}

#[test]
fn self_time_subtracts_children_coverage() {
    let spans = checkout_spans();
    let rows = waterfall_rows(&spans, "t1");
    let by_id: BTreeMap<&str, i64> = rows
        .iter()
        .map(|r| (r.span_id.as_str(), r.self_time_nanos))
        .collect();
    // Root: 420 minus the checkout span's 390.
    assert_eq!(by_id["01"], ms(30));
    // Checkout: 390 minus children coverage 40+20+174+110+13 = 357.
    assert_eq!(by_id["02"], ms(33));
    // A leaf keeps its whole duration.
    assert_eq!(by_id["09"], ms(13));
}

#[test]
fn critical_path_follows_the_last_finishing_child() {
    let spans = checkout_spans();
    assert_eq!(critical_path(&spans, "t1"), vec!["01", "02", "09"]);
}

#[test]
fn critical_path_of_empty_trace_is_empty() {
    assert!(critical_path(&[], "nope").is_empty());
    assert!(waterfall_rows(&[], "nope").is_empty());
}

#[test]
fn percentile_is_nearest_rank() {
    let values = vec![40, 10, 30, 20];
    assert_eq!(percentile_nanos(&values, 50), Some(20));
    assert_eq!(percentile_nanos(&values, 95), Some(40));
    assert_eq!(percentile_nanos(&values, 100), Some(40));
    assert_eq!(percentile_nanos(&[], 95), None);
}

#[test]
fn red_counts_server_spans_when_present() {
    let spans = vec![
        span("01", None, "a", SpanKind::Server, 0, 100, false),
        span("02", None, "a", SpanKind::Server, 0, 300, true),
        span("03", None, "a", SpanKind::Client, 0, 999, false),
        span("04", None, "b", SpanKind::Internal, 0, 50, false),
    ];
    let stats = red_by_service(&spans, 2.0);
    assert_eq!(stats.len(), 2);
    let a = &stats[0];
    assert_eq!(a.service, "a");
    assert_eq!(a.calls, 2, "the client span must not double the count");
    assert_eq!(a.errors, 1);
    assert_eq!(a.error_ratio, 0.5);
    assert_eq!(a.rate_per_sec, 1.0);
    assert_eq!(a.basis, "server");
    assert_eq!(a.p50_nanos, ms(100));
    assert_eq!(a.p99_nanos, ms(300));
    let b = &stats[1];
    assert_eq!(b.basis, "all");
    assert_eq!(b.calls, 1);
}

#[test]
fn derivations_do_not_depend_on_input_order() {
    let mut spans = checkout_spans();
    let rows_a = waterfall_rows(&spans, "t1");
    let path_a = critical_path(&spans, "t1");
    spans.reverse();
    assert_eq!(rows_a, waterfall_rows(&spans, "t1"));
    assert_eq!(path_a, critical_path(&spans, "t1"));
}
