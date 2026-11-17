//! Property tests over the derivations, driven by a deterministic
//! SplitMix64 generator so every run checks the same thousand cases and
//! any failure reproduces from its seed alone. No external crates: the
//! dependency discipline in DECISIONS.md binds tests too.

use telorama_scene::{Span, SpanKind, critical_path, percentile_nanos, red_by_service, waterfall_rows};
use std::collections::BTreeMap;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }
}

fn random_trace(rng: &mut Rng, count: usize) -> Vec<Span> {
    let kinds = [
        SpanKind::Internal,
        SpanKind::Server,
        SpanKind::Client,
        SpanKind::Producer,
        SpanKind::Consumer,
    ];
    let mut spans = Vec::with_capacity(count);
    for i in 0..count {
        let start = rng.below(10_000) as i64 * 1_000;
        let duration = (rng.below(500) as i64 + 1) * 1_000;
        let parent = if i == 0 || rng.below(3) == 0 {
            None
        } else {
            Some(format!("s{}", rng.below(i as u64)))
        };
        spans.push(Span {
            trace_id: "prop".to_string(),
            span_id: format!("s{i}"),
            parent_span_id: parent,
            name: format!("op-{i}"),
            kind: kinds[(rng.below(5)) as usize],
            start_unix_nano: start,
            end_unix_nano: start + duration,
            is_error: rng.below(7) == 0,
            service: format!("svc-{}", rng.below(4)),
            attributes: BTreeMap::new(),
        });
    }
    spans
}

#[test]
fn critical_path_is_always_a_root_to_leaf_chain() {
    for seed in 0..1000u64 {
        let mut rng = Rng(seed.wrapping_mul(0x2545F4914F6CDD1D) | 1);
        let spans = random_trace(&mut rng, 1 + (seed % 40) as usize);
        let by_id: BTreeMap<&str, &Span> = spans.iter().map(|s| (s.span_id.as_str(), s)).collect();
        let path = critical_path(&spans, "prop");
        if spans.is_empty() {
            assert!(path.is_empty());
            continue;
        }
        assert!(!path.is_empty(), "seed {seed}: non-empty trace needs a path");
        // Starts at a root.
        let first = by_id[path[0].as_str()];
        assert!(
            first.parent_span_id.as_deref().map(|p| !by_id.contains_key(p)).unwrap_or(true),
            "seed {seed}: path must start at a root"
        );
        // Each step is a parent-to-child step.
        for pair in path.windows(2) {
            let child = by_id[pair[1].as_str()];
            assert_eq!(
                child.parent_span_id.as_deref(),
                Some(pair[0].as_str()),
                "seed {seed}: path step {} -> {} is not a parent-child step",
                pair[0],
                pair[1]
            );
        }
        // Ends at a leaf.
        let last = path[path.len() - 1].as_str();
        assert!(
            !spans.iter().any(|s| s.parent_span_id.as_deref() == Some(last)),
            "seed {seed}: path must end at a leaf"
        );
    }
}

#[test]
fn self_time_stays_within_duration() {
    for seed in 0..1000u64 {
        let mut rng = Rng(seed ^ 0xA0761D6478BD642F);
        let spans = random_trace(&mut rng, 1 + (seed % 30) as usize);
        let by_id: BTreeMap<&str, &Span> = spans.iter().map(|s| (s.span_id.as_str(), s)).collect();
        for row in waterfall_rows(&spans, "prop") {
            let span = by_id[row.span_id.as_str()];
            assert!(
                row.self_time_nanos >= 0 && row.self_time_nanos <= span.duration_nanos(),
                "seed {seed}: self time {} outside duration {}",
                row.self_time_nanos,
                span.duration_nanos()
            );
        }
    }
}

#[test]
fn percentiles_are_monotone_and_red_ratios_are_bounded() {
    for seed in 0..1000u64 {
        let mut rng = Rng(seed.wrapping_add(0xE7037ED1A0B428DB));
        let spans = random_trace(&mut rng, 1 + (seed % 30) as usize);
        let durations: Vec<i64> = spans.iter().map(|s| s.duration_nanos()).collect();
        let p50 = percentile_nanos(&durations, 50).unwrap();
        let p95 = percentile_nanos(&durations, 95).unwrap();
        let p99 = percentile_nanos(&durations, 99).unwrap();
        assert!(p50 <= p95 && p95 <= p99, "seed {seed}: percentiles out of order");
        for stats in red_by_service(&spans, 60.0) {
            assert!(
                (0.0..=1.0).contains(&stats.error_ratio),
                "seed {seed}: error ratio {} out of bounds",
                stats.error_ratio
            );
            assert!(stats.p50_nanos <= stats.p95_nanos && stats.p95_nanos <= stats.p99_nanos);
        }
    }
}
