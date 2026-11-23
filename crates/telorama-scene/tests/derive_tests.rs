use std::collections::BTreeMap;
use telorama_scene::{
    Span, SpanKind, StackSample, TimeRange, critical_path, flame_from_spans, flame_tree,
    heatmap_grid, percentile_nanos, red_by_service, sankey_flow, service_graph, waterfall_rows,
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

#[test]
fn heatmap_buckets_by_duration_and_start() {
    let spans = checkout_spans();
    let range = TimeRange {
        from_unix_ms: 0,
        to_unix_ms: 1000,
    };
    let grid = heatmap_grid(&spans, range, &[0.01, 0.05, 0.1, 0.25, 0.5], 24);
    let totals: Vec<u64> = grid.rows.iter().map(|r| r.total).collect();
    // Bands bottom-up: (0,10ms], (10,50ms], (50,100ms], (100,250ms],
    // (250,500ms], overflow. The 13, 20, and 40 ms spans share the
    // second band; the two payment spans and shipping share the fourth.
    assert_eq!(totals, vec![0, 3, 1, 3, 2, 0]);
    assert_eq!(grid.max_count, 2);
    assert_eq!(grid.outside_window, 0);
    // Two cells tie at two spans; the lower band wins the scan, and in
    // it the 174 ms client span beats the 170 ms server span.
    let exemplar = grid.exemplar.expect("a busy grid has an exemplar");
    assert_eq!(exemplar.span_id, "05");
    assert_eq!(exemplar.duration_nanos, ms(174));
    assert_eq!((exemplar.row, exemplar.col), (3, 2));
}

#[test]
fn heatmap_counts_spans_outside_the_window() {
    let spans = checkout_spans();
    let range = TimeRange {
        from_unix_ms: 100,
        to_unix_ms: 200,
    };
    let grid = heatmap_grid(&spans, range, &[0.01, 0.05, 0.1, 0.25, 0.5], 24);
    // Only the inventory span starts inside [100, 200).
    assert_eq!(grid.outside_window, 8);
    let total: u64 = grid.rows.iter().map(|r| r.total).sum();
    assert_eq!(total, 1);
}

fn stack(frames: &[&str], count: u64) -> StackSample {
    StackSample {
        frames: frames.iter().map(|f| f.to_string()).collect(),
        count,
    }
}

#[test]
fn flame_tree_folds_stacks_and_orders_children() {
    let stacks = vec![
        stack(&["main", "a"], 30),
        stack(&["main", "b"], 50),
        stack(&["main", "a", "c"], 20),
        stack(&["gc"], 10),
    ];
    let tree = flame_tree(&stacks);
    assert_eq!(tree.value, 110);
    assert_eq!(tree.children.len(), 2);
    assert_eq!(tree.children[0].name, "main");
    assert_eq!(tree.children[0].value, 100);
    assert_eq!(tree.children[1].name, "gc");
    assert_eq!(tree.children[1].value, 10);
    let main = &tree.children[0];
    // a carries 30 direct plus 20 through c: 50, tying b at 50, and
    // the name breaks the tie.
    assert_eq!(main.children[0].name, "a");
    assert_eq!(main.children[0].value, 50);
    assert_eq!(main.children[1].name, "b");
    assert_eq!(main.children[0].children[0].name, "c");
    assert_eq!(main.children[0].children[0].value, 20);
}

#[test]
fn flame_from_spans_weights_self_time() {
    let spans = checkout_spans();
    let tree = flame_from_spans(&spans, "t1");
    // The tree's total is the sum of every span's self time: 500 ms.
    // That is more than the 420 ms wall clock, because the inventory
    // span runs inside the payment span's window: parallel work holds
    // self time the wall clock counts once. Self time is the honest
    // weight for a span-derived flame, and the render labels it.
    assert_eq!(tree.value, ms(500) as u64);
    assert_eq!(tree.children.len(), 1);
    assert_eq!(tree.children[0].value, ms(500) as u64);
    // The checkout span: 33 ms of self time plus 437 ms of child
    // values, the children again overlapping each other.
    assert_eq!(tree.children[0].children[0].value, ms(470) as u64);
}

#[test]
fn service_graph_edges_come_from_span_parents() {
    let spans = checkout_spans();
    let graph = service_graph(&spans, 1.0);
    assert_eq!(graph.nodes.len(), 8);
    // frontend into checkout, then checkout into its six callees.
    assert_eq!(graph.edges.len(), 7);
    let entry = graph
        .edges
        .iter()
        .find(|e| e.to == "checkout")
        .expect("the entry edge exists");
    assert_eq!(entry.from, "frontend");
    assert_eq!(entry.calls, 1);
    assert_eq!(entry.p99_nanos, ms(390));
    let email = graph
        .edges
        .iter()
        .find(|e| e.to == "email")
        .expect("the email edge exists");
    assert_eq!(email.from, "checkout");
    assert_eq!(email.calls, 1);
    assert_eq!(email.errors, 1);
}

#[test]
fn sankey_stages_spans_by_depth() {
    let spans = vec![
        span("a", None, "web", SpanKind::Server, 0, 100, false),
        span("b", Some("a"), "api", SpanKind::Server, 10, 80, false),
        span("c", Some("b"), "db", SpanKind::Client, 20, 50, false),
    ];
    let flow = sankey_flow(&spans, "other");
    assert_eq!(flow.stages, 3);
    assert_eq!(flow.residual_spans, 0);
    assert_eq!(flow.nodes.len(), 3);
    assert_eq!(flow.nodes[0].stage, 0);
    assert_eq!(flow.nodes[0].service, "web");
    assert_eq!(flow.nodes[1].stage, 1);
    assert_eq!(flow.nodes[1].service, "api");
    assert_eq!(flow.nodes[2].stage, 2);
    assert_eq!(flow.nodes[2].service, "db");
    assert_eq!(flow.links.len(), 2);
    assert!(flow.links.iter().all(|l| l.value == 1 && !l.from_residual));
}

#[test]
fn sankey_same_service_steps_are_flow() {
    let spans = vec![
        span("a", None, "web", SpanKind::Server, 0, 100, false),
        span("b", Some("a"), "web", SpanKind::Internal, 10, 60, false),
    ];
    let flow = sankey_flow(&spans, "other");
    assert_eq!(flow.nodes.len(), 2);
    assert_eq!(flow.nodes[0].stage, 0);
    assert_eq!(flow.nodes[1].stage, 1);
    assert_eq!(flow.nodes[1].service, "web");
    assert_eq!(flow.links.len(), 1);
    assert_eq!(flow.links[0].from_service, "web");
    assert_eq!(flow.links[0].to_service, "web");
    assert_eq!(flow.links[0].value, 1);
}

#[test]
fn sankey_orphans_and_cycles_enter_through_the_residual() {
    // An orphan whose parent is absent, with a child beneath it.
    let spans = vec![
        span("o", Some("missing"), "api", SpanKind::Server, 0, 90, false),
        span("p", Some("o"), "db", SpanKind::Client, 10, 40, false),
    ];
    let flow = sankey_flow(&spans, "other");
    assert_eq!(flow.residual_spans, 1);
    let residual = flow
        .nodes
        .iter()
        .find(|n| n.residual)
        .expect("the residual node exists");
    assert_eq!(residual.stage, 0);
    assert_eq!(residual.service, "other");
    assert_eq!(residual.value, 1);
    let api = flow
        .nodes
        .iter()
        .find(|n| n.service == "api")
        .expect("the orphan stages under its own service");
    assert_eq!(api.stage, 1);
    let entry = flow
        .links
        .iter()
        .find(|l| l.from_residual)
        .expect("the orphan links from the residual node");
    assert_eq!(entry.to_service, "api");
    assert_eq!(entry.value, 1);
    let child = flow
        .links
        .iter()
        .find(|l| l.to_service == "db")
        .expect("the child links from the orphan's node");
    assert_eq!(child.from_service, "api");
    assert!(!child.from_residual);

    // A two-span cycle: neither side can stage the other, so both
    // enter through the residual bucket.
    let cyclic = vec![
        span("x", Some("y"), "a", SpanKind::Server, 0, 50, false),
        span("y", Some("x"), "b", SpanKind::Server, 0, 50, false),
    ];
    let flow = sankey_flow(&cyclic, "other");
    assert_eq!(flow.residual_spans, 2);
    assert!(
        flow.nodes
            .iter()
            .filter(|n| !n.residual)
            .all(|n| n.stage == 1),
        "both cycle members stage at stage one"
    );
    assert_eq!(
        flow.links.iter().filter(|l| l.from_residual).count(),
        2,
        "both cycle members link from the residual node"
    );
}

#[test]
fn sankey_flow_conserves_on_the_fixtures() {
    let spans = vec![
        span("a", None, "web", SpanKind::Server, 0, 100, false),
        span("b", Some("a"), "api", SpanKind::Server, 10, 80, false),
        span("c", Some("a"), "api", SpanKind::Server, 15, 60, false),
        span("d", Some("b"), "db", SpanKind::Client, 20, 50, false),
        span("o", Some("missing"), "api", SpanKind::Server, 0, 30, false),
    ];
    let flow = sankey_flow(&spans, "other");
    // Every span is counted exactly once across the real nodes.
    let counted: u64 = flow
        .nodes
        .iter()
        .filter(|n| !n.residual)
        .map(|n| n.value)
        .sum();
    assert_eq!(counted, spans.len() as u64);
    // Per node: total inflow (parent links plus residual entries)
    // plus the root spans staged there equals the span count
    // exactly. Fan-out lives in the outflow figure, checked node by
    // node against the fixture: the web root parents two spans, the
    // api node parents one, the db node none.
    for node in flow.nodes.iter().filter(|n| !n.residual) {
        let inflow: u64 = flow
            .links
            .iter()
            .filter(|l| l.to_stage == node.stage && l.to_service == node.service)
            .map(|l| l.value)
            .sum();
        let roots = spans
            .iter()
            .filter(|s| s.parent_span_id.is_none() && s.service == node.service && node.stage == 0)
            .count() as u64;
        assert_eq!(
            inflow + roots,
            node.value,
            "node {} at stage {} must account for every staged span",
            node.service,
            node.stage
        );
    }
    let node = |stage: usize, service: &str| {
        flow.nodes
            .iter()
            .find(|n| n.stage == stage && n.service == service && !n.residual)
            .unwrap_or_else(|| panic!("node {service} at stage {stage} exists"))
    };
    assert_eq!(node(0, "web").outflow, 2, "the root fans out to two");
    assert_eq!(node(1, "api").outflow, 1);
    assert_eq!(node(2, "db").outflow, 0);
    // Total real outflow equals the number of spans whose parent is
    // in the set: three in this fixture.
    let total_out: u64 = flow
        .nodes
        .iter()
        .filter(|n| !n.residual)
        .map(|n| n.outflow)
        .sum();
    assert_eq!(total_out, 3);
    // The residual node's outflow is exactly the orphan count.
    let residual_out: u64 = flow
        .links
        .iter()
        .filter(|l| l.from_residual)
        .map(|l| l.value)
        .sum();
    assert_eq!(residual_out, flow.residual_spans);
    assert_eq!(flow.residual_spans, 1);
}

#[test]
fn sankey_duplicate_span_ids_stage_deterministically() {
    // Malformed telemetry: one id, two copies, one parented and
    // one a root. The canonical copy (the root, since a missing
    // parent sorts first) drives the staging walk; the parented
    // copy stages beneath its own parent. Nothing panics, every
    // copy is counted, and the flow is identical with the input
    // reversed.
    let spans = vec![
        span("x", Some("r"), "api", SpanKind::Server, 10, 50, false),
        span("r", None, "web", SpanKind::Server, 0, 100, false),
        span("x", None, "api", SpanKind::Server, 0, 90, false),
    ];
    let flow = sankey_flow(&spans, "other");
    let mut reversed = spans.clone();
    reversed.reverse();
    assert_eq!(flow, sankey_flow(&reversed, "other"));
    assert_eq!(flow.residual_spans, 0);
    let counted: u64 = flow
        .nodes
        .iter()
        .filter(|n| !n.residual)
        .map(|n| n.value)
        .sum();
    assert_eq!(counted, 3, "every copy is counted exactly once");
    let node = |stage: usize, service: &str| {
        flow.nodes
            .iter()
            .find(|n| n.stage == stage && n.service == service && !n.residual)
            .unwrap_or_else(|| panic!("node {service} at stage {stage} exists"))
    };
    assert_eq!(node(0, "web").value, 1);
    assert_eq!(
        node(0, "api").value,
        1,
        "the canonical root copy stages at zero"
    );
    assert_eq!(
        node(1, "api").value,
        1,
        "the parented copy stages beneath its parent"
    );
    assert_eq!(node(0, "web").outflow, 1);
}

#[test]
fn sankey_edge_cases_hold() {
    // A span that is its own parent cycles on itself and enters
    // through the residual.
    let selfish = vec![span("s", Some("s"), "api", SpanKind::Server, 0, 10, false)];
    let flow = sankey_flow(&selfish, "other");
    assert_eq!(flow.residual_spans, 1);
    assert_eq!(flow.stages, 2);
    // A tail hanging into a cycle stages beneath the cycle member
    // it hangs from; only the cycle members enter through the
    // residual.
    let spans = vec![
        span("x", Some("y"), "api", SpanKind::Server, 0, 50, false),
        span("y", Some("x"), "db", SpanKind::Server, 0, 50, false),
        span("z", Some("x"), "web", SpanKind::Client, 10, 20, false),
    ];
    let flow = sankey_flow(&spans, "other");
    assert_eq!(flow.residual_spans, 2);
    let z = flow
        .nodes
        .iter()
        .find(|n| n.service == "web")
        .expect("the tail stages");
    assert_eq!(z.stage, 2);
    // Span ids shared across traces stage independently: the same
    // id is a root in both traces and parents a child in one.
    let mut t2_root = span("a", None, "web", SpanKind::Server, 0, 100, false);
    t2_root.trace_id = "t2".to_string();
    let mut t2_child = span("b", Some("a"), "api", SpanKind::Server, 10, 50, false);
    t2_child.trace_id = "t2".to_string();
    let spans = vec![
        span("a", None, "web", SpanKind::Server, 0, 100, false),
        t2_root,
        t2_child,
    ];
    let flow = sankey_flow(&spans, "other");
    let web = flow
        .nodes
        .iter()
        .find(|n| n.service == "web" && n.stage == 0)
        .expect("the roots stage");
    assert_eq!(web.value, 2);
    assert_eq!(web.outflow, 1);
    // Empty input is an empty flow.
    let flow = sankey_flow(&[], "other");
    assert_eq!(flow.stages, 0);
    assert!(flow.nodes.is_empty());
    assert_eq!(flow.residual_spans, 0);
}

// ----------------------------------------------------------------
// Round 4 scale proof. The recipes are fixed arithmetic, so the
// shapes below are the same on every run and every machine. The
// ceilings are provisional until the first hosted run measures
// the real times, when INV-70 sets each at ten times measured.

const SCALE_CEILING: std::time::Duration = std::time::Duration::from_secs(60);

const SCALE_SERVICES: [&str; 8] = [
    "web", "api", "db", "cache", "queue", "worker", "auth", "search",
];

/// Ten thousand spans in one trace. A chain one thousand deep,
/// one hundred hub roots with eighty children each, and nine
/// hundred orphans naming a parent the data does not contain.
fn scale_spans_single_trace() -> Vec<Span> {
    let mut spans = Vec::with_capacity(10_000);
    for i in 0..1000 {
        let parent = (i > 0).then(|| format!("c{:04}", i - 1));
        spans.push(span(
            &format!("c{i:04}"),
            parent.as_deref(),
            SCALE_SERVICES[i % 8],
            SpanKind::Server,
            i as i64,
            i as i64 + 5,
            false,
        ));
    }
    for h in 0..100 {
        let hub = format!("h{h:03}");
        spans.push(span(
            &hub,
            None,
            SCALE_SERVICES[h % 8],
            SpanKind::Server,
            0,
            500,
            false,
        ));
        for j in 0..80 {
            spans.push(span(
                &format!("k{h:03}-{j:02}"),
                Some(&hub),
                SCALE_SERVICES[(h + j) % 8],
                SpanKind::Client,
                j as i64,
                j as i64 + 3,
                false,
            ));
        }
    }
    for o in 0..900 {
        spans.push(span(
            &format!("o{o:03}"),
            Some("absent"),
            SCALE_SERVICES[o % 8],
            SpanKind::Server,
            0,
            2,
            false,
        ));
    }
    assert_eq!(spans.len(), 10_000);
    spans
}

#[test]
fn sankey_flow_conserves_at_ten_thousand_spans() {
    let spans = scale_spans_single_trace();
    let started = std::time::Instant::now();
    let flow = sankey_flow(&spans, "other");
    assert!(
        started.elapsed() < SCALE_CEILING,
        "sankey at scale took {:?}",
        started.elapsed()
    );
    // The chain stages one thousand deep, and every span is
    // counted exactly once across the real nodes.
    assert_eq!(flow.stages, 1000);
    let counted: u64 = flow
        .nodes
        .iter()
        .filter(|n| !n.residual)
        .map(|n| n.value)
        .sum();
    assert_eq!(counted, 10_000);
    // The nine hundred orphans enter through the residual.
    assert_eq!(flow.residual_spans, 900);
    // Total inflow plus staged roots equals the span count.
    let inflow: u64 = flow.links.iter().map(|l| l.value).sum();
    let roots = spans.iter().filter(|s| s.parent_span_id.is_none()).count() as u64;
    assert_eq!(inflow + roots, 10_000);
}

#[test]
fn sankey_flow_holds_across_a_thousand_traces() {
    // The same span count spread over one thousand traces, the
    // shape that convicts or clears the staging walk's per-trace
    // index rebuild named in docs/SCALE-DESIGN.md.
    let mut spans = Vec::with_capacity(10_000);
    for t in 0..1000 {
        let trace = format!("mt{t:04}");
        for i in 0..10 {
            let mut s = span(
                &format!("s{i}"),
                (i > 0).then(|| format!("s{}", i - 1)).as_deref(),
                SCALE_SERVICES[i % 8],
                SpanKind::Server,
                i as i64,
                i as i64 + 2,
                false,
            );
            s.trace_id = trace.clone();
            spans.push(s);
        }
    }
    assert_eq!(spans.len(), 10_000);
    let started = std::time::Instant::now();
    let flow = sankey_flow(&spans, "other");
    assert!(
        started.elapsed() < SCALE_CEILING,
        "many-trace sankey took {:?}",
        started.elapsed()
    );
    let counted: u64 = flow
        .nodes
        .iter()
        .filter(|n| !n.residual)
        .map(|n| n.value)
        .sum();
    assert_eq!(counted, 10_000);
    assert_eq!(flow.residual_spans, 0);
    assert_eq!(flow.stages, 10);
}

#[test]
fn profile_holds_at_a_hundred_thousand_samples() {
    // Two hundred stacks whose counts sum to exactly one
    // hundred thousand samples, the last stack taking the
    // remainder so the total is exact by construction.
    let mut stacks: Vec<StackSample> = Vec::new();
    let mut assigned: u64 = 0;
    for i in 0..199 {
        let count = (i * 37 % 400) + 100;
        assigned += count;
        stacks.push(StackSample {
            frames: vec![
                "root".to_string(),
                format!("svc-{}", i % 10),
                format!("fn-{i}"),
            ],
            count,
        });
    }
    stacks.push(StackSample {
        frames: vec!["root".to_string(), "svc-0".to_string(), "fn-last".to_string()],
        count: 100_000 - assigned,
    });
    let started = std::time::Instant::now();
    let tree = flame_tree(&stacks);
    assert_eq!(tree.value, 100_000);
    // The heatmap over the trace fixture accounts for every
    // span, in a bucket or in the outside count.
    let spans = scale_spans_single_trace();
    let grid = heatmap_grid(
        &spans,
        TimeRange {
            from_unix_ms: 0,
            to_unix_ms: 20_000,
        },
        &[0.001, 0.005, 0.010],
        10,
    );
    let bucketed: u64 = grid.rows.iter().map(|r| r.total).sum();
    assert_eq!(bucketed + grid.outside_window as u64, 10_000);
    assert_eq!(grid.outside_window, 0);
    assert!(
        started.elapsed() < SCALE_CEILING,
        "profile at scale took {:?}",
        started.elapsed()
    );
}
