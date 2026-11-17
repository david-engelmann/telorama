//! Pure derivations from spans to panel-ready facts.
//!
//! Every function here is deterministic: inputs are sorted by stable keys
//! before any aggregation, maps are `BTreeMap`s, and no iteration order
//! from a hash map can reach an output.

use crate::profile::StackSample;
use crate::trace::{Span, SpanKind};
use std::collections::BTreeMap;

/// One row of a trace waterfall, in display order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaterfallRow {
    pub span_id: String,
    pub depth: usize,
    /// Time the span spent in its own code: its duration minus the parts
    /// covered by its direct children.
    pub self_time_nanos: i64,
}

/// Spans of one trace, sorted by the canonical key (start, span id).
fn trace_spans<'a>(spans: &'a [Span], trace_id: &str) -> Vec<&'a Span> {
    let mut out: Vec<&Span> = spans.iter().filter(|s| s.trace_id == trace_id).collect();
    out.sort_by(|a, b| {
        a.start_unix_nano
            .cmp(&b.start_unix_nano)
            .then_with(|| a.span_id.cmp(&b.span_id))
    });
    out
}

/// Children of each span in a trace, keyed by parent span id, each list in
/// canonical order.
fn children_map<'a>(spans: &[&'a Span]) -> BTreeMap<String, Vec<&'a Span>> {
    let ids: std::collections::BTreeSet<&str> = spans.iter().map(|s| s.span_id.as_str()).collect();
    let mut map: BTreeMap<String, Vec<&Span>> = BTreeMap::new();
    for span in spans {
        if let Some(parent) = span.parent_span_id.as_deref()
            && ids.contains(parent)
        {
            map.entry(parent.to_string()).or_default().push(span);
        }
    }
    map
}

/// Self time of one span: its duration minus the union of its direct
/// children's intervals, clipped to the span's own interval.
fn self_time(span: &Span, children: &[&Span]) -> i64 {
    if children.is_empty() {
        return span.duration_nanos();
    }
    let mut intervals: Vec<(i64, i64)> = children
        .iter()
        .map(|c| {
            (
                c.start_unix_nano.max(span.start_unix_nano),
                c.end_unix_nano.min(span.end_unix_nano),
            )
        })
        .filter(|(a, b)| b > a)
        .collect();
    intervals.sort();
    let mut covered: i64 = 0;
    let mut cur: Option<(i64, i64)> = None;
    for (a, b) in intervals {
        match cur {
            None => cur = Some((a, b)),
            Some((ca, cb)) if a <= cb => cur = Some((ca, cb.max(b))),
            Some((ca, cb)) => {
                covered += cb - ca;
                cur = Some((a, b));
            }
        }
    }
    if let Some((ca, cb)) = cur {
        covered += cb - ca;
    }
    (span.duration_nanos() - covered).max(0)
}

/// Build the waterfall rows for one trace: depth-first order, children in
/// canonical order, roots (spans whose parent is absent or outside the
/// trace) first in canonical order. Missing parents render as roots
/// rather than vanishing: a gap in the data stays visible as structure.
pub fn waterfall_rows(spans: &[Span], trace_id: &str) -> Vec<WaterfallRow> {
    let owned = trace_spans(spans, trace_id);
    let children = children_map(&owned);
    let ids: std::collections::BTreeSet<&str> = owned.iter().map(|s| s.span_id.as_str()).collect();
    let roots: Vec<&Span> = owned
        .iter()
        .copied()
        .filter(|s| match s.parent_span_id.as_deref() {
            None => true,
            Some(p) => !ids.contains(p),
        })
        .collect();

    let mut rows = Vec::with_capacity(owned.len());
    // Iterative depth-first walk so deep traces cannot overflow the stack.
    let mut stack: Vec<(&Span, usize)> = roots.iter().rev().map(|s| (*s, 0usize)).collect();
    while let Some((span, depth)) = stack.pop() {
        let kids: Vec<&Span> = children.get(&span.span_id).cloned().unwrap_or_default();
        rows.push(WaterfallRow {
            span_id: span.span_id.clone(),
            depth,
            self_time_nanos: self_time(span, &kids),
        });
        for kid in kids.iter().rev() {
            stack.push((kid, depth + 1));
        }
    }
    rows
}

/// The critical path of a trace, as span ids from root to leaf.
///
/// The walk follows the CRISP idea in tree form: start at the root, then
/// repeatedly move to the child that finishes last, because that child is
/// the one holding the parent open at its end. Shortening any span on
/// this chain shortens the trace; shortening anything off it does not.
/// Ties break on span id so the result is a pure function of the trace.
pub fn critical_path(spans: &[Span], trace_id: &str) -> Vec<String> {
    let owned = trace_spans(spans, trace_id);
    if owned.is_empty() {
        return Vec::new();
    }
    let children = children_map(&owned);
    let ids: std::collections::BTreeSet<&str> = owned.iter().map(|s| s.span_id.as_str()).collect();
    let root = owned
        .iter()
        .copied()
        .filter(|s| match s.parent_span_id.as_deref() {
            None => true,
            Some(p) => !ids.contains(p),
        })
        .min_by(|a, b| {
            a.start_unix_nano
                .cmp(&b.start_unix_nano)
                .then_with(|| a.span_id.cmp(&b.span_id))
        });
    let Some(mut current) = root else {
        return Vec::new();
    };
    let mut path = vec![current.span_id.clone()];
    loop {
        let kids = children.get(&current.span_id);
        let next = kids.and_then(|list| {
            list.iter().max_by(|a, b| {
                a.end_unix_nano
                    .cmp(&b.end_unix_nano)
                    .then_with(|| b.span_id.cmp(&a.span_id))
            })
        });
        match next {
            Some(child) => {
                path.push(child.span_id.clone());
                current = child;
            }
            None => break,
        }
    }
    path
}

/// Nearest-rank percentile over nanosecond durations. Returns `None` for
/// an empty input rather than inventing a zero.
pub fn percentile_nanos(values: &[i64], percentile: u32) -> Option<i64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let p = percentile.clamp(1, 100) as usize;
    let rank = (p * sorted.len()).div_ceil(100).max(1);
    Some(sorted[rank - 1])
}

/// RED figures for one service: rate, errors, duration.
#[derive(Debug, Clone, PartialEq)]
pub struct RedStats {
    pub service: String,
    pub calls: u64,
    pub errors: u64,
    pub error_ratio: f64,
    pub rate_per_sec: f64,
    pub p50_nanos: i64,
    pub p95_nanos: i64,
    pub p99_nanos: i64,
    /// Which spans were counted: `server` when the service has server
    /// spans (the honest basis, one count per request it served), or
    /// `all` as a declared fallback for services with none.
    pub basis: &'static str,
}

/// RED figures per service over a window, sorted by service name.
///
/// Counting rule: a service with server spans is counted on those spans
/// alone, so one request served counts once even when the same trace also
/// carries the caller's client span. A service with no server spans is
/// counted on all of its spans and says so in `basis`.
pub fn red_by_service(spans: &[Span], window_seconds: f64) -> Vec<RedStats> {
    let mut by_service: BTreeMap<&str, Vec<&Span>> = BTreeMap::new();
    for span in spans {
        by_service
            .entry(span.service.as_str())
            .or_default()
            .push(span);
    }
    let mut out = Vec::with_capacity(by_service.len());
    for (service, group) in by_service {
        let servers: Vec<&Span> = group
            .iter()
            .copied()
            .filter(|s| s.kind == SpanKind::Server)
            .collect();
        let (counted, basis): (Vec<&Span>, &'static str) = if servers.is_empty() {
            (group, "all")
        } else {
            (servers, "server")
        };
        let calls = counted.len() as u64;
        let errors = counted.iter().filter(|s| s.is_error).count() as u64;
        let durations: Vec<i64> = counted.iter().map(|s| s.duration_nanos()).collect();
        out.push(RedStats {
            service: service.to_string(),
            calls,
            errors,
            error_ratio: if calls == 0 {
                0.0
            } else {
                errors as f64 / calls as f64
            },
            rate_per_sec: if window_seconds > 0.0 {
                calls as f64 / window_seconds
            } else {
                0.0
            },
            p50_nanos: percentile_nanos(&durations, 50).unwrap_or(0),
            p95_nanos: percentile_nanos(&durations, 95).unwrap_or(0),
            p99_nanos: percentile_nanos(&durations, 99).unwrap_or(0),
            basis,
        });
    }
    out
}

/// A content hash for element ids and artifact identity: FNV-1a 64 over
/// the bytes, rendered as 16 lowercase hex characters. This is an identity
/// label, not a security primitive.
pub fn content_hash(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

/// One row of a latency heatmap: the spans whose duration falls in the
/// band ending at this row's edge, counted per time column.
#[derive(Debug, Clone, PartialEq)]
pub struct HeatmapRow {
    /// Upper bucket edge in seconds; `None` is the overflow row above
    /// the highest declared edge.
    pub upper_edge_seconds: Option<f64>,
    pub counts: Vec<u64>,
    pub total: u64,
}

/// A real span from the rendered set, named as the exemplar of the
/// busiest cell: the correlation link from a bucket back to one trace
/// that actually happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exemplar {
    pub trace_id: String,
    pub span_id: String,
    pub service: String,
    pub name: String,
    pub duration_nanos: i64,
    pub row: usize,
    pub col: usize,
}

/// The derived heatmap grid over a span set.
#[derive(Debug, Clone, PartialEq)]
pub struct HeatmapGrid {
    /// Rows in ascending bucket order: row 0 is the lowest band and the
    /// last row is the overflow row.
    pub rows: Vec<HeatmapRow>,
    pub columns: usize,
    pub max_count: u64,
    pub outside_window: usize,
    pub exemplar: Option<Exemplar>,
}

/// Bucket spans by duration against declared edges and by start time
/// across the window.
///
/// A span lands in the first bucket whose edge is at or above its
/// duration in seconds (the Prometheus `le` convention), or in the
/// overflow row when it exceeds every edge. Columns are equal divisions
/// of the window by span start. Spans starting outside the window are
/// excluded and counted, never silently dropped. The exemplar is the
/// longest span in the busiest cell, scanning rows in ascending bucket
/// order and columns left to right, with span id as the final
/// tie-break: a stated rule, reproducible from the data alone.
pub fn heatmap_grid(
    spans: &[Span],
    range: crate::spec::TimeRange,
    edges_seconds: &[f64],
    columns: usize,
) -> HeatmapGrid {
    let columns = columns.max(1);
    let row_count = edges_seconds.len() + 1;
    let mut counts = vec![vec![0u64; columns]; row_count];
    let from_nano = range.from_unix_ms * 1_000_000;
    let window_nano = (range.to_unix_ms - range.from_unix_ms).max(1) * 1_000_000;
    let mut outside_window = 0usize;
    // The longest span seen per cell, the exemplar candidates.
    let mut best: BTreeMap<(usize, usize), &Span> = BTreeMap::new();
    let mut sorted: Vec<&Span> = spans.iter().collect();
    sorted.sort_by(|a, b| a.span_id.cmp(&b.span_id));
    for span in sorted {
        if span.start_unix_nano < from_nano || span.start_unix_nano >= from_nano + window_nano {
            outside_window += 1;
            continue;
        }
        let col = (((span.start_unix_nano - from_nano) * columns as i64) / window_nano) as usize;
        let col = col.min(columns - 1);
        let duration_seconds = span.duration_nanos() as f64 / 1_000_000_000.0;
        let row = edges_seconds
            .iter()
            .position(|edge| duration_seconds <= *edge)
            .unwrap_or(edges_seconds.len());
        counts[row][col] += 1;
        let entry = best.entry((row, col)).or_insert(span);
        if span.duration_nanos() > entry.duration_nanos() {
            *entry = span;
        }
    }
    let mut rows: Vec<HeatmapRow> = Vec::with_capacity(row_count);
    let mut max_count = 0u64;
    for (r, row_counts) in counts.iter().enumerate() {
        let total: u64 = row_counts.iter().sum();
        if let Some(m) = row_counts.iter().max() {
            max_count = max_count.max(*m);
        }
        rows.push(HeatmapRow {
            upper_edge_seconds: edges_seconds.get(r).copied(),
            counts: row_counts.clone(),
            total,
        });
    }
    let exemplar = if max_count == 0 {
        None
    } else {
        let mut found = None;
        'scan: for r in 0..row_count {
            for c in 0..columns {
                if counts[r][c] == max_count {
                    let span = best[&(r, c)];
                    found = Some(Exemplar {
                        trace_id: span.trace_id.clone(),
                        span_id: span.span_id.clone(),
                        service: span.service.clone(),
                        name: span.name.clone(),
                        duration_nanos: span.duration_nanos(),
                        row: r,
                        col: c,
                    });
                    break 'scan;
                }
            }
        }
        found
    };
    HeatmapGrid {
        rows,
        columns,
        max_count,
        outside_window,
        exemplar,
    }
}

/// One node of a flame tree: a frame, the samples that passed through
/// it (its own samples plus every descendant's), and its children in
/// display order: samples descending, frame name as the tie-break.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlameNode {
    pub name: String,
    pub value: u64,
    pub children: Vec<FlameNode>,
}

fn sort_flame_children(children: &mut [FlameNode]) {
    children.sort_by(|a, b| b.value.cmp(&a.value).then_with(|| a.name.cmp(&b.name)));
}

struct FlameBuilder {
    name: String,
    value: u64,
    children: BTreeMap<String, FlameBuilder>,
}

impl FlameBuilder {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            value: 0,
            children: BTreeMap::new(),
        }
    }

    fn insert(&mut self, frames: &[String], count: u64) {
        self.value += count;
        if let Some((first, rest)) = frames.split_first() {
            self.children
                .entry(first.clone())
                .or_insert_with(|| FlameBuilder::new(first))
                .insert(rest, count);
        }
    }

    fn freeze(self) -> FlameNode {
        let mut children: Vec<FlameNode> = self
            .children
            .into_values()
            .map(FlameBuilder::freeze)
            .collect();
        sort_flame_children(&mut children);
        FlameNode {
            name: self.name,
            value: self.value,
            children,
        }
    }
}

/// Fold stacks into a flame tree under a synthetic root whose value is
/// the total sample count.
pub fn flame_tree(stacks: &[StackSample]) -> FlameNode {
    let mut root = FlameBuilder::new("root");
    for stack in stacks {
        root.insert(&stack.frames, stack.count);
    }
    root.freeze()
}

/// The labelled span fallback: a flame tree built from one trace, where
/// a span's weight is its self time in nanoseconds and a node's value
/// is its self time plus its children's values. This is a derivation
/// from spans, not a sampled profile, and renders must say so.
pub fn flame_from_spans(spans: &[Span], trace_id: &str) -> FlameNode {
    fn build(span: &Span, children: &BTreeMap<String, Vec<&Span>>) -> FlameNode {
        let kids: Vec<&Span> = children.get(&span.span_id).cloned().unwrap_or_default();
        let mut child_nodes: Vec<FlameNode> = kids.iter().map(|k| build(k, children)).collect();
        sort_flame_children(&mut child_nodes);
        let self_nanos = self_time(span, &kids).max(0) as u64;
        let value = self_nanos + child_nodes.iter().map(|c| c.value).sum::<u64>();
        FlameNode {
            name: format!("{} · {}", span.service, span.name),
            value,
            children: child_nodes,
        }
    }
    let trace = trace_spans(spans, trace_id);
    let children = children_map(&trace);
    let ids: std::collections::BTreeSet<&str> = trace.iter().map(|s| s.span_id.as_str()).collect();
    let mut root = FlameNode {
        name: "trace".to_string(),
        value: 0,
        children: Vec::new(),
    };
    for span in &trace {
        let is_root = span
            .parent_span_id
            .as_deref()
            .is_none_or(|p| !ids.contains(p));
        if is_root {
            let node = build(span, &children);
            root.value += node.value;
            root.children.push(node);
        }
    }
    sort_flame_children(&mut root.children);
    root
}
