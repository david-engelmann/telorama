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
    /// Id of the span drawn on this row.
    pub span_id: String,
    /// Position of the span occurrence this row was built
    /// from in the slice the caller passed. A renderer joins
    /// back to the span on this index, never on the id alone,
    /// because one trace can carry two spans under one id
    /// and each copy gets its own row (INV-89).
    pub span_index: usize,
    /// Nesting depth in the trace tree, with roots at 0 and
    /// each parent step adding one.
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

/// Spans of one trace with their positions in the caller's
/// slice, sorted by the canonical key (start, span id), with
/// input order breaking full ties. Attribution is by
/// occurrence, so a repeated span id yields one entry per
/// copy (INV-89).
fn trace_occurrences<'a>(spans: &'a [Span], trace_id: &str) -> Vec<(usize, &'a Span)> {
    let mut out: Vec<(usize, &Span)> = spans
        .iter()
        .enumerate()
        .filter(|(_, s)| s.trace_id == trace_id)
        .collect();
    out.sort_by(|a, b| {
        a.1.start_unix_nano
            .cmp(&b.1.start_unix_nano)
            .then_with(|| a.1.span_id.cmp(&b.1.span_id))
    });
    out
}

/// Children of each span id in a trace, as positions in the
/// occurrence list, in canonical order. The children of a
/// repeated id hang from its adopting copy alone, because
/// the copies cannot be told apart as parents (INV-89).
fn occurrence_children(occurrences: &[(usize, &Span)]) -> BTreeMap<String, Vec<usize>> {
    let ids: std::collections::BTreeSet<&str> = occurrences
        .iter()
        .map(|(_, s)| s.span_id.as_str())
        .collect();
    let mut map: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (pos, (_, span)) in occurrences.iter().enumerate() {
        if let Some(parent) = span.parent_span_id.as_deref()
            && ids.contains(parent)
        {
            map.entry(parent.to_string()).or_default().push(pos);
        }
    }
    map
}

/// The adopting copy of each span id in a trace, as a
/// position in the occurrence list (INV-89). The children of
/// an id hang from its first occurrence in canonical order,
/// because the copies cannot be told apart as parents. When
/// that copy cannot be reached from the trace's roots,
/// because its own parent chain cycles, the first reached
/// copy adopts instead, so the children of a shared id are
/// not lost with an unreachable copy. A copy no walk reaches
/// adopts nothing and builds nothing, the standing fate of
/// spans in a parent cycle (INV-82).
fn adopting_copies(
    occurrences: &[(usize, &Span)],
    children: &BTreeMap<String, Vec<usize>>,
) -> BTreeMap<String, usize> {
    let ids: std::collections::BTreeSet<&str> = occurrences
        .iter()
        .map(|(_, s)| s.span_id.as_str())
        .collect();
    let mut adopters: BTreeMap<String, usize> = BTreeMap::new();
    for (pos, (_, span)) in occurrences.iter().enumerate() {
        adopters.entry(span.span_id.clone()).or_insert(pos);
    }
    loop {
        // The closure the current adopters reach from the
        // roots of the trace.
        let mut reached: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
        let mut stack: Vec<usize> = occurrences
            .iter()
            .enumerate()
            .filter(|(_, (_, s))| match s.parent_span_id.as_deref() {
                None => true,
                Some(p) => !ids.contains(p),
            })
            .map(|(pos, _)| pos)
            .collect();
        while let Some(pos) = stack.pop() {
            if !reached.insert(pos) {
                continue;
            }
            let span = occurrences[pos].1;
            if adopters.get(span.span_id.as_str()) == Some(&pos)
                && let Some(list) = children.get(&span.span_id)
            {
                stack.extend(list.iter().copied());
            }
        }
        // Promote the first reached copy of any id whose
        // adopter went unreached, then close again. An
        // adopter only ever moves from an unreached copy to
        // a reached one, so the loop ends.
        let mut promoted = false;
        for (pos, (_, span)) in occurrences.iter().enumerate() {
            let adopter = adopters[span.span_id.as_str()];
            if !reached.contains(&adopter) && reached.contains(&pos) {
                adopters.insert(span.span_id.clone(), pos);
                promoted = true;
            }
        }
        if !promoted {
            return adopters;
        }
    }
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
                covered = covered.saturating_add(cb.saturating_sub(ca));
                cur = Some((a, b));
            }
        }
    }
    if let Some((ca, cb)) = cur {
        covered = covered.saturating_add(cb.saturating_sub(ca));
    }
    span.duration_nanos().saturating_sub(covered).max(0)
}

/// Build the waterfall rows for one trace: depth-first order, children in
/// canonical order, roots (spans whose parent is absent or outside the
/// trace) first in canonical order. Missing parents render as roots
/// rather than vanishing: a gap in the data stays visible as structure.
/// Every reached span occurrence is emitted once, tracked by its
/// position in the caller's slice, so a repeated id loses no reached
/// copy (INV-89) and any cycle the parent links form still
/// terminates (INV-82).
pub fn waterfall_rows(spans: &[Span], trace_id: &str) -> Vec<WaterfallRow> {
    let owned = trace_occurrences(spans, trace_id);
    let children = occurrence_children(&owned);
    let adopters = adopting_copies(&owned, &children);
    let ids: std::collections::BTreeSet<&str> =
        owned.iter().map(|(_, s)| s.span_id.as_str()).collect();
    let roots: Vec<usize> = owned
        .iter()
        .enumerate()
        .filter(|(_, (_, s))| match s.parent_span_id.as_deref() {
            None => true,
            Some(p) => !ids.contains(p),
        })
        .map(|(pos, _)| pos)
        .collect();

    let mut rows = Vec::with_capacity(owned.len());
    // Iterative depth-first walk so deep traces cannot overflow the stack.
    let mut seen: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
    let mut stack: Vec<(usize, usize)> = roots.iter().rev().map(|&pos| (pos, 0usize)).collect();
    while let Some((pos, depth)) = stack.pop() {
        let (index, span) = owned[pos];
        if !seen.insert(index) {
            continue;
        }
        let kids: Vec<&Span> = if adopters.get(span.span_id.as_str()) == Some(&pos) {
            children
                .get(&span.span_id)
                .map(|list| list.iter().map(|&p| owned[p].1).collect())
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        rows.push(WaterfallRow {
            span_id: span.span_id.clone(),
            span_index: index,
            depth,
            self_time_nanos: self_time(span, &kids),
        });
        if adopters.get(span.span_id.as_str()) == Some(&pos)
            && let Some(list) = children.get(&span.span_id)
        {
            for &kid_pos in list.iter().rev() {
                stack.push((kid_pos, depth + 1));
            }
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
/// The walk stops at the first repeated span id, so a cyclic trace
/// yields the longest walk that visits no span twice instead of
/// looping forever (INV-82).
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
    let mut seen: std::collections::BTreeSet<String> = path.iter().cloned().collect();
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
            Some(child) if seen.insert(child.span_id.clone()) => {
                path.push(child.span_id.clone());
                current = child;
            }
            _ => break,
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
    /// Service these figures roll up, one entry per service in
    /// the span set.
    pub service: String,
    /// Spans counted for the service under the basis rule, its
    /// server spans when it has any and all of its spans
    /// otherwise.
    pub calls: u64,
    /// How many of the counted spans carry an error status.
    pub errors: u64,
    /// Errors divided by calls as a fraction from 0 to 1, or 0
    /// when nothing was counted.
    pub error_ratio: f64,
    /// Counted spans per second across the window the caller
    /// passed, or 0 when that window was not positive.
    pub rate_per_sec: f64,
    /// Median duration of the counted spans in nanoseconds by
    /// nearest rank, or 0 when nothing was counted.
    pub p50_nanos: i64,
    /// 95th-percentile duration of the counted spans in
    /// nanoseconds by nearest rank, or 0 when nothing was
    /// counted.
    pub p95_nanos: i64,
    /// 99th-percentile duration of the counted spans in
    /// nanoseconds by nearest rank, or 0 when nothing was
    /// counted.
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
    /// Span counts per time column in column order, one entry
    /// per column of the grid.
    pub counts: Vec<u64>,
    /// Sum of the column counts, every span in this band
    /// across the whole window.
    pub total: u64,
}

/// A real span from the rendered set, named as the exemplar of the
/// busiest cell: the correlation link from a bucket back to one trace
/// that actually happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exemplar {
    /// Trace the exemplar span belongs to, half of the lookup
    /// key back into the raw spans.
    pub trace_id: String,
    /// Id of the exemplar span within its trace.
    pub span_id: String,
    /// Service the exemplar span ran on.
    pub service: String,
    /// Operation name of the exemplar span.
    pub name: String,
    /// Duration of the exemplar span in nanoseconds, the
    /// longest in its cell.
    pub duration_nanos: i64,
    /// Row of the busiest cell this exemplar came from, in
    /// the grid's ascending bucket order.
    pub row: usize,
    /// Column of the busiest cell this exemplar came from.
    pub col: usize,
}

/// The derived heatmap grid over a span set.
#[derive(Debug, Clone, PartialEq)]
pub struct HeatmapGrid {
    /// Rows in ascending bucket order: row 0 is the lowest band and the
    /// last row is the overflow row.
    pub rows: Vec<HeatmapRow>,
    /// Equal time divisions the window was split into, and
    /// the length of every row's counts vector.
    pub columns: usize,
    /// Largest count in any single cell, or 0 when no span
    /// fell inside the window.
    pub max_count: u64,
    /// Spans excluded because their start fell outside the
    /// window, counted here rather than dropped silently.
    pub outside_window: usize,
    /// Longest span in the busiest cell, or `None` when no
    /// span fell inside the window.
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
    let from_nano = range.from_unix_ms.saturating_mul(1_000_000);
    let window_nano = range
        .to_unix_ms
        .saturating_sub(range.from_unix_ms)
        .max(1)
        .saturating_mul(1_000_000);
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
        let col = ((span
            .start_unix_nano
            .saturating_sub(from_nano)
            .saturating_mul(columns as i64))
            / window_nano) as usize;
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
    /// Frame label, a stack frame for profile trees or
    /// service and operation for span-derived trees, with
    /// `root` and `trace` as the synthetic root labels.
    pub name: String,
    /// Weight passing through this frame, its own plus every
    /// descendant's, in samples for profile trees and in
    /// self-time nanoseconds for span-derived trees.
    pub value: u64,
    /// Child frames in display order, value descending with
    /// the frame name as tie-break.
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
        // A loop, not recursion (INV-87): one stack in a folded
        // profile can run tens of thousands of frames deep.
        let mut node = self;
        node.value = node.value.saturating_add(count);
        for frame in frames {
            if !node.children.contains_key(frame) {
                node.children
                    .insert(frame.clone(), FlameBuilder::new(frame));
            }
            node = node
                .children
                .get_mut(frame)
                .expect("the child was just ensured");
            node.value = node.value.saturating_add(count);
        }
    }

    fn freeze(self) -> FlameNode {
        // An explicit stack, not recursion (INV-87), assembling
        // children before their parent, the order the recursive
        // freeze worked in.
        struct Work {
            builder: FlameBuilder,
            frozen: Vec<FlameNode>,
        }
        let mut stack: Vec<Work> = vec![Work {
            builder: self,
            frozen: Vec::new(),
        }];
        loop {
            let next_child = stack
                .last_mut()
                .expect("a freeze in progress")
                .builder
                .children
                .pop_first();
            match next_child {
                Some((_, child)) => stack.push(Work {
                    builder: child,
                    frozen: Vec::new(),
                }),
                None => {
                    let work = stack.pop().expect("a freeze in progress");
                    let mut children = work.frozen;
                    sort_flame_children(&mut children);
                    let node = FlameNode {
                        name: work.builder.name,
                        value: work.builder.value,
                        children,
                    };
                    match stack.last_mut() {
                        Some(parent) => parent.frozen.push(node),
                        None => return node,
                    }
                }
            }
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

/// Take a flame tree apart level by level. Dropping a very
/// deep tree recurses through the drop glue and overflows the
/// stack (INV-87), so a consumer that owns one dismantles it
/// once it has read it.
pub fn dismantle_flame_tree(tree: &mut FlameNode) {
    let mut pending: Vec<FlameNode> = std::mem::take(&mut tree.children);
    while let Some(mut node) = pending.pop() {
        pending.append(&mut node.children);
    }
}

/// The labelled span fallback: a flame tree built from one trace, where
/// a span's weight is its self time in nanoseconds and a node's value
/// is its self time plus its children's values. This is a derivation
/// from spans, not a sampled profile, and renders must say so.
pub fn flame_from_spans(spans: &[Span], trace_id: &str) -> FlameNode {
    // The builder runs on an explicit stack instead of recursion,
    // so a very deep trace cannot overflow the call stack
    // (INV-86). Every reached span occurrence builds once, tracked
    // by its position in the caller's slice, so a cycle through
    // repeated ids stops at the first build (INV-82) and a
    // repeated id loses no reached copy (INV-89).
    struct Frame<'a> {
        span: &'a Span,
        kids: Vec<usize>,
        next: usize,
        built: Vec<FlameNode>,
    }
    fn frame_for<'a>(
        pos: usize,
        owned: &[(usize, &'a Span)],
        children: &BTreeMap<String, Vec<usize>>,
        adopters: &BTreeMap<String, usize>,
        seen: &mut std::collections::BTreeSet<usize>,
    ) -> Frame<'a> {
        let span = owned[pos].1;
        let kids: Vec<usize> = if adopters.get(span.span_id.as_str()) == Some(&pos) {
            children
                .get(&span.span_id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|&kid_pos| seen.insert(owned[kid_pos].0))
                .collect()
        } else {
            Vec::new()
        };
        Frame {
            span,
            kids,
            next: 0,
            built: Vec::new(),
        }
    }
    fn build(
        root_pos: usize,
        owned: &[(usize, &Span)],
        children: &BTreeMap<String, Vec<usize>>,
        adopters: &BTreeMap<String, usize>,
        seen: &mut std::collections::BTreeSet<usize>,
    ) -> FlameNode {
        let mut stack: Vec<Frame> = vec![frame_for(root_pos, owned, children, adopters, seen)];
        loop {
            let top = stack.last_mut().expect("a frame under construction");
            if top.next < top.kids.len() {
                let kid_pos = top.kids[top.next];
                top.next += 1;
                stack.push(frame_for(kid_pos, owned, children, adopters, seen));
                continue;
            }
            let frame = stack.pop().expect("a frame under construction");
            let mut child_nodes = frame.built;
            sort_flame_children(&mut child_nodes);
            let kid_spans: Vec<&Span> = frame.kids.iter().map(|&p| owned[p].1).collect();
            let self_nanos = self_time(frame.span, &kid_spans).max(0) as u64;
            let value = child_nodes
                .iter()
                .fold(self_nanos, |acc, c| acc.saturating_add(c.value));
            let node = FlameNode {
                name: format!("{} · {}", frame.span.service, frame.span.name),
                value,
                children: child_nodes,
            };
            match stack.last_mut() {
                Some(parent) => parent.built.push(node),
                None => return node,
            }
        }
    }
    let owned = trace_occurrences(spans, trace_id);
    let children = occurrence_children(&owned);
    let adopters = adopting_copies(&owned, &children);
    let ids: std::collections::BTreeSet<&str> =
        owned.iter().map(|(_, s)| s.span_id.as_str()).collect();
    let mut seen: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
    let mut root = FlameNode {
        name: "trace".to_string(),
        value: 0,
        children: Vec::new(),
    };
    for (pos, (index, span)) in owned.iter().enumerate() {
        let is_root = span
            .parent_span_id
            .as_deref()
            .is_none_or(|p| !ids.contains(p));
        if is_root && seen.insert(*index) {
            let node = build(pos, &owned, &children, &adopters, &mut seen);
            root.value = root.value.saturating_add(node.value);
            root.children.push(node);
        }
    }
    sort_flame_children(&mut root.children);
    root
}

/// One directed edge of a service map: calls observed from one service
/// into another, with the child spans as the evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapEdge {
    /// Caller end of the edge, the service of the parent
    /// spans that evidence it.
    pub from: String,
    /// Callee end of the edge, the service of the child
    /// spans counted on it.
    pub to: String,
    /// Evidence spans counted on this edge, one per observed
    /// call, with a client span and the server span it
    /// caused counted once at the server span.
    pub calls: u64,
    /// How many of the evidence spans carry an error status.
    pub errors: u64,
    /// Median duration of the evidence spans in nanoseconds
    /// by nearest rank.
    pub p50_nanos: i64,
    /// 99th-percentile duration of the evidence spans in
    /// nanoseconds by nearest rank.
    pub p99_nanos: i64,
}

/// The derived service map over a span set: every service as a node
/// with its RED rollup, and the evidenced edges between them.
#[derive(Debug, Clone, PartialEq)]
pub struct ServiceGraph {
    /// One RED rollup per service in the span set, on the
    /// same basis as `red_by_service` and sorted by service
    /// name.
    pub nodes: Vec<RedStats>,
    /// Edges sorted by caller then callee, present only
    /// where a span's parent sits in another service.
    pub edges: Vec<MapEdge>,
}

/// Derive the service map over a span set.
///
/// An edge exists only on span-tree evidence: a span whose parent, in
/// the same trace, belongs to another service. The child span is the
/// evidence and is counted once, with its duration and error state.
/// A client span and the server span it caused are one call, counted
/// at the server span; the client span's own parent edge, when it has
/// one, is separate evidence about the client's caller. Edges are
/// sorted by caller then callee; nodes carry the same RED rollup, on
/// the same basis, as the red_row panel.
pub fn service_graph(spans: &[Span], window_seconds: f64) -> ServiceGraph {
    // A repeated span id resolves to its first occurrence in
    // canonical order, the same parent the waterfall gives a
    // child (INV-89), so an edge never takes its caller from
    // a later copy by arrival accident.
    let mut by_key: BTreeMap<(&str, &str), &Span> = BTreeMap::new();
    let trace_ids: std::collections::BTreeSet<&str> =
        spans.iter().map(|s| s.trace_id.as_str()).collect();
    for trace_id in trace_ids {
        for span in trace_spans(spans, trace_id) {
            by_key
                .entry((trace_id, span.span_id.as_str()))
                .or_insert(span);
        }
    }
    let mut edge_spans: BTreeMap<(String, String), Vec<&Span>> = BTreeMap::new();
    for span in spans {
        let Some(parent_id) = span.parent_span_id.as_deref() else {
            continue;
        };
        let Some(parent) = by_key.get(&(span.trace_id.as_str(), parent_id)) else {
            continue;
        };
        if parent.service != span.service {
            edge_spans
                .entry((parent.service.clone(), span.service.clone()))
                .or_default()
                .push(span);
        }
    }
    let edges = edge_spans
        .into_iter()
        .map(|((from, to), evidence)| {
            let mut durations: Vec<i64> = evidence.iter().map(|s| s.duration_nanos()).collect();
            durations.sort_unstable();
            MapEdge {
                from,
                to,
                calls: evidence.len() as u64,
                errors: evidence.iter().filter(|s| s.is_error).count() as u64,
                p50_nanos: percentile_nanos(&durations, 50).unwrap_or(0),
                p99_nanos: percentile_nanos(&durations, 99).unwrap_or(0),
            }
        })
        .collect();
    ServiceGraph {
        nodes: red_by_service(spans, window_seconds),
        edges,
    }
}

/// The layer of every service in a service graph, keyed by service
/// name. Layers are longest paths over the graph's condensation.
/// Services in one strongly connected component share a layer, and
/// a component's layer is one more than the greatest layer among
/// the components that call into it, or zero when none do. On an
/// acyclic graph every component is a single service and the
/// layers are the longest-path layers the renderer has always
/// used. On a cyclic graph the cycle collapses to one layer
/// instead of inflating the count (INV-82). An edge that names a
/// service absent from the graph's nodes takes no part in the
/// layering (INV-86).
pub fn service_layers(graph: &ServiceGraph) -> std::collections::BTreeMap<String, usize> {
    use std::collections::{BTreeMap, BTreeSet};
    let names: Vec<&str> = graph.nodes.iter().map(|n| n.service.as_str()).collect();
    let node_set: BTreeSet<&str> = names.iter().copied().collect();
    let mut out_adj: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    let mut in_adj: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for edge in &graph.edges {
        if !node_set.contains(edge.from.as_str()) || !node_set.contains(edge.to.as_str()) {
            continue;
        }
        out_adj
            .entry(edge.from.as_str())
            .or_default()
            .insert(edge.to.as_str());
        in_adj
            .entry(edge.to.as_str())
            .or_default()
            .insert(edge.from.as_str());
    }
    // Kosaraju pass one: finish order over the call graph.
    let mut visited: BTreeSet<&str> = BTreeSet::new();
    let mut finish: Vec<&str> = Vec::new();
    for start in &names {
        if visited.contains(start) {
            continue;
        }
        let mut stack: Vec<(&str, bool)> = vec![(start, false)];
        while let Some((node, exiting)) = stack.pop() {
            if exiting {
                finish.push(node);
                continue;
            }
            if !visited.insert(node) {
                continue;
            }
            stack.push((node, true));
            if let Some(nexts) = out_adj.get(node) {
                for next in nexts {
                    if !visited.contains(next) {
                        stack.push((next, false));
                    }
                }
            }
        }
    }
    // Pass two: components over the reversed graph, in reverse
    // finish order.
    let mut comp_of: BTreeMap<&str, usize> = BTreeMap::new();
    let mut comp_count = 0usize;
    for start in finish.iter().rev() {
        if comp_of.contains_key(start) {
            continue;
        }
        let mut stack: Vec<&str> = vec![start];
        while let Some(node) = stack.pop() {
            if comp_of.contains_key(node) {
                continue;
            }
            comp_of.insert(node, comp_count);
            if let Some(preds) = in_adj.get(node) {
                for pred in preds {
                    if !comp_of.contains_key(pred) {
                        stack.push(pred);
                    }
                }
            }
        }
        comp_count += 1;
    }
    // Longest path over the component graph, by relaxation. The
    // component graph is acyclic, so comp_count passes always
    // reach the fixpoint.
    let mut comp_edges: BTreeSet<(usize, usize)> = BTreeSet::new();
    for edge in &graph.edges {
        if !node_set.contains(edge.from.as_str()) || !node_set.contains(edge.to.as_str()) {
            continue;
        }
        let (from, to) = (comp_of[edge.from.as_str()], comp_of[edge.to.as_str()]);
        if from != to {
            comp_edges.insert((from, to));
        }
    }
    let mut comp_layer = vec![0usize; comp_count];
    for _ in 0..comp_count {
        let mut changed = false;
        for (from, to) in &comp_edges {
            if comp_layer[*to] < comp_layer[*from] + 1 {
                comp_layer[*to] = comp_layer[*from] + 1;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    names
        .iter()
        .map(|name| (name.to_string(), comp_layer[comp_of[name]]))
        .collect()
}

/// One node of a sankey flow: one service at one call-depth stage,
/// holding the number of spans staged there. The residual node is a
/// bucket, not a service, and carries the flag that says so.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SankeyNode {
    /// Call-depth stage of this node, with roots entering
    /// at 0 and residual entries landing at 1.
    pub stage: usize,
    /// Service staged at this node, or the panel's residual
    /// name on the residual node.
    pub service: String,
    /// Spans staged at this node: inflow plus the root and residual
    /// entries staged there, exactly.
    pub value: u64,
    /// Child spans this node's spans produced. A span parents many
    /// children, so outflow can exceed the value: fan-out is real
    /// flow, and the renderer draws the block at the greater of the
    /// two so no ribbon ever overflows it.
    pub outflow: u64,
    /// True on the single residual bucket node, which is a
    /// bucket rather than a service, and false on every
    /// real service node.
    pub residual: bool,
}

/// One aggregated step of sankey flow between two nodes: the number
/// of spans whose parent step lands between them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SankeyLink {
    /// Stage of the source node.
    pub from_stage: usize,
    /// Service of the source node, or the residual name
    /// when the source is the residual bucket.
    pub from_service: String,
    /// True when the source node is the residual bucket.
    pub from_residual: bool,
    /// Stage of the target node.
    pub to_stage: usize,
    /// Service of the target node.
    pub to_service: String,
    /// Spans whose parent step lands on this link, one
    /// unit of flow per span.
    pub value: u64,
}

/// The derived sankey flow over a span set: nodes in stage order,
/// links between them, and the count of spans that entered through
/// the residual bucket.
#[derive(Debug, Clone, PartialEq)]
pub struct SankeyFlow {
    /// One node per stage and service pair that received
    /// spans, plus the residual node when any span entered
    /// through it, ordered by stage, then value
    /// descending, then service.
    pub nodes: Vec<SankeyNode>,
    /// Aggregated parent steps between nodes, ordered by
    /// source node then target.
    pub links: Vec<SankeyLink>,
    /// Spans that entered through the residual bucket
    /// because their parent was absent or their ancestry
    /// cycled, equal to the residual node's outflow.
    pub residual_spans: u64,
    /// Number of stages in the flow, one past the deepest
    /// staged span, or 0 for an empty span set.
    pub stages: usize,
}

/// How one span's unit of flow entered its node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SankeyEntry {
    /// The span begins its trace: no parent at all.
    Root,
    /// The span's parent is in the set and stages it.
    Parent,
    /// The span's parent is named but absent, or its ancestry
    /// cycles: the residual bucket stages it.
    Residual,
}

/// Derive the sankey flow over a span set.
///
/// The unit of flow is one span, and every span is counted exactly
/// once, in exactly one node, staged by its depth in its own trace.
/// A span whose parent is in the set links from its parent's node.
/// A root span enters at stage zero. A span whose parent is named
/// but absent, or whose ancestry cycles back on itself, cannot be
/// staged on evidence: it enters through the residual node, a
/// single stage-zero source named by the panel, at stage one, and
/// its descendants stage normally beneath it. A span id that
/// repeats within a trace is malformed telemetry: one canonical
/// copy of the id, chosen by the span's own fields alone, drives
/// the staging walk, and every other copy stages by its own
/// parent chain against that staging, so no copy is dropped,
/// none is double-counted, and the result never depends on
/// input order. The accounting is
/// exact: for every real node, inflow plus the root and residual
/// entries staged there equals the node's span count, and the
/// residual node's outflow is exactly the count of spans that
/// entered through it. Outflow is a separate figure, the count of
/// child spans, and fan-out can push it past the span count.
/// Nodes are ordered by stage, then value descending, then
/// service name; links by their source node, then target.
pub fn sankey_flow(spans: &[Span], residual_name: &str) -> SankeyFlow {
    // Choose the canonical copy of every span id, per trace. The
    // choice compares the spans' own fields only, so it is the
    // same whatever order the spans arrived in.
    let canon_key = |s: &Span| {
        (
            s.parent_span_id.clone(),
            s.service.clone(),
            s.name.clone(),
            s.start_unix_nano,
            s.end_unix_nano,
        )
    };
    let mut traces: BTreeMap<&str, Vec<(usize, &Span)>> = BTreeMap::new();
    for (idx, span) in spans.iter().enumerate() {
        traces
            .entry(span.trace_id.as_str())
            .or_default()
            .push((idx, span));
    }
    let mut canonical_index: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    let mut canonical_by_key: BTreeMap<(&str, &str), &Span> = BTreeMap::new();
    let mut canonical_by_trace: BTreeMap<&str, BTreeMap<&str, &Span>> = BTreeMap::new();
    for (trace_id, members) in &traces {
        let mut best: BTreeMap<&str, (usize, &Span)> = BTreeMap::new();
        for &(idx, member) in members {
            match best.get(member.span_id.as_str()) {
                Some((_, current)) if canon_key(current) <= canon_key(member) => {}
                _ => {
                    best.insert(member.span_id.as_str(), (idx, member));
                }
            }
        }
        let mut by_id: BTreeMap<&str, &Span> = BTreeMap::new();
        for (span_id, (idx, member)) in best {
            canonical_index.insert((*trace_id, span_id), idx);
            canonical_by_key.insert((*trace_id, span_id), member);
            by_id.insert(span_id, member);
        }
        canonical_by_trace.insert(*trace_id, by_id);
    }

    // Stage the canonical spans, trace by trace, walking parent
    // links. Ids are unique in this pass by construction.
    let mut staged: BTreeMap<(&str, &str), (usize, SankeyEntry)> = BTreeMap::new();
    for trace_id in traces.keys() {
        // The trace's canonical spans, grouped once in the pass
        // above. Filtering the whole canonical map once per
        // trace made staging quadratic in the trace count,
        // convicted by the thousand-trace scale fixture.
        let trace_by_id = &canonical_by_trace[*trace_id];
        for span in trace_by_id.values() {
            if staged.contains_key(&(*trace_id, span.span_id.as_str())) {
                continue;
            }
            // Walk up from this span until the path anchors on a
            // root, a resolved span, a missing parent, or a cycle.
            let mut path: Vec<&Span> = vec![span];
            let mut cur: &Span = span;
            let mut assignments: Vec<(usize, SankeyEntry)> = Vec::new();
            loop {
                let Some(parent_id) = cur.parent_span_id.as_deref() else {
                    // The top of the path is a root.
                    let top = path.len() - 1;
                    for (i, _) in path.iter().enumerate() {
                        let entry = if i == top {
                            SankeyEntry::Root
                        } else {
                            SankeyEntry::Parent
                        };
                        assignments.push((top - i, entry));
                    }
                    break;
                };
                if let Some((stage, _)) = staged.get(&(*trace_id, parent_id)) {
                    // The parent is already staged: hang the path
                    // beneath it, nearest first.
                    let base = *stage;
                    for (i, _) in path.iter().enumerate() {
                        assignments.push((base + 1 + (path.len() - 1 - i), SankeyEntry::Parent));
                    }
                    break;
                }
                let Some(parent) = trace_by_id.get(parent_id) else {
                    // The top of the path names a parent the data
                    // does not contain: it enters through the
                    // residual bucket at stage one.
                    let top = path.len() - 1;
                    for (i, _) in path.iter().enumerate() {
                        let entry = if i == top {
                            SankeyEntry::Residual
                        } else {
                            SankeyEntry::Parent
                        };
                        assignments.push((1 + (top - i), entry));
                    }
                    break;
                };
                if let Some(cycle_at) = path.iter().position(|s| s.span_id == *parent_id) {
                    // The ancestry cycles. Every span in the cycle
                    // enters through the residual bucket at stage
                    // one; the spans below the cycle stage beneath
                    // the cycle member they hang from.
                    for (i, _) in path.iter().enumerate() {
                        if i >= cycle_at {
                            assignments.push((1, SankeyEntry::Residual));
                        } else {
                            assignments.push((1 + (cycle_at - i), SankeyEntry::Parent));
                        }
                    }
                    break;
                }
                path.push(parent);
                cur = parent;
            }
            for (member, (stage, entry)) in path.iter().zip(assignments.iter()) {
                staged.insert((*trace_id, member.span_id.as_str()), (*stage, *entry));
            }
        }
    }

    // Aggregate nodes and links over every span. A canonical span
    // takes the walked stage and entry. A repeated copy stages by
    // its own parent chain against the canonical staging: beneath
    // its parent id when that id staged, through the residual
    // when it did not, at stage zero when it names no parent.
    // Every span is counted exactly once and brings exactly the
    // inflow its entry names, so the accounting identities hold
    // for malformed sets too, and nothing here can panic.
    let mut node_values: BTreeMap<(usize, String, bool), u64> = BTreeMap::new();
    let mut link_values: BTreeMap<(usize, String, bool, usize, String), u64> = BTreeMap::new();
    let mut residual_spans: u64 = 0;
    let mut max_stage: usize = 0;
    for (idx, span) in spans.iter().enumerate() {
        let key = (span.trace_id.as_str(), span.span_id.as_str());
        let (stage, entry) = if canonical_index.get(&key) == Some(&idx) {
            staged[&key]
        } else {
            match span.parent_span_id.as_deref() {
                None => (0, SankeyEntry::Root),
                Some(parent_id) => match staged.get(&(key.0, parent_id)) {
                    Some((parent_stage, _)) => (*parent_stage + 1, SankeyEntry::Parent),
                    None => (1, SankeyEntry::Residual),
                },
            }
        };
        max_stage = max_stage.max(stage);
        *node_values
            .entry((stage, span.service.clone(), false))
            .or_default() += 1;
        match entry {
            SankeyEntry::Root => {}
            SankeyEntry::Parent => {
                let parent_id = span.parent_span_id.as_deref().unwrap_or_default();
                match canonical_by_key.get(&(key.0, parent_id)) {
                    Some(parent) => {
                        let (parent_stage, _) = staged[&(key.0, parent_id)];
                        *link_values
                            .entry((
                                parent_stage,
                                parent.service.clone(),
                                false,
                                stage,
                                span.service.clone(),
                            ))
                            .or_default() += 1;
                    }
                    None => {
                        // Unreachable by construction; degrade to
                        // the residual rather than drop the unit.
                        residual_spans += 1;
                        *link_values
                            .entry((
                                0,
                                residual_name.to_string(),
                                true,
                                stage,
                                span.service.clone(),
                            ))
                            .or_default() += 1;
                    }
                }
            }
            SankeyEntry::Residual => {
                residual_spans += 1;
                *link_values
                    .entry((
                        0,
                        residual_name.to_string(),
                        true,
                        stage,
                        span.service.clone(),
                    ))
                    .or_default() += 1;
            }
        }
    }
    if residual_spans > 0 {
        node_values.insert((0, residual_name.to_string(), true), residual_spans);
    }
    let mut outflow: BTreeMap<(usize, String, bool), u64> = BTreeMap::new();
    for ((from_stage, from_service, from_residual, _, _), value) in &link_values {
        *outflow
            .entry((*from_stage, from_service.clone(), *from_residual))
            .or_default() += value;
    }
    let mut nodes: Vec<SankeyNode> = node_values
        .into_iter()
        .map(|((stage, service, residual), value)| {
            let out = outflow
                .get(&(stage, service.clone(), residual))
                .copied()
                .unwrap_or(0);
            SankeyNode {
                stage,
                service,
                value,
                outflow: out,
                residual,
            }
        })
        .collect();
    nodes.sort_by(|a, b| {
        a.stage
            .cmp(&b.stage)
            .then_with(|| b.value.cmp(&a.value))
            .then_with(|| a.service.cmp(&b.service))
            .then_with(|| a.residual.cmp(&b.residual))
    });
    let links: Vec<SankeyLink> = link_values
        .into_iter()
        .map(
            |((from_stage, from_service, from_residual, to_stage, to_service), value)| SankeyLink {
                from_stage,
                from_service,
                from_residual,
                to_stage,
                to_service,
                value,
            },
        )
        .collect();
    SankeyFlow {
        stages: if spans.is_empty() { 0 } else { max_stage + 1 },
        nodes,
        links,
        residual_spans,
    }
}
