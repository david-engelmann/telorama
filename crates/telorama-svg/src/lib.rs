//! telorama-svg: the DisplayList and the deterministic SVG emitter.
//!
//! Rendering is a pure function of the scene and the data. Layout is
//! computed here, in Rust, from sorted inputs; floats are formatted by
//! one canonical function; element order is insertion order over sorted
//! data. The same scene and the same data produce the same bytes.
//!
//! M1 layout uses fixed columns rather than measured text. The text
//! crate (fontdb plus harfrust over an embedded subset font) replaces
//! the fixed label column with measured widths in a later milestone.

use std::collections::BTreeMap;
use std::fmt;
use telorama_scene::{
    Panel, Scene, Span, StatKind, Unit, critical_path, percentile_nanos, red_by_service,
    waterfall_rows,
};

/// Canonical float formatting for emitted geometry: two decimal places,
/// trailing zeros trimmed. One function, one rounding rule, everywhere.
pub fn fmt_num(v: f64) -> String {
    let raw = format!("{v:.2}");
    if raw.contains('.') {
        raw.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        raw
    }
}

/// Format a nanosecond duration with automatic magnitude scaling.
pub fn fmt_duration(nanos: i64) -> String {
    let ns = nanos.max(0) as f64;
    if ns < 1_000.0 {
        format!("{} ns", nanos.max(0))
    } else if ns < 1_000_000.0 {
        format!("{} µs", fmt_num(ns / 1_000.0))
    } else if ns < 1_000_000_000.0 {
        format!("{} ms", fmt_num(ns / 1_000_000.0))
    } else {
        format!("{} s", fmt_num(ns / 1_000_000_000.0))
    }
}

/// Format a byte count with automatic magnitude scaling.
pub fn fmt_bytes(bytes: i64) -> String {
    let b = bytes.max(0) as f64;
    if b < 1024.0 {
        format!("{} B", bytes.max(0))
    } else if b < 1024.0 * 1024.0 {
        format!("{} KiB", fmt_num(b / 1024.0))
    } else {
        format!("{} MiB", fmt_num(b / (1024.0 * 1024.0)))
    }
}

fn civil_from_days(z_in: i64) -> (i64, u32, u32) {
    let z = z_in + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Format Unix milliseconds as an ISO 8601 UTC timestamp with second
/// precision, for provenance captions.
pub fn fmt_time_ms(ms: i64) -> String {
    let secs = ms.div_euclid(1000);
    let days = secs.div_euclid(86_400);
    let tod = secs.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        tod / 3600,
        (tod % 3600) / 60,
        tod % 60
    )
}

fn escape_xml(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out
}

/// One drawing primitive in the DisplayList.
#[derive(Debug, Clone, PartialEq)]
pub enum Prim {
    Rect {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        fill: String,
        opacity: Option<f64>,
        stroke: Option<(String, f64)>,
    },
    Line {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        stroke: String,
        width: f64,
    },
    Text {
        x: f64,
        y: f64,
        size: f64,
        fill: String,
        bold: bool,
        mono: bool,
        anchor: &'static str,
        content: String,
    },
}

/// An ordered list of primitives plus the canvas identity. Emitting the
/// same DisplayList always produces the same SVG bytes.
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayList {
    pub width: f64,
    pub height: f64,
    pub title: String,
    pub description: String,
    pub prims: Vec<Prim>,
}

impl DisplayList {
    /// Emit the canonical SVG document for this display list.
    pub fn to_svg(&self) -> String {
        let mut out = String::new();
        out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        out.push_str(&format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" role=\"img\">\n",
            fmt_num(self.width),
            fmt_num(self.height),
            fmt_num(self.width),
            fmt_num(self.height)
        ));
        out.push_str(&format!("<title>{}</title>\n", escape_xml(&self.title)));
        out.push_str(&format!("<desc>{}</desc>\n", escape_xml(&self.description)));
        out.push_str("<rect x=\"0\" y=\"0\" width=\"");
        out.push_str(&fmt_num(self.width));
        out.push_str("\" height=\"");
        out.push_str(&fmt_num(self.height));
        out.push_str("\" fill=\"#FFFFFF\"/>\n");
        for prim in &self.prims {
            match prim {
                Prim::Rect {
                    x,
                    y,
                    w,
                    h,
                    fill,
                    opacity,
                    stroke,
                } => {
                    out.push_str(&format!(
                        "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\"",
                        fmt_num(*x),
                        fmt_num(*y),
                        fmt_num(*w),
                        fmt_num(*h),
                        escape_xml(fill)
                    ));
                    if let Some(o) = opacity {
                        out.push_str(&format!(" fill-opacity=\"{}\"", fmt_num(*o)));
                    }
                    if let Some((color, width)) = stroke {
                        out.push_str(&format!(
                            " stroke=\"{}\" stroke-width=\"{}\"",
                            escape_xml(color),
                            fmt_num(*width)
                        ));
                    }
                    out.push_str("/>\n");
                }
                Prim::Line {
                    x1,
                    y1,
                    x2,
                    y2,
                    stroke,
                    width,
                } => {
                    out.push_str(&format!(
                        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"{}\"/>\n",
                        fmt_num(*x1),
                        fmt_num(*y1),
                        fmt_num(*x2),
                        fmt_num(*y2),
                        escape_xml(stroke),
                        fmt_num(*width)
                    ));
                }
                Prim::Text {
                    x,
                    y,
                    size,
                    fill,
                    bold,
                    mono,
                    anchor,
                    content,
                } => {
                    let family = if *mono {
                        "ui-monospace, monospace"
                    } else {
                        "system-ui, sans-serif"
                    };
                    out.push_str(&format!(
                        "<text x=\"{}\" y=\"{}\" font-family=\"{}\" font-size=\"{}\" fill=\"{}\" text-anchor=\"{}\"",
                        fmt_num(*x),
                        fmt_num(*y),
                        family,
                        fmt_num(*size),
                        escape_xml(fill),
                        anchor
                    ));
                    if *bold {
                        out.push_str(" font-weight=\"700\"");
                    }
                    out.push_str(&format!(">{}</text>\n", escape_xml(content)));
                }
            }
        }
        out.push_str("</svg>\n");
        out
    }
}

/// A rendering failure. Rendering never guesses: a panel whose data
/// cannot support it fails with a specific reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderError {
    pub message: String,
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RenderError {}

fn fail(message: impl Into<String>) -> RenderError {
    RenderError {
        message: message.into(),
    }
}

/// Okabe-Ito categorical palette, ordered for bar use. Assignment is by
/// sorted service name, so a service keeps its color across panels and
/// across re-renders of the same data. Contrast filtering against the
/// theme background arrives with the field system in M2.
pub const SERVICE_PALETTE: [&str; 8] = [
    "#0072B2", "#E69F00", "#009E73", "#D55E00", "#56B4E9", "#CC79A7", "#F0E442", "#000000",
];

/// The vermillion reserved for errors when error highlighting is on.
pub const ERROR_COLOR: &str = "#D55E00";

const INK: &str = "#1A1A1A";
const MUTED: &str = "#5A5A5A";
const WIDTH: f64 = 960.0;
const MARGIN: f64 = 24.0;

fn service_colors(spans: &[Span]) -> BTreeMap<String, String> {
    let mut services: Vec<&str> = spans.iter().map(|s| s.service.as_str()).collect();
    services.sort_unstable();
    services.dedup();
    services
        .iter()
        .enumerate()
        .map(|(i, name)| {
            (
                name.to_string(),
                SERVICE_PALETTE[i % SERVICE_PALETTE.len()].to_string(),
            )
        })
        .collect()
}

// A text call is position, size, color, weight, family, and content: the
// argument list is the interface, so the count lint is allowed here.
#[allow(clippy::too_many_arguments)]
fn text(
    prims: &mut Vec<Prim>,
    x: f64,
    y: f64,
    size: f64,
    fill: &str,
    bold: bool,
    mono: bool,
    content: impl Into<String>,
) {
    prims.push(Prim::Text {
        x,
        y,
        size,
        fill: fill.to_string(),
        bold,
        mono,
        anchor: "start",
        content: content.into(),
    });
}

fn truncate_label(label: &str, max_chars: usize) -> String {
    if label.chars().count() <= max_chars {
        return label.to_string();
    }
    let mut out: String = label.chars().take(max_chars - 1).collect();
    out.push('…');
    out
}

/// Spans a data entry points at: the spans of its trace when it names
/// one, otherwise every span in the set.
fn dataset<'a>(scene: &Scene, spans: &'a [Span], data_id: &str) -> Vec<&'a Span> {
    let entry = scene.data.iter().find(|d| d.id == data_id);
    match entry.and_then(|e| e.source.trace_id.as_deref()) {
        Some(trace_id) => spans.iter().filter(|s| s.trace_id == trace_id).collect(),
        None => spans.iter().collect(),
    }
}

// The waterfall renderer takes the scene, its data, its two highlight
// switches, the color map, and the shared layout cursor: one call, one
// panel, so the count lint is allowed here.
#[allow(clippy::too_many_arguments)]
fn render_waterfall(
    scene: &Scene,
    spans: &[Span],
    data_id: &str,
    highlight_path: bool,
    highlight_errors: bool,
    colors: &BTreeMap<String, String>,
    prims: &mut Vec<Prim>,
    y: &mut f64,
) -> Result<(), RenderError> {
    let entry = scene
        .data
        .iter()
        .find(|d| d.id == data_id)
        .ok_or_else(|| fail(format!("waterfall reads undeclared data id {data_id:?}")))?;
    let trace_id = entry
        .source
        .trace_id
        .as_deref()
        .ok_or_else(|| fail("waterfall panel needs a data entry with a trace_id"))?;
    let rows = waterfall_rows(spans, trace_id);
    if rows.is_empty() {
        return Err(fail(format!("no spans found for trace {trace_id}")));
    }
    let by_id: BTreeMap<&str, &Span> = spans
        .iter()
        .filter(|s| s.trace_id == trace_id)
        .map(|s| (s.span_id.as_str(), s))
        .collect();
    let t0 = by_id.values().map(|s| s.start_unix_nano).min().unwrap_or(0);
    let t1 = by_id.values().map(|s| s.end_unix_nano).max().unwrap_or(0);
    let total = (t1 - t0).max(1) as f64;
    let critical: std::collections::BTreeSet<String> =
        critical_path(spans, trace_id).into_iter().collect();

    let plot_x0 = 320.0;
    let plot_x1 = WIDTH - MARGIN;
    let plot_w = plot_x1 - plot_x0;

    // Time axis.
    let axis_y = *y + 4.0;
    prims.push(Prim::Line {
        x1: plot_x0,
        y1: axis_y,
        x2: plot_x1,
        y2: axis_y,
        stroke: "#CCCCCC".to_string(),
        width: 1.0,
    });
    for step in 0..=4 {
        let frac = f64::from(step) / 4.0;
        let tx = plot_x0 + frac * plot_w;
        prims.push(Prim::Line {
            x1: tx,
            y1: axis_y,
            x2: tx,
            y2: axis_y + 4.0,
            stroke: "#CCCCCC".to_string(),
            width: 1.0,
        });
        prims.push(Prim::Text {
            x: tx,
            y: axis_y - 4.0,
            size: 10.0,
            fill: MUTED.to_string(),
            bold: false,
            mono: false,
            anchor: "middle",
            content: fmt_duration((frac * total) as i64),
        });
    }
    *y += 22.0;

    let row_h = 20.0;
    let bar_h = 11.0;
    for row in &rows {
        let span = by_id[row.span_id.as_str()];
        let label = truncate_label(&format!("{} · {}", span.service, span.name), 36);
        text(
            prims,
            MARGIN + row.depth as f64 * 14.0,
            *y + 12.0,
            11.0,
            INK,
            false,
            false,
            label,
        );
        let bar_x = plot_x0 + (span.start_unix_nano - t0) as f64 / total * plot_w;
        let bar_w = ((span.duration_nanos() as f64) / total * plot_w).max(1.5);
        let fill = if highlight_errors && span.is_error {
            ERROR_COLOR.to_string()
        } else {
            colors
                .get(&span.service)
                .cloned()
                .unwrap_or_else(|| "#888888".to_string())
        };
        let on_path = highlight_path && critical.contains(&span.span_id);
        prims.push(Prim::Rect {
            x: bar_x,
            y: *y + 2.0,
            w: bar_w,
            h: bar_h,
            fill,
            opacity: None,
            stroke: if on_path {
                Some(("#111111".to_string(), 1.5))
            } else {
                None
            },
        });
        if on_path {
            prims.push(Prim::Rect {
                x: bar_x,
                y: *y + 2.0 + bar_h - 3.0,
                w: bar_w,
                h: 3.0,
                fill: "#111111".to_string(),
                opacity: Some(0.45),
                stroke: None,
            });
        }
        let mut note = fmt_duration(span.duration_nanos());
        if span.is_error {
            note.push_str(" · error");
        }
        text(
            prims,
            bar_x + bar_w + 6.0,
            *y + 12.0,
            11.0,
            MUTED,
            false,
            false,
            note,
        );
        *y += row_h;
    }
    text(
        prims,
        MARGIN,
        *y + 14.0,
        11.0,
        MUTED,
        false,
        false,
        format!(
            "{} spans, {} on the critical path. Self time is computed per span in the scene data.",
            rows.len(),
            critical.len()
        ),
    );
    *y += 22.0;
    Ok(())
}

fn render_stat(
    scene: &Scene,
    spans: &[Span],
    panel: &Panel,
    prims: &mut Vec<Prim>,
    y: &mut f64,
) -> Result<(), RenderError> {
    let Panel::Stat {
        data,
        stat,
        unit,
        service,
    } = panel
    else {
        return Err(fail("render_stat called on a non-stat panel"));
    };
    let mut set = dataset(scene, spans, data);
    if let Some(want) = service {
        set.retain(|s| &s.service == want);
    }
    let durations: Vec<i64> = set.iter().map(|s| s.duration_nanos()).collect();
    let calls = set.len() as u64;
    let errors = set.iter().filter(|s| s.is_error).count() as u64;

    let (value, derivation) = match stat {
        StatKind::P50 | StatKind::P95 | StatKind::P99 => {
            let p = stat.percentile().unwrap_or(95);
            match percentile_nanos(&durations, p) {
                Some(v) => (
                    format_by_unit(v, *unit),
                    format!("{} of {} span durations in the window", stat.name(), calls),
                ),
                None => ("-".to_string(), "no spans in the window".to_string()),
            }
        }
        StatKind::Count => (format!("{calls}"), format!("{calls} spans in the window")),
        StatKind::Rate => {
            let secs = scene.context.range.seconds();
            let rate = if secs > 0.0 { calls as f64 / secs } else { 0.0 };
            (
                fmt_num(rate),
                format!("{calls} spans over a {} s window", fmt_num(secs)),
            )
        }
        StatKind::ErrorRatio => {
            let ratio = if calls == 0 {
                0.0
            } else {
                errors as f64 / calls as f64
            };
            (
                format!("{:.1}%", ratio * 100.0),
                format!("{errors} errors of {calls} calls"),
            )
        }
        StatKind::Last => {
            let last = set.iter().max_by(|a, b| {
                a.end_unix_nano
                    .cmp(&b.end_unix_nano)
                    .then_with(|| b.span_id.cmp(&a.span_id))
            });
            match last {
                Some(span) => (
                    format_by_unit(span.duration_nanos(), *unit),
                    "duration of the last span to finish".to_string(),
                ),
                None => ("-".to_string(), "no spans in the window".to_string()),
            }
        }
    };

    let mut label = format!("{} · {}", stat.name(), unit.name());
    if let Some(s) = service {
        label.push_str(&format!(" · {s}"));
    }
    text(prims, MARGIN, *y + 12.0, 12.0, MUTED, false, false, label);
    text(prims, MARGIN, *y + 44.0, 30.0, INK, true, false, value);
    text(
        prims,
        MARGIN,
        *y + 62.0,
        11.0,
        MUTED,
        false,
        false,
        derivation,
    );
    *y += 70.0;
    Ok(())
}

fn format_by_unit(value_nanos: i64, unit: Unit) -> String {
    match unit {
        Unit::Duration => fmt_duration(value_nanos),
        Unit::Bytes => fmt_bytes(value_nanos),
        Unit::Count => format!("{value_nanos}"),
        Unit::Ratio => format!("{:.1}%", value_nanos as f64 / 100.0),
        Unit::RequestsPerSecond => fmt_num(value_nanos as f64),
    }
}

fn render_red_row(
    scene: &Scene,
    spans: &[Span],
    panel: &Panel,
    prims: &mut Vec<Prim>,
    y: &mut f64,
) -> Result<(), RenderError> {
    let Panel::RedRow { data, service } = panel else {
        return Err(fail("render_red_row called on a non-red panel"));
    };
    let set_owned: Vec<Span> = dataset(scene, spans, data).into_iter().cloned().collect();
    let mut stats = red_by_service(&set_owned, scene.context.range.seconds());
    if let Some(want) = service {
        stats.retain(|s| &s.service == want);
    }
    if stats.is_empty() {
        return Err(fail("red_row panel has no services to report"));
    }
    let columns: [(&str, f64); 7] = [
        ("service", MARGIN),
        ("calls", 300.0),
        ("rate/s", 380.0),
        ("errors", 470.0),
        ("p50", 570.0),
        ("p95", 680.0),
        ("p99", 790.0),
    ];
    for (name, x) in columns {
        text(prims, x, *y + 12.0, 11.0, MUTED, true, false, name);
    }
    *y += 18.0;
    for row in &stats {
        text(
            prims,
            MARGIN,
            *y + 12.0,
            12.0,
            INK,
            false,
            false,
            row.service.clone(),
        );
        text(
            prims,
            300.0,
            *y + 12.0,
            12.0,
            INK,
            false,
            true,
            row.calls.to_string(),
        );
        text(
            prims,
            380.0,
            *y + 12.0,
            12.0,
            INK,
            false,
            true,
            fmt_num(row.rate_per_sec),
        );
        let err_fill = if row.error_ratio >= 0.05 {
            ERROR_COLOR
        } else {
            INK
        };
        text(
            prims,
            470.0,
            *y + 12.0,
            12.0,
            err_fill,
            false,
            true,
            format!("{:.1}%", row.error_ratio * 100.0),
        );
        text(
            prims,
            570.0,
            *y + 12.0,
            12.0,
            INK,
            false,
            true,
            fmt_duration(row.p50_nanos),
        );
        text(
            prims,
            680.0,
            *y + 12.0,
            12.0,
            INK,
            false,
            true,
            fmt_duration(row.p95_nanos),
        );
        text(
            prims,
            790.0,
            *y + 12.0,
            12.0,
            INK,
            false,
            true,
            fmt_duration(row.p99_nanos),
        );
        *y += 18.0;
    }
    let bases: std::collections::BTreeSet<&str> = stats.iter().map(|s| s.basis).collect();
    let basis_note = if bases.len() == 1 {
        format!(
            "Counted on {} spans over a {} s window. Duration is percentiles of span duration.",
            bases.iter().next().unwrap_or(&"all"),
            fmt_num(scene.context.range.seconds())
        )
    } else {
        "Counted on server spans where a service has them, all spans otherwise. Duration is percentiles of span duration.".to_string()
    };
    text(
        prims,
        MARGIN,
        *y + 14.0,
        11.0,
        MUTED,
        false,
        false,
        basis_note,
    );
    *y += 22.0;
    Ok(())
}

/// Render a scene over a span set to a deterministic SVG document.
///
/// Panels render in spec order under the headline. The provenance
/// caption at the foot of the scene prints the window, every data
/// source with its query or trace id and sampling state, and the
/// generating question, so the picture carries its own evidence.
pub fn render_scene(scene: &Scene, spans: &[Span]) -> Result<String, RenderError> {
    let colors = service_colors(spans);
    let mut prims: Vec<Prim> = Vec::new();
    let mut y: f64 = 30.0;

    text(
        &mut prims,
        MARGIN,
        y,
        20.0,
        INK,
        true,
        false,
        scene.headline.clone(),
    );
    y += 20.0;
    text(
        &mut prims,
        MARGIN,
        y,
        12.0,
        MUTED,
        false,
        false,
        scene.title.clone(),
    );
    y += 12.0;

    for panel in &scene.panels {
        y += 18.0;
        let heading = match panel {
            Panel::Waterfall { .. } => "Waterfall",
            Panel::Stat { .. } => "Stat",
            Panel::RedRow { .. } => "Rate, errors, duration by service",
            other => {
                return Err(fail(format!(
                    "panel type {} is in the spec but has no renderer yet: it lands in a later milestone",
                    other.type_name()
                )));
            }
        };
        text(&mut prims, MARGIN, y, 14.0, INK, true, false, heading);
        y += 8.0;
        match panel {
            Panel::Waterfall {
                data,
                highlight_critical_path,
                highlight_errors,
            } => render_waterfall(
                scene,
                spans,
                data,
                *highlight_critical_path,
                *highlight_errors,
                &colors,
                &mut prims,
                &mut y,
            )?,
            Panel::Stat { .. } => render_stat(scene, spans, panel, &mut prims, &mut y)?,
            Panel::RedRow { .. } => render_red_row(scene, spans, panel, &mut prims, &mut y)?,
            _ => unreachable!("heading match already returned for unsupported panels"),
        }
    }

    // Provenance caption.
    y += 10.0;
    prims.push(Prim::Line {
        x1: MARGIN,
        y1: y,
        x2: WIDTH - MARGIN,
        y2: y,
        stroke: "#CCCCCC".to_string(),
        width: 1.0,
    });
    y += 16.0;
    text(
        &mut prims,
        MARGIN,
        y,
        11.0,
        MUTED,
        false,
        false,
        format!(
            "Window {} to {}",
            fmt_time_ms(scene.context.range.from_unix_ms),
            fmt_time_ms(scene.context.range.to_unix_ms)
        ),
    );
    y += 15.0;
    for entry in &scene.data {
        let source_desc = match (&entry.source.query, &entry.source.trace_id) {
            (Some(q), _) => format!("query {q}"),
            (None, Some(t)) => format!("trace {t}"),
            (None, None) => "no source detail".to_string(),
        };
        let sampled = match entry.sampled {
            telorama_scene::Sampling::Unsampled => "unsampled",
            telorama_scene::Sampling::Sampled => "sampled",
            telorama_scene::Sampling::Unknown => "sampling unknown",
        };
        text(
            &mut prims,
            MARGIN,
            y,
            11.0,
            MUTED,
            false,
            false,
            format!(
                "Data {} · {} · {} · {}",
                entry.id, entry.source.kind, source_desc, sampled
            ),
        );
        y += 15.0;
    }
    text(
        &mut prims,
        MARGIN,
        y,
        11.0,
        MUTED,
        false,
        false,
        format!(
            "Generated by {} · Question: {}",
            scene.context.provenance.generated_by, scene.context.provenance.question
        ),
    );
    y += 14.0;

    let list = DisplayList {
        width: WIDTH,
        height: y,
        title: scene.title.clone(),
        description: scene.headline.clone(),
        prims,
    };
    Ok(list.to_svg())
}
