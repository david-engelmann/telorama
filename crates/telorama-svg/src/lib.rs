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
    AgentRun, AgentStateEntry, AgentStep, Finding, FlameNode, Panel, RedStats, SankeyLink, Scene,
    ServiceGraph, Span, StackSample, StatKind, Thresholds, Unit, critical_path, flame_from_spans,
    flame_tree, heatmap_grid, percentile_nanos, red_by_service, sankey_flow, service_graph,
    waterfall_rows,
};

/// Canonical float formatting for emitted geometry. The implementation
/// lives in telorama-scene's field system, so every surface formats a
/// value the same way; these are the renderer's handles on it.
pub fn fmt_num(v: f64) -> String {
    telorama_scene::format_number(v)
}

/// Format a nanosecond duration with automatic magnitude scaling.
pub fn fmt_duration(nanos: i64) -> String {
    telorama_scene::format_duration_nanos(nanos)
}

/// Format a byte count with automatic magnitude scaling.
pub fn fmt_bytes(bytes: i64) -> String {
    telorama_scene::format_bytes(bytes)
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
    Polygon {
        points: Vec<(f64, f64)>,
        fill: String,
        opacity: Option<f64>,
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
                Prim::Polygon {
                    points,
                    fill,
                    opacity,
                } => {
                    let joined = points
                        .iter()
                        .map(|(x, y)| format!("{},{}", fmt_num(*x), fmt_num(*y)))
                        .collect::<Vec<_>>()
                        .join(" ");
                    out.push_str(&format!(
                        "<polygon points=\"{}\" fill=\"{}\"",
                        joined,
                        escape_xml(fill)
                    ));
                    if let Some(o) = opacity {
                        out.push_str(&format!(" fill-opacity=\"{}\"", fmt_num(*o)));
                    }
                    out.push_str("/>\n");
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
/// across re-renders of the same data. Verdict colors (a value crossing
/// its threshold, an error span) come from the field system's
/// thresholds, not from this palette.
pub const SERVICE_PALETTE: [&str; 8] = [
    "#0072B2", "#E69F00", "#009E73", "#D55E00", "#56B4E9", "#CC79A7", "#F0E442", "#000000",
];

/// The vermillion reserved for errors when error highlighting is on.
pub const ERROR_COLOR: &str = "#D55E00";

const INK: &str = "#1A1A1A";
const MUTED: &str = "#5A5A5A";
const WIDTH: f64 = 960.0;
const MARGIN: f64 = 24.0;

pub fn service_colors(spans: &[Span]) -> BTreeMap<String, String> {
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
    if max_chars == 0 {
        return String::new();
    }
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

/// A USD cost at the precision agent telemetry reports: four
/// decimals, trailing zeros trimmed, never fewer than two decimals.
fn fmt_cost(value: f64) -> String {
    let mut text = format!("{value:.4}");
    while text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.push_str("00");
    }
    format!("${text}")
}

#[allow(clippy::too_many_arguments)]
fn layout_agent_step(
    prims: &mut Vec<Prim>,
    step: &AgentStep,
    depth: usize,
    from_nano: i64,
    window_nano: i64,
    y: &mut f64,
) {
    let plot_x0 = 200.0;
    let plot_x1 = WIDTH - MARGIN;
    let x_of =
        |t: i64| plot_x0 + ((t - from_nano) as f64 / window_nano as f64) * (plot_x1 - plot_x0);
    let start = step.start_unix_nano();
    let end = step.end_unix_nano();
    let x1 = x_of(start);
    let x2 = x_of(end).max(x1 + 1.5);
    let label_x = MARGIN + depth as f64 * 14.0;
    let (label, fill) = match step {
        AgentStep::ModelCall(call) => (format!("model {}", call.model), "#0072B2"),
        AgentStep::ToolCall(tool) => (format!("tool {}", tool.name), "#009E73"),
        AgentStep::Subagent(sub) => (format!("subagent {}", sub.agent_id), "#E69F00"),
    };
    text(
        prims,
        label_x,
        *y + 11.0,
        11.0,
        INK,
        false,
        false,
        truncate_label(&label, ((plot_x0 - label_x - 8.0) / 6.0) as usize),
    );
    if let AgentStep::ToolCall(tool) = step
        && tool.wait_nanos > 0
    {
        // The permission wait in grey at the head of the bar, the
        // execution in the tool color behind it.
        let wait_end = x_of(start + tool.wait_nanos).min(x2);
        prims.push(Prim::Rect {
            x: x1,
            y: *y,
            w: (wait_end - x1).max(0.0),
            h: 12.0,
            fill: "#BBBBBB".to_string(),
            opacity: None,
            stroke: None,
        });
        prims.push(Prim::Rect {
            x: wait_end,
            y: *y,
            w: (x2 - wait_end).max(0.0),
            h: 12.0,
            fill: fill.to_string(),
            opacity: None,
            stroke: None,
        });
    } else {
        prims.push(Prim::Rect {
            x: x1,
            y: *y,
            w: x2 - x1,
            h: 12.0,
            fill: fill.to_string(),
            opacity: None,
            stroke: None,
        });
    }
    let mut detail = fmt_duration(end - start);
    let mut is_error = false;
    match step {
        AgentStep::ModelCall(call) => {
            if let Some(ttft) = call.ttft_nanos {
                detail.push_str(&format!(" · ttft {}", fmt_duration(ttft)));
            }
            if call.attempts > 1 {
                detail.push_str(&format!(" · {} attempts", call.attempts));
            }
            is_error = call.is_error;
        }
        AgentStep::ToolCall(tool) => {
            if tool.wait_nanos > 0 {
                detail.push_str(&format!(
                    " · waited {} on permission",
                    fmt_duration(tool.wait_nanos)
                ));
            }
            if let Some(decision) = &tool.decision {
                detail.push_str(&format!(" · decision {decision}"));
            }
            is_error = tool.is_error;
        }
        AgentStep::Subagent(_) => {}
    }
    text(
        prims,
        x2 + 6.0,
        *y + 11.0,
        11.0,
        MUTED,
        false,
        false,
        detail.clone(),
    );
    if is_error {
        text(
            prims,
            x2 + 12.0 + detail.chars().count() as f64 * 6.05,
            *y + 11.0,
            11.0,
            ERROR_COLOR,
            true,
            false,
            "· error".to_string(),
        );
    }
    *y += 18.0;
    if let AgentStep::Subagent(sub) = step {
        for child in &sub.steps {
            layout_agent_step(prims, child, depth + 1, from_nano, window_nano, y);
        }
    }
}

fn render_agent_state(
    scene: &Scene,
    data: &SceneData,
    panel: &Panel,
    prims: &mut Vec<Prim>,
    y: &mut f64,
) -> Result<(), RenderError> {
    let Panel::AgentState { data: data_id } = panel else {
        return Err(fail("render_agent_state called on a non-agent-state panel"));
    };
    match scene.data.iter().find(|d| d.id == *data_id) {
        Some(entry) if entry.source.kind == "agent_state" => {}
        Some(_) => {
            return Err(fail(format!(
                "agent_state panel reads data id {data_id:?}, whose source kind is not agent_state"
            )));
        }
        None => {
            return Err(fail(format!(
                "agent_state reads undeclared data id {data_id:?}"
            )));
        }
    }
    if data.agent_state.is_empty() {
        return Err(fail(
            "agent_state panel needs agent state: supply a state report for its data entry",
        ));
    }
    for entry in data.agent_state {
        text(
            prims,
            MARGIN,
            *y + 12.0,
            12.0,
            INK,
            true,
            false,
            format!("{} · {}", entry.agent, entry.doing),
        );
        *y += 15.0;
        text(
            prims,
            MARGIN,
            *y + 12.0,
            12.0,
            INK,
            false,
            false,
            format!("Why · {}", entry.why),
        );
        *y += 15.0;
        if let Some(waiting) = &entry.waiting_on {
            text(
                prims,
                MARGIN,
                *y + 12.0,
                12.0,
                ERROR_COLOR,
                false,
                false,
                format!("Waiting on · {waiting}"),
            );
            *y += 15.0;
        }
        if let Some(next) = &entry.next {
            text(
                prims,
                MARGIN,
                *y + 12.0,
                12.0,
                MUTED,
                false,
                false,
                format!("Next · {next}"),
            );
            *y += 15.0;
        }
        *y += 6.0;
    }
    *y -= 6.0;
    Ok(())
}

fn render_agent_run(
    scene: &Scene,
    data: &SceneData,
    panel: &Panel,
    prims: &mut Vec<Prim>,
    y: &mut f64,
) -> Result<(), RenderError> {
    let Panel::AgentRun { data: data_id } = panel else {
        return Err(fail("render_agent_run called on a non-agent-run panel"));
    };
    if !scene.data.iter().any(|d| d.id == *data_id) {
        return Err(fail(format!(
            "agent_run reads undeclared data id {data_id:?}"
        )));
    }
    let run = data.agent_run.ok_or_else(|| {
        fail("agent_run panel needs an agent run: supply a session export for its data entry")
    })?;
    if run.turns.is_empty() {
        return Err(fail("the agent run has no turns"));
    }
    let from_nano = scene.context.range.from_unix_ms * 1_000_000;
    let window_nano =
        ((scene.context.range.to_unix_ms - scene.context.range.from_unix_ms) * 1_000_000).max(1);

    // Time axis, the same five ticks the waterfall uses.
    let plot_x0 = 200.0;
    let plot_x1 = WIDTH - MARGIN;
    let axis_y = *y + 4.0;
    prims.push(Prim::Line {
        x1: plot_x0,
        y1: axis_y,
        x2: plot_x1,
        y2: axis_y,
        stroke: "#CCCCCC".to_string(),
        width: 1.0,
    });
    for step_i in 0..=4 {
        let frac = f64::from(step_i) / 4.0;
        let tx = plot_x0 + frac * (plot_x1 - plot_x0);
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
            content: fmt_duration((frac * window_nano as f64) as i64),
        });
    }
    *y += 22.0;

    let count = |v: u64| telorama_scene::format_value(Unit::Count, v as f64);
    let mut total_in = 0u64;
    let mut total_out = 0u64;
    let mut total_cache = 0u64;
    let mut total_cost = 0.0f64;
    for (i, turn) in run.turns.iter().enumerate() {
        total_in += turn.input_tokens;
        total_out += turn.output_tokens;
        total_cache += turn.cache_read_tokens;
        total_cost += turn.cost_usd;
        text(
            prims,
            MARGIN,
            *y + 12.0,
            12.0,
            INK,
            true,
            false,
            format!(
                "Turn {} · {} in · {} out · {} cache read · {}",
                i + 1,
                count(turn.input_tokens),
                count(turn.output_tokens),
                count(turn.cache_read_tokens),
                fmt_cost(turn.cost_usd)
            ),
        );
        *y += 20.0;
        for step in &turn.steps {
            layout_agent_step(prims, step, 0, from_nano, window_nano, y);
        }
        *y += 6.0;
    }
    text(
        prims,
        MARGIN,
        *y + 12.0,
        11.0,
        MUTED,
        false,
        false,
        format!(
            "Session {} · {} · {} in · {} out · {} cache read · {}",
            run.session_id,
            plural(run.turns.len() as u64, "turn", "turns"),
            count(total_in),
            count(total_out),
            count(total_cache),
            fmt_cost(total_cost)
        ),
    );
    *y += 15.0;
    let authority = match run.dialect.as_str() {
        "codex" => "tokens come from log events, spans counted but not merged in this dialect",
        "genai" | "openinference" => {
            "tokens and structure come from span attributes in this dialect"
        }
        _ => "waits and subagents come from spans, tokens and cost from logs",
    };
    text(
        prims,
        MARGIN,
        *y + 12.0,
        11.0,
        MUTED,
        false,
        false,
        format!(
            "Coverage · {} · {} · {} · {authority}",
            plural(run.coverage.log_events, "log event", "log events"),
            plural(run.coverage.spans_seen, "span read", "spans read"),
            plural(run.coverage.spans_merged, "span merged", "spans merged")
        ),
    );
    *y += 15.0;
    if let Some(pricing) = &run.pricing {
        let mut line = format!(
            "Cost is an estimate priced by telorama from tokens at the price table dated {}",
            pricing.table_as_of
        );
        if !pricing.unpriced_models.is_empty() {
            line.push_str(&format!(
                "; unpriced models excluded: {}",
                pricing.unpriced_models.join(", ")
            ));
        }
        text(prims, MARGIN, *y + 12.0, 11.0, MUTED, false, false, line);
        *y += 15.0;
    }
    *y += 7.0;
    Ok(())
}

fn plural(n: u64, singular: &str, plural_form: &str) -> String {
    if n == 1 {
        format!("{n} {singular}")
    } else {
        format!("{n} {plural_form}")
    }
}

/// Layered layout for the service map. A service's layer is the
/// longest path from a root service, found by bounded relaxation so
/// cyclic call graphs still terminate. Within a layer, services order
/// by the barycenter of their predecessors in the layer above, then
/// by name. Returns (service, layer, position in layer).
fn map_positions(graph: &ServiceGraph) -> Vec<(String, usize, usize)> {
    let mut layer: BTreeMap<&str, usize> = graph
        .nodes
        .iter()
        .map(|n| (n.service.as_str(), 0usize))
        .collect();
    for _ in 0..graph.nodes.len() {
        let mut changed = false;
        for edge in &graph.edges {
            let want = layer[edge.from.as_str()] + 1;
            if layer[edge.to.as_str()] < want {
                layer.insert(edge.to.as_str(), want);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let mut preds: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for edge in &graph.edges {
        preds
            .entry(edge.to.as_str())
            .or_default()
            .push(edge.from.as_str());
    }
    let max_layer = layer.values().copied().max().unwrap_or(0);
    let mut by_layer: Vec<Vec<String>> = vec![Vec::new(); max_layer + 1];
    for node in &graph.nodes {
        by_layer[layer[node.service.as_str()]].push(node.service.clone());
    }
    let mut prev_pos: BTreeMap<String, usize> = BTreeMap::new();
    for (l, members) in by_layer.iter_mut().enumerate() {
        if l == 0 {
            members.sort();
        } else {
            let key = |name: &str, prev_pos: &BTreeMap<String, usize>| -> f64 {
                let ps: Vec<f64> = preds
                    .get(name)
                    .map(|v| {
                        v.iter()
                            .filter_map(|p| prev_pos.get(*p).map(|i| *i as f64))
                            .collect()
                    })
                    .unwrap_or_default();
                if ps.is_empty() {
                    f64::MAX
                } else {
                    ps.iter().sum::<f64>() / ps.len() as f64
                }
            };
            members.sort_by(|a, b| {
                key(a, &prev_pos)
                    .partial_cmp(&key(b, &prev_pos))
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.cmp(b))
            });
        }
        prev_pos = members
            .iter()
            .enumerate()
            .map(|(i, s)| (s.clone(), i))
            .collect();
    }
    let mut out = Vec::new();
    for (l, members) in by_layer.into_iter().enumerate() {
        for (i, name) in members.into_iter().enumerate() {
            out.push((name, l, i));
        }
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn render_sankey(
    scene: &Scene,
    spans: &[Span],
    panel: &Panel,
    colors: &BTreeMap<String, String>,
    prims: &mut Vec<Prim>,
    y: &mut f64,
) -> Result<(), RenderError> {
    let Panel::Sankey { data, residual } = panel else {
        return Err(fail("render_sankey called on a non-sankey panel"));
    };
    let set_owned: Vec<Span> = dataset(scene, spans, data).into_iter().cloned().collect();
    if set_owned.is_empty() {
        return Err(fail("sankey has no spans in its data set"));
    }
    let flow = sankey_flow(&set_owned, residual);
    // Layout: stages are columns, a node's block height is the
    // greater of its span count and its outflow on one scale set by
    // the tallest stage, and links are trapezoids stacked in node
    // order at both ends. Past seven stages the columns narrow
    // instead of overlapping: the block width shrinks with the
    // column step and the labels shorten to fit.
    let plot_w = WIDTH - 2.0 * MARGIN;
    let nominal_step = if flow.stages > 1 {
        (plot_w - 120.0) / (flow.stages - 1) as f64
    } else {
        0.0
    };
    let node_w = if flow.stages > 7 {
        (nominal_step * 0.6).max(8.0)
    } else {
        120.0
    };
    let label_chars = if flow.stages > 7 {
        ((nominal_step / 6.5) as usize).max(4)
    } else {
        24
    };
    let gap = 18.0;
    let step = if flow.stages > 1 {
        (plot_w - node_w) / (flow.stages - 1) as f64
    } else {
        0.0
    };
    let mut stage_totals: Vec<u64> = vec![0; flow.stages];
    for node in &flow.nodes {
        stage_totals[node.stage] += node.value.max(node.outflow);
    }
    let max_total = stage_totals.iter().copied().max().unwrap_or(1).max(1);
    let unit = (340.0 / max_total as f64).min(6.0);
    let top = *y + 20.0;
    let mut positions: BTreeMap<(usize, String, bool), (f64, f64, f64)> = BTreeMap::new();
    let mut stage_heights: Vec<f64> = vec![0.0; flow.stages];
    for (stage, height) in stage_heights.iter_mut().enumerate() {
        let mut cursor = top;
        for node in flow.nodes.iter().filter(|n| n.stage == stage) {
            let h = node.value.max(node.outflow) as f64 * unit;
            positions.insert(
                (node.stage, node.service.clone(), node.residual),
                (MARGIN + stage as f64 * step, cursor, h),
            );
            cursor += h + gap;
        }
        *height = cursor - gap - top;
    }
    // Links first, so the blocks draw over their ends. Stacking order
    // at both faces follows the node order, so the picture is a pure
    // function of the derived flow.
    let order: BTreeMap<(usize, String, bool), usize> = flow
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| ((n.stage, n.service.clone(), n.residual), i))
        .collect();
    let mut links: Vec<&SankeyLink> = flow.links.iter().collect();
    links.sort_by_key(|l| {
        (
            order[&(l.from_stage, l.from_service.clone(), l.from_residual)],
            order[&(l.to_stage, l.to_service.clone(), false)],
        )
    });
    let mut out_cursor: BTreeMap<(usize, String, bool), f64> = BTreeMap::new();
    let mut in_cursor: BTreeMap<(usize, String, bool), f64> = BTreeMap::new();
    for link in links {
        let fkey = (
            link.from_stage,
            link.from_service.clone(),
            link.from_residual,
        );
        let tkey = (link.to_stage, link.to_service.clone(), false);
        let (fx, ftop, _) = positions[&fkey];
        let (tx, ttop, _) = positions[&tkey];
        let h = link.value as f64 * unit;
        let out_at = out_cursor.entry(fkey).or_insert(ftop);
        let s1 = *out_at;
        *out_at += h;
        let in_at = in_cursor.entry(tkey).or_insert(ttop);
        let t1 = *in_at;
        *in_at += h;
        let fill = if link.from_residual {
            "#999999".to_string()
        } else {
            colors
                .get(&link.from_service)
                .cloned()
                .unwrap_or_else(|| "#999999".to_string())
        };
        prims.push(Prim::Polygon {
            points: vec![
                (fx + node_w, s1),
                (tx, t1),
                (tx, t1 + h),
                (fx + node_w, s1 + h),
            ],
            fill,
            opacity: Some(0.45),
        });
    }
    // Blocks, each labeled in the gap above it with its service and
    // its span count. The residual block is gray: it is a bucket,
    // not a service.
    for node in &flow.nodes {
        let (x, ntop, h) = positions[&(node.stage, node.service.clone(), node.residual)];
        let fill = if node.residual {
            "#999999".to_string()
        } else {
            colors
                .get(&node.service)
                .cloned()
                .unwrap_or_else(|| "#999999".to_string())
        };
        text(
            prims,
            x,
            ntop - 5.0,
            11.0,
            INK,
            false,
            false,
            format!(
                "{} · {}",
                truncate_label(&node.service, label_chars),
                plural(node.value, "span", "spans")
            ),
        );
        prims.push(Prim::Rect {
            x,
            y: ntop,
            w: node_w,
            h,
            fill,
            opacity: None,
            stroke: None,
        });
    }
    let tallest = stage_heights.iter().copied().fold(0.0, f64::max);
    *y = top + tallest + 6.0;
    if flow.residual_spans > 0 {
        text(
            prims,
            MARGIN,
            *y + 12.0,
            11.0,
            MUTED,
            false,
            false,
            format!(
                "{} entered through the {} bucket because their parent chain could not stage them",
                plural(flow.residual_spans, "span", "spans"),
                residual
            ),
        );
        *y += 15.0;
    }
    *y += 7.0;
    Ok(())
}

fn render_service_map(
    scene: &Scene,
    spans: &[Span],
    panel: &Panel,
    colors: &BTreeMap<String, String>,
    prims: &mut Vec<Prim>,
    y: &mut f64,
) -> Result<(), RenderError> {
    let Panel::ServiceMap { data } = panel else {
        return Err(fail("render_service_map called on a non-service-map panel"));
    };
    let set_owned: Vec<Span> = dataset(scene, spans, data).into_iter().cloned().collect();
    if set_owned.is_empty() {
        return Err(fail("service map has no spans in its data set"));
    }
    let window_seconds =
        ((scene.context.range.to_unix_ms - scene.context.range.from_unix_ms) as f64 / 1000.0)
            .max(0.001);
    let graph = service_graph(&set_owned, window_seconds);
    let positions = map_positions(&graph);
    let layers = positions.iter().map(|(_, l, _)| *l).max().unwrap_or(0) + 1;
    let rows = positions.iter().map(|(_, _, i)| *i).max().unwrap_or(0) + 1;
    let step = (WIDTH - 2.0 * MARGIN) / layers as f64;
    // The width floors at eight pixels, the same narrowing the
    // sankey applies past seven stages, so a deep or cyclic
    // graph narrows its boxes instead of driving the width and
    // the label budget negative.
    let box_w = (step - 10.0).min(160.0).max(8.0);
    let box_h = 32.0;
    let row_step = 46.0;
    let map_top = *y;
    let mut centers: BTreeMap<String, (f64, f64)> = BTreeMap::new();
    for (name, l, i) in &positions {
        let bx = MARGIN + *l as f64 * step + 5.0;
        let by = map_top + *i as f64 * row_step;
        centers.insert(name.clone(), (bx, by));
    }
    // Edges first, so the boxes draw over their ends.
    let max_calls = graph.edges.iter().map(|e| e.calls).max().unwrap_or(1);
    let thresholds = Thresholds::error_ratio();
    for edge in &graph.edges {
        let (fx, fy) = centers[edge.from.as_str()];
        let (tx, ty) = centers[edge.to.as_str()];
        let ratio = if edge.calls == 0 {
            0.0
        } else {
            edge.errors as f64 / edge.calls as f64
        };
        prims.push(Prim::Line {
            x1: fx + box_w,
            y1: fy + box_h / 2.0,
            x2: tx,
            y2: ty + box_h / 2.0,
            stroke: thresholds.color_for(ratio, "#888888"),
            width: 1.0 + 3.0 * (edge.calls as f64 / max_calls as f64),
        });
        prims.push(Prim::Text {
            x: (fx + box_w + tx) / 2.0,
            y: (fy + ty) / 2.0 + box_h / 2.0 - 4.0,
            size: 10.0,
            fill: MUTED.to_string(),
            bold: false,
            mono: false,
            anchor: "middle",
            content: format!(
                "{} · p99 {}",
                plural(edge.calls, "call", "calls"),
                fmt_duration(edge.p99_nanos)
            ),
        });
    }
    // Boxes, with the service color and its call and error counts.
    for node in &graph.nodes {
        let (bx, by) = centers[node.service.as_str()];
        let fill = colors
            .get(&node.service)
            .cloned()
            .unwrap_or_else(|| "#999999".to_string());
        prims.push(Prim::Rect {
            x: bx,
            y: by,
            w: box_w,
            h: box_h,
            fill: fill.clone(),
            opacity: None,
            stroke: None,
        });
        let text_fill = bar_text_fill(&fill).to_string();
        prims.push(Prim::Text {
            x: bx + 8.0,
            y: by + 14.0,
            size: 11.0,
            fill: text_fill.clone(),
            bold: true,
            mono: false,
            anchor: "start",
            content: truncate_label(
                &node.service,
                (((box_w - 12.0) / 6.5) as usize).max(4),
            ),
        });
        prims.push(Prim::Text {
            x: bx + 8.0,
            y: by + 27.0,
            size: 10.0,
            fill: text_fill,
            bold: false,
            mono: false,
            anchor: "start",
            content: format!(
                "{} · {}",
                plural(node.calls, "call", "calls"),
                plural(node.errors, "error", "errors")
            ),
        });
    }
    // The legend table doubles as the ranked summary: services by
    // calls descending, with the full RED line for each.
    *y = map_top + rows as f64 * row_step + 2.0;
    let mut ranked: Vec<&RedStats> = graph.nodes.iter().collect();
    ranked.sort_by(|a, b| {
        b.calls
            .cmp(&a.calls)
            .then_with(|| a.service.cmp(&b.service))
    });
    for node in ranked {
        text(
            prims,
            MARGIN,
            *y + 12.0,
            11.0,
            MUTED,
            false,
            false,
            format!(
                "{} · {} · {} ({}) · p50 {} · p99 {}",
                node.service,
                plural(node.calls, "call", "calls"),
                plural(node.errors, "error", "errors"),
                telorama_scene::format_ratio(node.error_ratio),
                fmt_duration(node.p50_nanos),
                fmt_duration(node.p99_nanos),
            ),
        );
        *y += 15.0;
    }
    *y += 7.0;
    Ok(())
}

/// FNV-1a over a frame name, for palette assignment: the same frame
/// keeps its color wherever it appears in the tree.
fn name_hash(name: &str) -> usize {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in name.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    (h % SERVICE_PALETTE.len() as u64) as usize
}

fn flame_depth(node: &FlameNode) -> usize {
    1 + node.children.iter().map(flame_depth).max().unwrap_or(0)
}

/// Text color for a bar of the given fill: white on dark fills, ink on
/// light ones, by perceived luminance.
fn bar_text_fill(fill: &str) -> &'static str {
    let hex = fill.trim_start_matches('#');
    if hex.len() != 6 {
        return INK;
    }
    let channel = |i: usize| u32::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0) as f64;
    let luminance = 0.299 * channel(0) + 0.587 * channel(2) + 0.114 * channel(4);
    if luminance < 140.0 { "#FFFFFF" } else { INK }
}

fn layout_flame(
    prims: &mut Vec<Prim>,
    node: &FlameNode,
    x: f64,
    w: f64,
    y: f64,
    total: u64,
    depth: usize,
) {
    let fill = if depth == 0 {
        "#E8E8E8".to_string()
    } else {
        SERVICE_PALETTE[name_hash(&node.name)].to_string()
    };
    prims.push(Prim::Rect {
        x,
        y,
        w,
        h: 14.0,
        fill: fill.clone(),
        opacity: None,
        stroke: Some(("#FFFFFF".to_string(), 0.5)),
    });
    if w >= 26.0 && total > 0 {
        let frac = node.value as f64 / total as f64;
        let label = truncate_label(
            &format!("{} · {}", node.name, telorama_scene::format_ratio(frac)),
            (w / 6.0) as usize,
        );
        prims.push(Prim::Text {
            x: x + 4.0,
            y: y + 11.0,
            size: 10.0,
            fill: bar_text_fill(&fill).to_string(),
            bold: false,
            mono: false,
            anchor: "start",
            content: label,
        });
    }
    if node.value > 0 {
        let mut child_x = x;
        for child in &node.children {
            let child_w = w * (child.value as f64 / node.value as f64);
            layout_flame(prims, child, child_x, child_w, y + 16.0, total, depth + 1);
            child_x += child_w;
        }
    }
}

fn render_flame(
    scene: &Scene,
    data: &SceneData,
    panel: &Panel,
    prims: &mut Vec<Prim>,
    y: &mut f64,
) -> Result<(), RenderError> {
    let Panel::Flame {
        data: data_id,
        sample_type,
        min_samples,
        sample_count,
    } = panel
    else {
        return Err(fail("render_flame called on a non-flame panel"));
    };
    let entry = scene
        .data
        .iter()
        .find(|d| d.id == *data_id)
        .ok_or_else(|| fail(format!("flame reads undeclared data id {data_id:?}")))?;
    let (tree, note) = if let Some(trace_id) = entry.source.trace_id.as_deref() {
        let tree = flame_from_spans(data.spans, trace_id);
        if tree.children.is_empty() {
            return Err(fail(format!("no spans found for trace {trace_id}")));
        }
        let note = format!(
            "Derived from span self time, not sampled profiles. The trace holds {} of self time across its spans.",
            fmt_duration(tree.value as i64)
        );
        (tree, note)
    } else {
        if data.stacks.is_empty() {
            return Err(fail(
                "flame panel has no profile data: supply folded stacks, or point the panel's data at a trace for the span fallback",
            ));
        }
        let tree = flame_tree(data.stacks);
        if tree.value != *sample_count {
            return Err(fail(format!(
                "declared sample count {sample_count} does not match the profile data total {}: a scene must not state a count its data does not carry",
                tree.value
            )));
        }
        let note = format!(
            "{} samples of {} from {} folded stacks.",
            tree.value,
            sample_type,
            data.stacks.len()
        );
        (tree, note)
    };
    if tree.value < *min_samples {
        return Err(fail(format!(
            "flame graph has {} samples, below its stated floor of {}",
            tree.value, min_samples
        )));
    }
    let depth = flame_depth(&tree);
    if depth > 48 {
        return Err(fail(format!(
            "flame tree is {depth} levels deep, beyond the 48-level render limit"
        )));
    }
    let plot_w = WIDTH - 2.0 * MARGIN;
    layout_flame(prims, &tree, MARGIN, plot_w, *y, tree.value, 0);
    *y += depth as f64 * 16.0 + 6.0;
    text(prims, MARGIN, *y + 12.0, 11.0, MUTED, false, false, note);
    *y += 22.0;
    Ok(())
}

/// The band a heatmap row covers, in words: "up to 10 ms", "10 ms to
/// 50 ms", "over 500 ms".
fn band_label(edges: &[f64], row: usize) -> String {
    let fmt_edge = |s: f64| fmt_duration((s * 1_000_000_000.0).round() as i64);
    match edges.get(row) {
        Some(upper) if row == 0 => format!("up to {}", fmt_edge(*upper)),
        Some(upper) => format!("{} to {}", fmt_edge(edges[row - 1]), fmt_edge(*upper)),
        None => format!("over {}", fmt_edge(edges[edges.len() - 1])),
    }
}

/// Cell shading for a count, in four steps up to the busiest cell.
/// The full step returns no opacity (solid); an empty cell is drawn by
/// the caller in the empty grey, never by this function.
fn cell_shade(count: u64, max: u64) -> Option<f64> {
    if count == 0 || max == 0 {
        return None;
    }
    let frac = count as f64 / max as f64;
    if frac <= 0.25 {
        Some(0.35)
    } else if frac <= 0.5 {
        Some(0.55)
    } else if frac <= 0.75 {
        Some(0.8)
    } else {
        None
    }
}

fn render_heatmap(
    scene: &Scene,
    spans: &[Span],
    panel: &Panel,
    prims: &mut Vec<Prim>,
    y: &mut f64,
) -> Result<(), RenderError> {
    let Panel::Heatmap { data, unit } = panel else {
        return Err(fail("render_heatmap called on a non-heatmap panel"));
    };
    if *unit != Unit::Duration {
        return Err(fail(
            "heatmap over spans needs unit duration: other units land with their data sources",
        ));
    }
    let entry = scene
        .data
        .iter()
        .find(|d| d.id == *data)
        .ok_or_else(|| fail(format!("heatmap reads undeclared data id {data:?}")))?;
    let edges = entry
        .source
        .buckets
        .clone()
        .ok_or_else(|| fail("heatmap data entry declares no bucket edges"))?;
    if edges.len() < 2 {
        return Err(fail("heatmap bucket edges need at least two edges"));
    }
    let set_owned: Vec<Span> = dataset(scene, spans, data).into_iter().cloned().collect();
    let grid = heatmap_grid(&set_owned, scene.context.range, &edges, 24);

    let plot_x0 = 200.0;
    let plot_x1 = WIDTH - MARGIN;
    let col_w = (plot_x1 - plot_x0) / grid.columns as f64;
    let window_nano =
        (scene.context.range.to_unix_ms - scene.context.range.from_unix_ms) as f64 * 1_000_000.0;

    // Time axis, the same five ticks the waterfall uses.
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
        let tx = plot_x0 + frac * (plot_x1 - plot_x0);
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
            content: fmt_duration((frac * window_nano) as i64),
        });
    }
    *y += 22.0;

    // Grid rows, highest band at the top.
    let row_h = 15.0;
    let row_count = grid.rows.len();
    for display in 0..row_count {
        let r = row_count - 1 - display;
        let row = &grid.rows[r];
        let label = match row.upper_edge_seconds {
            Some(e) => fmt_duration((e * 1_000_000_000.0).round() as i64),
            None => "+Inf".to_string(),
        };
        prims.push(Prim::Text {
            x: plot_x0 - 8.0,
            y: *y + 11.0,
            size: 11.0,
            fill: MUTED.to_string(),
            bold: false,
            mono: false,
            anchor: "end",
            content: label,
        });
        for (c, count) in row.counts.iter().enumerate() {
            let is_exemplar = grid
                .exemplar
                .as_ref()
                .is_some_and(|e| e.row == r && e.col == c);
            let (fill, opacity) = if *count == 0 {
                ("#F2F2F2".to_string(), None)
            } else {
                ("#56B4E9".to_string(), cell_shade(*count, grid.max_count))
            };
            prims.push(Prim::Rect {
                x: plot_x0 + c as f64 * col_w + 0.5,
                y: *y,
                w: col_w - 1.0,
                h: row_h - 2.0,
                fill,
                opacity,
                stroke: if is_exemplar {
                    Some((ERROR_COLOR.to_string(), 1.5))
                } else {
                    None
                },
            });
        }
        *y += row_h;
    }

    // Legend table: one line per band with its total, then the shading
    // rule, then the exemplar, then anything the window excluded.
    *y += 6.0;
    for r in (0..row_count).rev() {
        let row = &grid.rows[r];
        let noun = if row.total == 1 { "span" } else { "spans" };
        text(
            prims,
            MARGIN,
            *y + 12.0,
            11.0,
            MUTED,
            false,
            false,
            format!("{} · {} {}", band_label(&edges, r), row.total, noun),
        );
        *y += 15.0;
    }
    if grid.max_count == 0 {
        text(
            prims,
            MARGIN,
            *y + 12.0,
            11.0,
            MUTED,
            false,
            false,
            "No spans fell in the window.".to_string(),
        );
        *y += 15.0;
    } else {
        text(
            prims,
            MARGIN,
            *y + 12.0,
            11.0,
            MUTED,
            false,
            false,
            format!(
                "Cell shading is the span count in the cell, in four steps up to the busiest cell at {}.",
                grid.max_count
            ),
        );
        *y += 15.0;
    }
    if let Some(exemplar) = &grid.exemplar {
        text(
            prims,
            MARGIN,
            *y + 12.0,
            11.0,
            MUTED,
            false,
            false,
            format!(
                "Exemplar trace {} · {} {} · {} · the longest span in the busiest cell, outlined in the grid.",
                exemplar.trace_id,
                exemplar.service,
                exemplar.name,
                fmt_duration(exemplar.duration_nanos)
            ),
        );
        *y += 15.0;
    }
    if grid.outside_window > 0 {
        let noun = if grid.outside_window == 1 {
            "span"
        } else {
            "spans"
        };
        text(
            prims,
            MARGIN,
            *y + 12.0,
            11.0,
            MUTED,
            false,
            false,
            format!(
                "{} {} started outside the window and are not in the grid.",
                grid.outside_window, noun
            ),
        );
        *y += 15.0;
    }
    *y += 7.0;
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
        thresholds,
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

    let (value, derivation, raw) = match stat {
        StatKind::P50 | StatKind::P95 | StatKind::P99 => {
            let p = stat.percentile().unwrap_or(95);
            match percentile_nanos(&durations, p) {
                Some(v) => (
                    format_by_unit(v, *unit),
                    format!("{} of {} span durations in the window", stat.name(), calls),
                    Some(v as f64),
                ),
                None => ("-".to_string(), "no spans in the window".to_string(), None),
            }
        }
        StatKind::Count => (
            format!("{calls}"),
            format!("{calls} spans in the window"),
            Some(calls as f64),
        ),
        StatKind::Rate => {
            let secs = scene.context.range.seconds();
            let rate = if secs > 0.0 { calls as f64 / secs } else { 0.0 };
            (
                fmt_num(rate),
                format!("{calls} spans over a {} s window", fmt_num(secs)),
                Some(rate),
            )
        }
        StatKind::ErrorRatio => {
            let ratio = if calls == 0 {
                0.0
            } else {
                errors as f64 / calls as f64
            };
            (
                telorama_scene::format_ratio(ratio),
                format!("{errors} errors of {calls} calls"),
                Some(ratio),
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
                    Some(span.duration_nanos() as f64),
                ),
                None => ("-".to_string(), "no spans in the window".to_string(), None),
            }
        }
    };

    let mut label = format!("{} · {}", stat.name(), unit.name());
    if let Some(s) = service {
        label.push_str(&format!(" · {s}"));
    }
    text(prims, MARGIN, *y + 12.0, 12.0, MUTED, false, false, label);
    let value_fill = match raw {
        Some(v) if !thresholds.is_empty() => thresholds.color_for(v, INK),
        _ => INK.to_string(),
    };
    text(
        prims,
        MARGIN,
        *y + 44.0,
        30.0,
        &value_fill,
        true,
        false,
        value,
    );
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
    let Panel::RedRow {
        data,
        service,
        thresholds,
    } = panel
    else {
        return Err(fail("render_red_row called on a non-red panel"));
    };
    let verdicts = if thresholds.is_empty() {
        Thresholds::error_ratio()
    } else {
        thresholds.clone()
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
        let err_fill = verdicts.color_for(row.error_ratio, INK);
        text(
            prims,
            470.0,
            *y + 12.0,
            12.0,
            &err_fill,
            false,
            true,
            telorama_scene::format_ratio(row.error_ratio),
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

/// Everything a scene can read: the span set, the folded stacks for
/// flame panels, and the agent run for agent_run panels.
pub struct SceneData<'a> {
    pub spans: &'a [Span],
    pub stacks: &'a [StackSample],
    pub agent_run: Option<&'a AgentRun>,
    /// The agent-state entries, when a panel renders them.
    pub agent_state: &'a [AgentStateEntry],
    /// The investigation layer for this render, ranked findings from
    /// `telorama_scene::investigate`. Empty draws no strip.
    pub findings: &'a [Finding],
}

/// Render a scene over a span set to a deterministic SVG document.
/// Scenes whose flame panels read profile data need
/// [`render_scene_with_data`].
pub fn render_scene(scene: &Scene, spans: &[Span]) -> Result<String, RenderError> {
    render_scene_with_data(
        scene,
        &SceneData {
            spans,
            stacks: &[],
            agent_run: None,
            agent_state: &[],
            findings: &[],
        },
    )
}

/// Wrap a rendered SVG in a standalone HTML document: no scripts and
/// no external references, nothing to fetch. This is the export for
/// hosts that show HTML but do not render MCP Apps, and for saving a
/// scene as a file a person can open anywhere.
pub fn standalone_html(title: &str, svg: &str) -> String {
    format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<title>{}</title>\n<style>body{{margin:0;background:#FFFFFF;color:#1A1A1A;font-family:system-ui,sans-serif}}main{{max-width:1000px;margin:0 auto;padding:24px}}svg{{width:100%;height:auto}}</style>\n</head>\n<body>\n<main>\n{}</main>\n</body>\n</html>\n",
        escape_xml(title),
        svg
    )
}

/// Render a scene to a standalone HTML document carrying the SVG.
pub fn render_scene_html(scene: &Scene, data: &SceneData) -> Result<String, RenderError> {
    let svg = render_scene_with_data(scene, data)?;
    Ok(standalone_html(&scene.title, &svg))
}

/// Render a scene over a full data set to a deterministic SVG document.
///
/// Panels render in spec order under the headline. The provenance
/// caption at the foot of the scene prints the window, every data
/// source with its query or trace id and sampling state, and the
/// generating question, so the picture carries its own evidence.
pub fn render_scene_with_data(scene: &Scene, data: &SceneData) -> Result<String, RenderError> {
    let spans = data.spans;
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

    // The investigation strip. Ranked findings sit directly under the
    // headline so the first thing a reader sees after the title is
    // where to look, with the evidence stated on each line.
    if !data.findings.is_empty() {
        y += 10.0;
        text(
            &mut prims,
            MARGIN,
            y,
            12.0,
            INK,
            true,
            false,
            "Look here first".to_string(),
        );
        y += 16.0;
        for finding in data.findings {
            text(
                &mut prims,
                MARGIN,
                y,
                12.0,
                INK,
                false,
                false,
                format!("{} · {}", finding.rank, finding.statement),
            );
            y += 15.0;
        }
        y -= 3.0;
    }

    for panel in &scene.panels {
        y += 18.0;
        let heading = match panel {
            Panel::Waterfall { .. } => "Waterfall",
            Panel::Stat { .. } => "Stat",
            Panel::RedRow { .. } => "Rate, errors, duration by service",
            Panel::Heatmap { .. } => "Latency heatmap",
            Panel::Flame { .. } => "Flame graph",
            Panel::ServiceMap { .. } => "Service map",
            Panel::Sankey { .. } => "Sankey",
            Panel::AgentRun { .. } => "Agent run",
            Panel::AgentState { .. } => "Agent state",
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
            Panel::Heatmap { .. } => render_heatmap(scene, spans, panel, &mut prims, &mut y)?,
            Panel::Flame { .. } => render_flame(scene, data, panel, &mut prims, &mut y)?,
            Panel::ServiceMap { .. } => {
                render_service_map(scene, spans, panel, &colors, &mut prims, &mut y)?
            }
            Panel::Sankey { .. } => {
                render_sankey(scene, spans, panel, &colors, &mut prims, &mut y)?
            }
            Panel::AgentRun { .. } => render_agent_run(scene, data, panel, &mut prims, &mut y)?,
            Panel::AgentState { .. } => render_agent_state(scene, data, panel, &mut prims, &mut y)?,
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
