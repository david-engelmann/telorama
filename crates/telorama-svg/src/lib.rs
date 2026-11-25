//! telorama-svg: the DisplayList and the deterministic SVG emitter.
//!
//! Rendering is a pure function of the scene and the data. Layout is
//! computed here, in Rust, from sorted inputs; floats are formatted by
//! one canonical function; element order is insertion order over sorted
//! data. The same scene and the same data produce the same bytes.
//!
//! Layout uses fixed columns and character class widths rather
//! than measured text, so the output is byte-identical on every
//! machine without a font dependency. A character's width is a
//! function of its class (INV-92). Combining marks count zero,
//! East Asian wide and fullwidth characters count two, and
//! everything else, the East Asian ambiguous class included,
//! counts one.
//!
//! # Example
//!
//! Render a validated scene and its spans to an SVG string.
//!
//! ```rust
//! let scene_text = r#"{
//!     "scene": "0.1",
//!     "title": "One request",
//!     "headline": "The request answered in 500 ms",
//!     "context": {
//!         "range": {"from_unix_ms": 0, "to_unix_ms": 1000},
//!         "provenance": {"generated_by": "agent", "question": "How long did it take?"}
//!     },
//!     "data": [{
//!         "id": "trace",
//!         "sampled": "unknown",
//!         "source": {"kind": "otlp_file", "trace_id": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}
//!     }],
//!     "panels": [{"type": "waterfall", "data": "trace"}]
//! }"#;
//! let export = r#"{"resourceSpans": [{
//!     "resource": {"attributes": [
//!         {"key": "service.name", "value": {"stringValue": "checkout"}}
//!     ]},
//!     "scopeSpans": [{"spans": [{
//!         "traceId": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
//!         "spanId": "bbbbbbbbbbbbbbbb",
//!         "name": "GET /checkout",
//!         "kind": 2,
//!         "startTimeUnixNano": "0",
//!         "endTimeUnixNano": "500000000"
//!     }]}]
//! }]}"#;
//! let scene = telorama_scene::parse_scene(scene_text).expect("the scene must validate");
//! let spans = telorama_ingest::parse_otlp_traces(export).expect("the export must parse");
//! let svg = telorama_svg::render_scene(&scene, &spans).expect("the scene must render");
//! assert!(svg.contains("One request"));
//! ```

#![deny(missing_docs)]

use std::collections::BTreeMap;
use std::fmt;
use telorama_scene::{
    AgentRun, AgentStateEntry, AgentStep, Finding, FlameNode, Panel, RedStats, SankeyLink, Scene,
    ServiceGraph, Span, StackSample, StatKind, Thresholds, Unit, critical_path,
    dismantle_flame_tree, flame_from_spans, flame_tree, heatmap_grid, percentile_nanos,
    red_by_service, sankey_flow, service_graph, waterfall_rows,
};

/// Canonical float formatting for emitted geometry. The implementation
/// lives in telorama-scene's field system, so every surface formats a
/// value the same way. These are the renderer's handles on it.
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
    /// A filled rectangle, emitted as an SVG `rect` element.
    Rect {
        /// Left edge in SVG user units.
        x: f64,
        /// Top edge in SVG user units.
        y: f64,
        /// Width in SVG user units.
        w: f64,
        /// Height in SVG user units.
        h: f64,
        /// Fill color as a CSS color string.
        fill: String,
        /// Fill opacity from 0 to 1. `None` emits no opacity
        /// attribute, so the fill is fully opaque.
        opacity: Option<f64>,
        /// Border as a color and a width in SVG user units.
        /// `None` emits no stroke.
        stroke: Option<(String, f64)>,
    },
    /// A straight segment, emitted as an SVG `line` element.
    Line {
        /// Start point's horizontal coordinate in SVG user units.
        x1: f64,
        /// Start point's vertical coordinate in SVG user units.
        y1: f64,
        /// End point's horizontal coordinate in SVG user units.
        x2: f64,
        /// End point's vertical coordinate in SVG user units.
        y2: f64,
        /// Stroke color as a CSS color string.
        stroke: String,
        /// Stroke width in SVG user units.
        width: f64,
    },
    /// A filled shape through a list of vertices, emitted as an
    /// SVG `polygon` element. Sankey links are drawn this way.
    Polygon {
        /// Vertices in draw order, each a pair of coordinates in
        /// SVG user units.
        points: Vec<(f64, f64)>,
        /// Fill color as a CSS color string.
        fill: String,
        /// Fill opacity from 0 to 1. `None` emits no opacity
        /// attribute, so the fill is fully opaque.
        opacity: Option<f64>,
    },
    /// A run of text, emitted as an SVG `text` element.
    Text {
        /// Horizontal position of the anchor point in SVG user
        /// units.
        x: f64,
        /// Vertical position of the text baseline in SVG user
        /// units.
        y: f64,
        /// Font size in SVG user units.
        size: f64,
        /// Text color as a CSS color string.
        fill: String,
        /// When true, the text renders at font weight 700.
        bold: bool,
        /// When true, the text renders in a monospace family
        /// instead of the default sans family.
        mono: bool,
        /// The SVG `text-anchor` value, one of `start`, `middle`,
        /// or `end`.
        anchor: &'static str,
        /// The literal characters drawn, XML-escaped at emission.
        content: String,
    },
}

/// An ordered list of primitives plus the canvas identity. Emitting the
/// same DisplayList always produces the same SVG bytes.
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayList {
    /// Canvas width in SVG user units. Also the `viewBox` width.
    pub width: f64,
    /// Canvas height in SVG user units. Also the `viewBox`
    /// height.
    pub height: f64,
    /// Emitted as the SVG document's `title` element, the
    /// accessible name of the drawing.
    pub title: String,
    /// Emitted as the SVG document's `desc` element, a one-line
    /// summary of what the drawing shows.
    pub description: String,
    /// Color of the full-canvas rectangle painted behind every
    /// primitive, as a CSS color string.
    pub background: String,
    /// The primitives in draw order. Later entries paint over
    /// earlier ones.
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
        out.push_str(&format!("\" fill=\"{}\"/>\n", self.background));
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
    /// The specific reason rendering failed, worded for the
    /// caller to show as-is.
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

/// The renderer's own colors under one theme. Every color the
/// renderer chooses resolves through this record, so a theme is
/// a complete, auditable list rather than a scatter of literals.
/// Colors a scene declares for itself are data, not chrome, and
/// never pass through here (INV-74).
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    /// Ground color painted behind the whole canvas, and the
    /// page background of the themed HTML export.
    pub background: &'static str,
    /// Fill of the flame graph's root box, the one box that
    /// carries no service color.
    pub surface: &'static str,
    /// Fill of a heatmap cell whose count is zero.
    pub empty_cell: &'static str,
    /// Primary text color, for values and labels a reader must
    /// not miss.
    pub ink: &'static str,
    /// Secondary text color, for captions, notes, and axis
    /// labels.
    pub muted: &'static str,
    /// Stroke color of gridlines and axis rules.
    pub grid: &'static str,
    /// Color that marks errors under this theme, on error spans
    /// and error counts.
    pub error: &'static str,
    /// The eight categorical colors assigned to services in
    /// sorted service-name order.
    pub service: [&'static str; 8],
    /// Neutral gray used when a mark has no assigned service
    /// color, and the base a verdict threshold blends from.
    pub fallback: &'static str,
    /// Color of the sankey residual bucket, for spans whose
    /// parent chain could not be staged.
    pub residual: &'static str,
    /// Fill of the permission-wait segment at the head of a
    /// tool-call bar in agent-run panels.
    pub waiting: &'static str,
    /// Color of the critical-path highlight in waterfall
    /// panels, a stroke around path bars and a band beneath
    /// them.
    pub outline: &'static str,
    /// Thin stroke separating adjacent flame boxes, so
    /// neighbouring fills stay distinct.
    pub flame_stroke: &'static str,
    /// Color nonzero heatmap cells are filled with. Each cell's
    /// opacity carries its share of the maximum count.
    pub heatmap_base: &'static str,
    /// True in the dark palette. Filled-bar labels use it to
    /// pick between the background and ink colors by contrast.
    pub dark: bool,
}

/// The light palette. These are the colors the renderer has
/// always emitted, gathered in one place. The light theme's
/// bytes do not change (INV-74).
pub const LIGHT_PALETTE: Palette = Palette {
    background: "#FFFFFF",
    surface: "#E8E8E8",
    empty_cell: "#F2F2F2",
    ink: INK,
    muted: MUTED,
    grid: "#CCCCCC",
    error: ERROR_COLOR,
    service: SERVICE_PALETTE,
    fallback: "#888888",
    residual: "#999999",
    waiting: "#BBBBBB",
    outline: "#111111",
    flame_stroke: "#FFFFFF",
    heatmap_base: "#56B4E9",
    dark: false,
};

/// The dark palette, derived from Okabe-Ito by lightening each
/// entry only as far as its contrast floor on the dark ground
/// requires (INV-75). The yellow and the sky blue already clear
/// their floors and carry over unchanged. The black entry
/// cannot appear on a dark ground and becomes a pale gray.
pub const DARK_PALETTE: Palette = Palette {
    background: "#121212",
    surface: "#1E1E1E",
    empty_cell: "#1E1E1E",
    ink: "#EDEDED",
    muted: "#A3A3A3",
    grid: "#3A3A3A",
    error: "#EE8434",
    service: [
        "#4DA3D8", "#E69F00", "#2FBF8F", "#EE8434", "#56B4E9", "#CC79A7", "#F0E442", "#BDBDBD",
    ],
    fallback: "#9A9A9A",
    residual: "#8A8A8A",
    waiting: "#767676",
    outline: "#EDEDED",
    flame_stroke: "#121212",
    heatmap_base: "#56B4E9",
    dark: true,
};

/// The palette a scene's declared theme renders with.
pub fn palette_for(theme: telorama_scene::Theme) -> Palette {
    match theme {
        telorama_scene::Theme::Light => LIGHT_PALETTE,
        telorama_scene::Theme::Dark => DARK_PALETTE,
    }
}

/// Service colors under the light palette, keyed by service
/// name. Kept for callers that render without a theme.
pub fn service_colors(spans: &[Span]) -> BTreeMap<String, String> {
    service_colors_for(spans, &LIGHT_PALETTE)
}

/// Service colors under a palette. Assignment is by sorted
/// service name under both themes, so a service keeps its
/// palette slot across themes as well as across panels.
pub fn service_colors_for(spans: &[Span], palette: &Palette) -> BTreeMap<String, String> {
    let mut services: Vec<&str> = spans.iter().map(|s| s.service.as_str()).collect();
    services.sort_unstable();
    services.dedup();
    services
        .iter()
        .enumerate()
        .map(|(i, name)| {
            (
                name.to_string(),
                palette.service[i % palette.service.len()].to_string(),
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

/// Code point ranges whose characters take no width of their
/// own. The principal non-spacing mark blocks of the Unicode
/// Character Database (general categories Mn and Me), the
/// variation selectors, and the zero width format characters.
/// The list is the principal blocks, stated as such, not an
/// exhaustive category dump.
const ZERO_WIDTH_RANGES: &[(u32, u32)] = &[
    (0x0300, 0x036F),
    (0x0483, 0x0489),
    (0x0591, 0x05BD),
    (0x05BF, 0x05BF),
    (0x05C1, 0x05C2),
    (0x05C4, 0x05C7),
    (0x0610, 0x061A),
    (0x064B, 0x065F),
    (0x0670, 0x0670),
    (0x06D6, 0x06DC),
    (0x06DF, 0x06E4),
    (0x06E7, 0x06E8),
    (0x06EA, 0x06ED),
    (0x0711, 0x0711),
    (0x0730, 0x074A),
    (0x07A6, 0x07B0),
    (0x07EB, 0x07F3),
    (0x0816, 0x0819),
    (0x081B, 0x0823),
    (0x0825, 0x0827),
    (0x0829, 0x082D),
    (0x0859, 0x085B),
    (0x08D3, 0x0902),
    (0x093A, 0x093A),
    (0x093C, 0x093C),
    (0x0941, 0x0948),
    (0x094D, 0x094D),
    (0x0951, 0x0957),
    (0x0962, 0x0963),
    (0x0981, 0x0981),
    (0x09BC, 0x09BC),
    (0x09C1, 0x09C4),
    (0x09CD, 0x09CD),
    (0x09E2, 0x09E3),
    (0x0A01, 0x0A03),
    (0x0A3C, 0x0A3C),
    (0x0A41, 0x0A48),
    (0x0A4D, 0x0A4D),
    (0x0A51, 0x0A51),
    (0x0A70, 0x0A71),
    (0x0A75, 0x0A75),
    (0x0A81, 0x0A82),
    (0x0ABC, 0x0ABC),
    (0x0AC1, 0x0AC5),
    (0x0AC7, 0x0AC8),
    (0x0ACD, 0x0ACD),
    (0x0AE2, 0x0AE3),
    (0x0B01, 0x0B01),
    (0x0B3C, 0x0B3C),
    (0x0B3F, 0x0B3F),
    (0x0B41, 0x0B44),
    (0x0B4D, 0x0B4D),
    (0x0B56, 0x0B56),
    (0x0B62, 0x0B63),
    (0x0B82, 0x0B82),
    (0x0BC0, 0x0BC0),
    (0x0BCD, 0x0BCD),
    (0x0C00, 0x0C00),
    (0x0C3E, 0x0C40),
    (0x0C46, 0x0C48),
    (0x0C4A, 0x0C4D),
    (0x0C55, 0x0C56),
    (0x0C62, 0x0C63),
    (0x0C81, 0x0C81),
    (0x0CBC, 0x0CBC),
    (0x0CBF, 0x0CBF),
    (0x0CC6, 0x0CC6),
    (0x0CCC, 0x0CCD),
    (0x0CE2, 0x0CE3),
    (0x0D01, 0x0D01),
    (0x0D41, 0x0D44),
    (0x0D4D, 0x0D4D),
    (0x0D62, 0x0D63),
    (0x0DCA, 0x0DCA),
    (0x0DD2, 0x0DD4),
    (0x0DD6, 0x0DD6),
    (0x0E31, 0x0E31),
    (0x0E34, 0x0E3A),
    (0x0E47, 0x0E4E),
    (0x0EB1, 0x0EB1),
    (0x0EB4, 0x0EB9),
    (0x0EBB, 0x0EBC),
    (0x0EC8, 0x0ECE),
    (0x0F18, 0x0F19),
    (0x0F35, 0x0F35),
    (0x0F37, 0x0F37),
    (0x0F39, 0x0F39),
    (0x0F71, 0x0F7E),
    (0x0F80, 0x0F84),
    (0x0F86, 0x0F87),
    (0x0F8D, 0x0F97),
    (0x0F99, 0x0FBC),
    (0x0FBE, 0x0FC5),
    (0x0FC7, 0x0FC7),
    (0x102D, 0x1030),
    (0x1032, 0x1037),
    (0x1039, 0x103A),
    (0x103D, 0x103E),
    (0x1058, 0x1059),
    (0x105E, 0x1060),
    (0x1071, 0x1074),
    (0x1082, 0x1082),
    (0x1085, 0x1086),
    (0x108D, 0x108D),
    (0x109D, 0x109D),
    (0x135D, 0x135F),
    (0x1712, 0x1714),
    (0x1732, 0x1734),
    (0x1752, 0x1753),
    (0x1772, 0x1773),
    (0x17B4, 0x17B5),
    (0x17B7, 0x17BD),
    (0x17C6, 0x17C6),
    (0x17C9, 0x17D3),
    (0x17DD, 0x17DD),
    (0x180B, 0x180E),
    (0x18A9, 0x18A9),
    (0x1920, 0x1922),
    (0x1927, 0x1928),
    (0x1932, 0x1932),
    (0x1939, 0x193B),
    (0x1A17, 0x1A18),
    (0x1A1B, 0x1A1B),
    (0x1A56, 0x1A56),
    (0x1A58, 0x1A5E),
    (0x1A60, 0x1A60),
    (0x1A62, 0x1A62),
    (0x1A65, 0x1A6C),
    (0x1A73, 0x1A7C),
    (0x1A7F, 0x1A7F),
    (0x1AB0, 0x1ABE),
    (0x1B00, 0x1B03),
    (0x1B34, 0x1B34),
    (0x1B36, 0x1B3A),
    (0x1B3C, 0x1B3C),
    (0x1B42, 0x1B42),
    (0x1B6B, 0x1B73),
    (0x1B80, 0x1B81),
    (0x1BA2, 0x1BA5),
    (0x1BA8, 0x1BA9),
    (0x1BAB, 0x1BAD),
    (0x1BE6, 0x1BE6),
    (0x1BE8, 0x1BE9),
    (0x1BED, 0x1BED),
    (0x1BEF, 0x1BF1),
    (0x1C2C, 0x1C33),
    (0x1C36, 0x1C37),
    (0x1CD0, 0x1CD2),
    (0x1CD4, 0x1CE0),
    (0x1CE2, 0x1CE8),
    (0x1CED, 0x1CED),
    (0x1CF4, 0x1CF4),
    (0x1CF8, 0x1CF9),
    (0x1DC0, 0x1DF5),
    (0x1DFC, 0x1DFF),
    (0x200B, 0x200D),
    (0x20D0, 0x20F0),
    (0x2CEF, 0x2CF1),
    (0x2D7F, 0x2D7F),
    (0x2DE0, 0x2DFF),
    (0x302A, 0x302D),
    (0x3099, 0x309A),
    (0xA66F, 0xA672),
    (0xA674, 0xA67D),
    (0xA69E, 0xA69F),
    (0xA6F0, 0xA6F1),
    (0xA802, 0xA802),
    (0xA806, 0xA806),
    (0xA80B, 0xA80B),
    (0xA825, 0xA826),
    (0xA8C4, 0xA8C4),
    (0xA8E0, 0xA8F1),
    (0xA926, 0xA92D),
    (0xA947, 0xA951),
    (0xA980, 0xA982),
    (0xA9B3, 0xA9B3),
    (0xA9B6, 0xA9B9),
    (0xA9BC, 0xA9BC),
    (0xA9E5, 0xA9E5),
    (0xAA29, 0xAA2E),
    (0xAA31, 0xAA32),
    (0xAA35, 0xAA36),
    (0xAA43, 0xAA43),
    (0xAA4C, 0xAA4C),
    (0xAA7C, 0xAA7C),
    (0xAAB0, 0xAAB0),
    (0xAAB2, 0xAAB4),
    (0xAAB7, 0xAAB8),
    (0xAABE, 0xAABF),
    (0xAAC1, 0xAAC1),
    (0xAAEC, 0xAAED),
    (0xAAF6, 0xAAF6),
    (0xABE5, 0xABE5),
    (0xABE8, 0xABE8),
    (0xABED, 0xABED),
    (0xFB1E, 0xFB1E),
    (0xFE00, 0xFE0F),
    (0xFE20, 0xFE2F),
    (0xFEFF, 0xFEFF),
    (0x101FD, 0x101FD),
    (0x102E0, 0x102E0),
    (0x10376, 0x1037A),
    (0x10A01, 0x10A03),
    (0x10A05, 0x10A06),
    (0x10A0C, 0x10A0F),
    (0x10A38, 0x10A3A),
    (0x10A3F, 0x10A3F),
    (0x10AE5, 0x10AE6),
    (0x11001, 0x11001),
    (0x11038, 0x11046),
    (0x11070, 0x11070),
    (0x11073, 0x11074),
    (0x1107F, 0x11081),
    (0x110B3, 0x110B6),
    (0x110B9, 0x110BA),
    (0x11100, 0x11102),
    (0x11127, 0x1112B),
    (0x1112D, 0x11134),
    (0x11173, 0x11173),
    (0x11180, 0x11181),
    (0x111B6, 0x111BE),
    (0x111CA, 0x111CC),
    (0x1122F, 0x11231),
    (0x11234, 0x11234),
    (0x11236, 0x11237),
    (0x112DF, 0x112DF),
    (0x112E3, 0x112EA),
    (0x11300, 0x11301),
    (0x1133C, 0x1133C),
    (0x11340, 0x11340),
    (0x11366, 0x1136C),
    (0x11370, 0x11374),
    (0x11438, 0x1143F),
    (0x11442, 0x11444),
    (0x11446, 0x11446),
    (0x114B3, 0x114B8),
    (0x114BA, 0x114BA),
    (0x114BF, 0x114C0),
    (0x114C2, 0x114C3),
    (0x115B2, 0x115B5),
    (0x115BC, 0x115BD),
    (0x115BF, 0x115C0),
    (0x115DC, 0x115DD),
    (0x11633, 0x1163A),
    (0x1163D, 0x1163D),
    (0x1163F, 0x11640),
    (0x116AB, 0x116AB),
    (0x116AD, 0x116AD),
    (0x116B0, 0x116B5),
    (0x116B7, 0x116B7),
    (0x1171D, 0x1171F),
    (0x11722, 0x11725),
    (0x11727, 0x1172B),
    (0x11A01, 0x11A0A),
    (0x11A33, 0x11A38),
    (0x11A3B, 0x11A3E),
    (0x11A47, 0x11A47),
    (0x11A51, 0x11A56),
    (0x11A59, 0x11A5B),
    (0x11A8A, 0x11A96),
    (0x11A98, 0x11A99),
    (0x11C30, 0x11C36),
    (0x11C38, 0x11C3D),
    (0x11C3F, 0x11C3F),
    (0x11C92, 0x11CA7),
    (0x11CAA, 0x11CB0),
    (0x11CB2, 0x11CB3),
    (0x11CB5, 0x11CB6),
    (0x16AF0, 0x16AF4),
    (0x16B30, 0x16B36),
    (0x16F8F, 0x16F92),
    (0x1BC9D, 0x1BC9E),
    (0x1D167, 0x1D169),
    (0x1D17B, 0x1D182),
    (0x1D185, 0x1D18B),
    (0x1D1AA, 0x1D1AD),
    (0x1D242, 0x1D244),
    (0x1DA00, 0x1DA36),
    (0x1DA3B, 0x1DA6C),
    (0x1DA75, 0x1DA75),
    (0x1DA84, 0x1DA84),
    (0x1DA9B, 0x1DA9F),
    (0x1DAA1, 0x1DAAF),
    (0x1E000, 0x1E006),
    (0x1E008, 0x1E018),
    (0x1E01B, 0x1E021),
    (0x1E023, 0x1E024),
    (0x1E026, 0x1E02A),
    (0x1E130, 0x1E136),
    (0x1E2AE, 0x1E2AE),
    (0x1E2EC, 0x1E2EF),
    (0x1E8D0, 0x1E8D6),
    (0x1E944, 0x1E94A),
    (0xE0100, 0xE01EF),
];

/// Code point ranges whose characters take two columns. The
/// wide and fullwidth values of the Unicode East Asian Width
/// property, in their principal blocks. The ambiguous class is
/// deliberately absent. The renderer's own label separator
/// and truncation ellipsis are ambiguous characters, and they
/// count one, as they did when every width was a count.
const WIDE_RANGES: &[(u32, u32)] = &[
    (0x1100, 0x115F),
    (0x2E80, 0x303E),
    (0x3041, 0x33FF),
    (0x3400, 0x4DBF),
    (0x4E00, 0x9FFF),
    (0xA000, 0xA4CF),
    (0xAC00, 0xD7A3),
    (0xF900, 0xFAFF),
    (0xFE10, 0xFE19),
    (0xFE30, 0xFE52),
    (0xFE54, 0xFE66),
    (0xFE68, 0xFE6B),
    (0xFF00, 0xFF60),
    (0xFFE0, 0xFFE6),
    (0x16FE0, 0x16FE4),
    (0x17000, 0x187F7),
    (0x18800, 0x18CD5),
    (0x1AFF0, 0x1AFF3),
    (0x1AFF5, 0x1AFFB),
    (0x1AFFD, 0x1AFFE),
    (0x1B000, 0x1B122),
    (0x1B150, 0x1B152),
    (0x1B164, 0x1B167),
    (0x1B170, 0x1B2FB),
    (0x1F004, 0x1F004),
    (0x1F0CF, 0x1F0CF),
    (0x1F18E, 0x1F18E),
    (0x1F191, 0x1F19A),
    (0x1F200, 0x1F202),
    (0x1F210, 0x1F23B),
    (0x1F240, 0x1F248),
    (0x1F250, 0x1F251),
    (0x1F260, 0x1F265),
    (0x1F300, 0x1F320),
    (0x1F32D, 0x1F335),
    (0x1F337, 0x1F37C),
    (0x1F37E, 0x1F393),
    (0x1F3A0, 0x1F3CA),
    (0x1F3CF, 0x1F3D3),
    (0x1F3E0, 0x1F3F0),
    (0x1F3F4, 0x1F3F4),
    (0x1F3F8, 0x1F43E),
    (0x1F440, 0x1F440),
    (0x1F442, 0x1F4FC),
    (0x1F4FF, 0x1F53D),
    (0x1F54B, 0x1F54E),
    (0x1F550, 0x1F567),
    (0x1F57A, 0x1F57A),
    (0x1F595, 0x1F596),
    (0x1F5A4, 0x1F5A4),
    (0x1F5FB, 0x1F64F),
    (0x1F680, 0x1F6C5),
    (0x1F6CC, 0x1F6CC),
    (0x1F6D0, 0x1F6D2),
    (0x1F6D5, 0x1F6D7),
    (0x1F6DC, 0x1F6DF),
    (0x1F6EB, 0x1F6EC),
    (0x1F6F4, 0x1F6FC),
    (0x1F7E0, 0x1F7EB),
    (0x1F7F0, 0x1F7F0),
    (0x1F90C, 0x1F93A),
    (0x1F93C, 0x1F945),
    (0x1F947, 0x1F9FF),
    (0x1FA70, 0x1FA74),
    (0x1FA78, 0x1FA7C),
    (0x1FA80, 0x1FA88),
    (0x1FA90, 0x1FABD),
    (0x1FABF, 0x1FAC5),
    (0x1FACE, 0x1FADB),
    (0x1FAE0, 0x1FAE8),
    (0x1FAF0, 0x1FAF8),
    (0x20000, 0x2FFFD),
    (0x30000, 0x3FFFD),
];

/// Whether a code point sits inside one of the ranges.
fn in_ranges(cp: u32, ranges: &[(u32, u32)]) -> bool {
    ranges.iter().any(|&(lo, hi)| (lo..=hi).contains(&cp))
}

/// The layout width of one character, in columns (INV-92).
/// Zero for a character that takes no width of its own, two
/// for a wide or fullwidth character, one for everything
/// else, the ambiguous class included.
fn char_width(c: char) -> usize {
    let cp = c as u32;
    if in_ranges(cp, ZERO_WIDTH_RANGES) {
        0
    } else if in_ranges(cp, WIDE_RANGES) {
        2
    } else {
        1
    }
}

/// The layout width of a string, in columns (INV-92). For a
/// string whose characters all count one, this is exactly the
/// character count the layout used before the width model.
fn text_width(text: &str) -> usize {
    text.chars().map(char_width).sum()
}

/// Cut a label to a width budget, in columns (INV-92). The
/// ellipsis costs one column of the budget, as it did when
/// the budget was a character count. The cut lands on a
/// character boundary and never splits a combining sequence,
/// because a zero width character always fits after the base
/// character it follows.
fn truncate_label(label: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    if text_width(label) <= max_width {
        return label.to_string();
    }
    let budget = max_width - 1;
    let mut width = 0;
    let mut out = String::new();
    for c in label.chars() {
        let w = char_width(c);
        if width + w > budget {
            break;
        }
        width += w;
        out.push(c);
    }
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
    palette: &Palette,
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
        if spans.iter().any(|s| s.trace_id == trace_id) {
            return Err(fail(format!(
                "trace {trace_id} has spans but none can root a waterfall, because every span's parent chain cycles"
            )));
        }
        return Err(fail(format!("no spans found for trace {trace_id}")));
    }
    let t0 = spans
        .iter()
        .filter(|s| s.trace_id == trace_id)
        .map(|s| s.start_unix_nano)
        .min()
        .unwrap_or(0);
    let t1 = spans
        .iter()
        .filter(|s| s.trace_id == trace_id)
        .map(|s| s.end_unix_nano)
        .max()
        .unwrap_or(0);
    let total = t1.saturating_sub(t0).max(1) as f64;
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
        stroke: palette.grid.to_string(),
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
            stroke: palette.grid.to_string(),
            width: 1.0,
        });
        prims.push(Prim::Text {
            x: tx,
            y: axis_y - 4.0,
            size: 10.0,
            fill: palette.muted.to_string(),
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
        let span = &spans[row.span_index];
        let label = truncate_label(&format!("{} · {}", span.service, span.name), 36);
        text(
            prims,
            MARGIN + row.depth as f64 * 14.0,
            *y + 12.0,
            11.0,
            palette.ink,
            false,
            false,
            label,
        );
        let bar_x = plot_x0 + span.start_unix_nano.saturating_sub(t0) as f64 / total * plot_w;
        let bar_w = ((span.duration_nanos() as f64) / total * plot_w).max(1.5);
        let fill = if highlight_errors && span.is_error {
            palette.error.to_string()
        } else {
            colors
                .get(&span.service)
                .cloned()
                .unwrap_or_else(|| palette.fallback.to_string())
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
                Some((palette.outline.to_string(), 1.5))
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
                fill: palette.outline.to_string(),
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
            palette.muted,
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
        palette.muted,
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
    palette: &Palette,
    step: &AgentStep,
    depth: usize,
    from_nano: i64,
    window_nano: i64,
    y: &mut f64,
) {
    // A pre-order walk on an explicit stack (INV-87), drawing
    // each step's row at its depth in visit order, the order
    // the recursive walk drew them.
    let mut stack: Vec<(&AgentStep, usize)> = vec![(step, depth)];
    while let Some((step, depth)) = stack.pop() {
        layout_agent_step_row(prims, palette, step, depth, from_nano, window_nano, y);
        if let AgentStep::Subagent(sub) = step {
            for child in sub.steps.iter().rev() {
                stack.push((child, depth + 1));
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn layout_agent_step_row(
    prims: &mut Vec<Prim>,
    palette: &Palette,
    step: &AgentStep,
    depth: usize,
    from_nano: i64,
    window_nano: i64,
    y: &mut f64,
) {
    let plot_x0 = 200.0;
    let plot_x1 = WIDTH - MARGIN;
    let x_of = |t: i64| {
        plot_x0 + (t.saturating_sub(from_nano) as f64 / window_nano as f64) * (plot_x1 - plot_x0)
    };
    let start = step.start_unix_nano();
    let end = step.end_unix_nano();
    let x1 = x_of(start);
    let x2 = x_of(end).max(x1 + 1.5);
    let label_x = MARGIN + depth as f64 * 14.0;
    let (label, fill) = match step {
        AgentStep::ModelCall(call) => (format!("model {}", call.model), palette.service[0]),
        AgentStep::ToolCall(tool) => (format!("tool {}", tool.name), palette.service[2]),
        AgentStep::Subagent(sub) => (format!("subagent {}", sub.agent_id), palette.service[1]),
    };
    text(
        prims,
        label_x,
        *y + 11.0,
        11.0,
        palette.ink,
        false,
        false,
        truncate_label(&label, ((plot_x0 - label_x - 8.0) / 6.0) as usize),
    );
    if let AgentStep::ToolCall(tool) = step
        && tool.wait_nanos > 0
    {
        // The permission wait in grey at the head of the bar, the
        // execution in the tool color behind it.
        let wait_end = x_of(start.saturating_add(tool.wait_nanos)).min(x2);
        prims.push(Prim::Rect {
            x: x1,
            y: *y,
            w: (wait_end - x1).max(0.0),
            h: 12.0,
            fill: palette.waiting.to_string(),
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
    let mut detail = fmt_duration(end.saturating_sub(start));
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
        palette.muted,
        false,
        false,
        detail.clone(),
    );
    if is_error {
        text(
            prims,
            x2 + 12.0 + text_width(&detail) as f64 * 6.05,
            *y + 11.0,
            11.0,
            palette.error,
            true,
            false,
            "· error".to_string(),
        );
    }
    *y += 18.0;
}

fn render_agent_state(
    scene: &Scene,
    data: &SceneData,
    panel: &Panel,
    prims: &mut Vec<Prim>,
    palette: &Palette,
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
            palette.ink,
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
            palette.ink,
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
                palette.error,
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
                palette.muted,
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
    palette: &Palette,
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
    let from_nano = scene.context.range.from_unix_ms.saturating_mul(1_000_000);
    let window_nano = (scene
        .context
        .range
        .to_unix_ms
        .saturating_sub(scene.context.range.from_unix_ms)
        .saturating_mul(1_000_000))
    .max(1);

    // Time axis, the same five ticks the waterfall uses.
    let plot_x0 = 200.0;
    let plot_x1 = WIDTH - MARGIN;
    let axis_y = *y + 4.0;
    prims.push(Prim::Line {
        x1: plot_x0,
        y1: axis_y,
        x2: plot_x1,
        y2: axis_y,
        stroke: palette.grid.to_string(),
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
            stroke: palette.grid.to_string(),
            width: 1.0,
        });
        prims.push(Prim::Text {
            x: tx,
            y: axis_y - 4.0,
            size: 10.0,
            fill: palette.muted.to_string(),
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
        total_in = total_in.saturating_add(turn.input_tokens);
        total_out = total_out.saturating_add(turn.output_tokens);
        total_cache = total_cache.saturating_add(turn.cache_read_tokens);
        total_cost = telorama_scene::add_costs(total_cost, turn.cost_usd);
        text(
            prims,
            MARGIN,
            *y + 12.0,
            12.0,
            palette.ink,
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
            layout_agent_step(prims, palette, step, 0, from_nano, window_nano, y);
        }
        *y += 6.0;
    }
    text(
        prims,
        MARGIN,
        *y + 12.0,
        11.0,
        palette.muted,
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
        palette.muted,
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
        text(
            prims,
            MARGIN,
            *y + 12.0,
            11.0,
            palette.muted,
            false,
            false,
            line,
        );
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
    // Layers come from the graph's condensation, so a cyclic
    // graph collapses its cycles instead of inflating the layer
    // count (INV-82). Acyclic layers are unchanged.
    let layer_owned = telorama_scene::service_layers(graph);
    let layer: BTreeMap<&str, usize> = layer_owned
        .iter()
        .map(|(name, layer)| (name.as_str(), *layer))
        .collect();
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
    palette: &Palette,
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
            palette.residual.to_string()
        } else {
            colors
                .get(&link.from_service)
                .cloned()
                .unwrap_or_else(|| palette.residual.to_string())
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
            palette.residual.to_string()
        } else {
            colors
                .get(&node.service)
                .cloned()
                .unwrap_or_else(|| palette.residual.to_string())
        };
        text(
            prims,
            x,
            ntop - 5.0,
            11.0,
            palette.ink,
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
            palette.muted,
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
    palette: &Palette,
    y: &mut f64,
) -> Result<(), RenderError> {
    let Panel::ServiceMap { data } = panel else {
        return Err(fail("render_service_map called on a non-service-map panel"));
    };
    let set_owned: Vec<Span> = dataset(scene, spans, data).into_iter().cloned().collect();
    if set_owned.is_empty() {
        return Err(fail("service map has no spans in its data set"));
    }
    let window_seconds = (scene
        .context
        .range
        .to_unix_ms
        .saturating_sub(scene.context.range.from_unix_ms) as f64
        / 1000.0)
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
    let box_w = (step - 10.0).clamp(8.0, 160.0);
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
            stroke: thresholds.color_for(ratio, palette.fallback),
            width: 1.0 + 3.0 * (edge.calls as f64 / max_calls as f64),
        });
        prims.push(Prim::Text {
            x: (fx + box_w + tx) / 2.0,
            y: (fy + ty) / 2.0 + box_h / 2.0 - 4.0,
            size: 10.0,
            fill: palette.muted.to_string(),
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
            .unwrap_or_else(|| palette.residual.to_string());
        prims.push(Prim::Rect {
            x: bx,
            y: by,
            w: box_w,
            h: box_h,
            fill: fill.clone(),
            opacity: None,
            stroke: None,
        });
        let text_fill = bar_text_fill(&fill, palette).to_string();
        prims.push(Prim::Text {
            x: bx + 8.0,
            y: by + 14.0,
            size: 11.0,
            fill: text_fill.clone(),
            bold: true,
            mono: false,
            anchor: "start",
            content: truncate_label(&node.service, (((box_w - 12.0) / 6.5) as usize).max(4)),
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
            palette.muted,
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
fn name_hash(name: &str, palette_len: usize) -> usize {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in name.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    (h % palette_len as u64) as usize
}

fn flame_depth(node: &FlameNode) -> usize {
    // Iterative, like the builder (INV-86): the depth verdict
    // must not overflow the stack on the deep trees it exists
    // to refuse.
    let mut deepest = 0usize;
    let mut stack: Vec<(&FlameNode, usize)> = vec![(node, 1)];
    while let Some((node, depth)) = stack.pop() {
        deepest = deepest.max(depth);
        stack.extend(node.children.iter().map(|child| (child, depth + 1)));
    }
    deepest
}

/// WCAG relative luminance of a hex color, or None when the
/// value is not a six-digit hex color.
fn relative_luminance(hex: &str) -> Option<f64> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let channel = |i: usize| {
        let c = u32::from_str_radix(&hex[i..i + 2], 16).ok()? as f64 / 255.0;
        Some(if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        })
    };
    Some(0.2126 * channel(0)? + 0.7152 * channel(2)? + 0.0722 * channel(4)?)
}

/// The WCAG contrast ratio between two hex colors. An
/// unparseable color ratios at one to one, so a caller
/// comparing candidates falls back to its second choice.
fn contrast_ratio(a: &str, b: &str) -> f64 {
    let (Some(la), Some(lb)) = (relative_luminance(a), relative_luminance(b)) else {
        return 1.0;
    };
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// Text color for a label drawn on a fill. Under the light
/// theme, white on dark fills and ink on light ones, by
/// perceived luminance. Under the dark theme, whichever of
/// the ground and the ink contrasts more with the fill, so
/// a label on the dark surface root of a flame graph takes
/// the ink while a label on a light service fill takes the
/// ground.
fn bar_text_fill(fill: &str, palette: &Palette) -> &'static str {
    if palette.dark {
        if contrast_ratio(fill, palette.background) >= contrast_ratio(fill, palette.ink) {
            return palette.background;
        }
        return palette.ink;
    }
    let hex = fill.trim_start_matches('#');
    if hex.len() != 6 {
        return palette.ink;
    }
    let channel = |i: usize| u32::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0) as f64;
    let luminance = 0.299 * channel(0) + 0.587 * channel(2) + 0.114 * channel(4);
    if luminance < 140.0 {
        "#FFFFFF"
    } else {
        palette.ink
    }
}

// A flame layout call is position, size, palette, and tree state: the
// argument list is the interface, so the count lint is allowed here.
#[allow(clippy::too_many_arguments)]
fn layout_flame(
    prims: &mut Vec<Prim>,
    palette: &Palette,
    node: &FlameNode,
    x: f64,
    w: f64,
    y: f64,
    total: u64,
    depth: usize,
) {
    let fill = if depth == 0 {
        palette.surface.to_string()
    } else {
        palette.service[name_hash(&node.name, palette.service.len())].to_string()
    };
    prims.push(Prim::Rect {
        x,
        y,
        w,
        h: 14.0,
        fill: fill.clone(),
        opacity: None,
        stroke: Some((palette.flame_stroke.to_string(), 0.5)),
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
            fill: bar_text_fill(&fill, palette).to_string(),
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
            layout_flame(
                prims,
                palette,
                child,
                child_x,
                child_w,
                y + 16.0,
                total,
                depth + 1,
            );
            child_x += child_w;
        }
    }
}

fn render_flame(
    scene: &Scene,
    data: &SceneData,
    panel: &Panel,
    prims: &mut Vec<Prim>,
    palette: &Palette,
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
    let (mut tree, note) = if let Some(trace_id) = entry.source.trace_id.as_deref() {
        let tree = flame_from_spans(data.spans, trace_id);
        if tree.children.is_empty() {
            if data.spans.iter().any(|s| s.trace_id == trace_id) {
                return Err(fail(format!(
                    "trace {trace_id} has spans but none can root a flame graph, because every span's parent chain cycles"
                )));
            }
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
        let mut tree = flame_tree(data.stacks);
        if tree.value != *sample_count {
            let message = format!(
                "declared sample count {sample_count} does not match the profile data total {}: a scene must not state a count its data does not carry",
                tree.value
            );
            dismantle_flame_tree(&mut tree);
            return Err(fail(message));
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
        let message = format!(
            "flame graph has {} samples, below its stated floor of {}",
            tree.value, min_samples
        );
        dismantle_flame_tree(&mut tree);
        return Err(fail(message));
    }
    let depth = flame_depth(&tree);
    if depth > 48 {
        let message =
            format!("flame tree is {depth} levels deep, beyond the 48-level render limit");
        dismantle_flame_tree(&mut tree);
        return Err(fail(message));
    }
    let plot_w = WIDTH - 2.0 * MARGIN;
    layout_flame(prims, palette, &tree, MARGIN, plot_w, *y, tree.value, 0);
    *y += depth as f64 * 16.0 + 6.0;
    text(
        prims,
        MARGIN,
        *y + 12.0,
        11.0,
        palette.muted,
        false,
        false,
        note,
    );
    *y += 22.0;
    dismantle_flame_tree(&mut tree);
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
/// The full step returns no opacity (solid). An empty cell is drawn by
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
    palette: &Palette,
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
    let window_nano = scene
        .context
        .range
        .to_unix_ms
        .saturating_sub(scene.context.range.from_unix_ms) as f64
        * 1_000_000.0;

    // Time axis, the same five ticks the waterfall uses.
    let axis_y = *y + 4.0;
    prims.push(Prim::Line {
        x1: plot_x0,
        y1: axis_y,
        x2: plot_x1,
        y2: axis_y,
        stroke: palette.grid.to_string(),
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
            stroke: palette.grid.to_string(),
            width: 1.0,
        });
        prims.push(Prim::Text {
            x: tx,
            y: axis_y - 4.0,
            size: 10.0,
            fill: palette.muted.to_string(),
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
            fill: palette.muted.to_string(),
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
                (palette.empty_cell.to_string(), None)
            } else {
                (
                    palette.heatmap_base.to_string(),
                    cell_shade(*count, grid.max_count),
                )
            };
            prims.push(Prim::Rect {
                x: plot_x0 + c as f64 * col_w + 0.5,
                y: *y,
                w: col_w - 1.0,
                h: row_h - 2.0,
                fill,
                opacity,
                stroke: if is_exemplar {
                    Some((palette.error.to_string(), 1.5))
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
            palette.muted,
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
            palette.muted,
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
            palette.muted,
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
            palette.muted,
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
            palette.muted,
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
    palette: &Palette,
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
    text(
        prims,
        MARGIN,
        *y + 12.0,
        12.0,
        palette.muted,
        false,
        false,
        label,
    );
    let value_fill = match raw {
        Some(v) if !thresholds.is_empty() => thresholds.color_for(v, palette.ink),
        _ => palette.ink.to_string(),
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
        palette.muted,
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
    palette: &Palette,
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
        text(prims, x, *y + 12.0, 11.0, palette.muted, true, false, name);
    }
    *y += 18.0;
    for row in &stats {
        text(
            prims,
            MARGIN,
            *y + 12.0,
            12.0,
            palette.ink,
            false,
            false,
            row.service.clone(),
        );
        text(
            prims,
            300.0,
            *y + 12.0,
            12.0,
            palette.ink,
            false,
            true,
            row.calls.to_string(),
        );
        text(
            prims,
            380.0,
            *y + 12.0,
            12.0,
            palette.ink,
            false,
            true,
            fmt_num(row.rate_per_sec),
        );
        let err_fill = verdicts.color_for(row.error_ratio, palette.ink);
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
            palette.ink,
            false,
            true,
            fmt_duration(row.p50_nanos),
        );
        text(
            prims,
            680.0,
            *y + 12.0,
            12.0,
            palette.ink,
            false,
            true,
            fmt_duration(row.p95_nanos),
        );
        text(
            prims,
            790.0,
            *y + 12.0,
            12.0,
            palette.ink,
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
        palette.muted,
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
    /// The span set trace panels read, and the input every
    /// derivation runs over.
    pub spans: &'a [Span],
    /// Folded stack samples for flame panels. Empty when the
    /// scene's flame panels derive from spans instead.
    pub stacks: &'a [StackSample],
    /// The agent run agent_run panels read. `None` when the
    /// scene carries no session data.
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
    standalone_html_themed(title, svg, &LIGHT_PALETTE)
}

/// Wrap a rendered SVG in a standalone HTML document painted with
/// the given palette, so a dark scene exports as a dark page.
pub fn standalone_html_themed(title: &str, svg: &str, palette: &Palette) -> String {
    format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<title>{}</title>\n<style>body{{margin:0;background:{};color:{};font-family:system-ui,sans-serif}}main{{max-width:1000px;margin:0 auto;padding:24px}}svg{{width:100%;height:auto}}</style>\n</head>\n<body>\n<main>\n{}</main>\n</body>\n</html>\n",
        escape_xml(title),
        palette.background,
        palette.ink,
        svg
    )
}

/// Render a scene to a standalone HTML document carrying the SVG.
pub fn render_scene_html(scene: &Scene, data: &SceneData) -> Result<String, RenderError> {
    let svg = render_scene_with_data(scene, data)?;
    Ok(standalone_html_themed(
        &scene.title,
        &svg,
        &palette_for(scene.theme),
    ))
}

/// Render a scene over a full data set to a deterministic SVG document.
///
/// Panels render in spec order under the headline. The provenance
/// caption at the foot of the scene prints the window, every data
/// source with its query or trace id and sampling state, and the
/// generating question, so the picture carries its own evidence.
pub fn render_scene_with_data(scene: &Scene, data: &SceneData) -> Result<String, RenderError> {
    let spans = data.spans;
    let palette = palette_for(scene.theme);
    let colors = service_colors_for(spans, &palette);
    let mut prims: Vec<Prim> = Vec::new();
    let mut y: f64 = 30.0;

    text(
        &mut prims,
        MARGIN,
        y,
        20.0,
        palette.ink,
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
        palette.muted,
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
            palette.ink,
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
                palette.ink,
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
        text(
            &mut prims,
            MARGIN,
            y,
            14.0,
            palette.ink,
            true,
            false,
            heading,
        );
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
                &palette,
                &mut y,
            )?,
            Panel::Stat { .. } => render_stat(scene, spans, panel, &mut prims, &palette, &mut y)?,
            Panel::RedRow { .. } => {
                render_red_row(scene, spans, panel, &mut prims, &palette, &mut y)?
            }
            Panel::Heatmap { .. } => {
                render_heatmap(scene, spans, panel, &mut prims, &palette, &mut y)?
            }
            Panel::Flame { .. } => render_flame(scene, data, panel, &mut prims, &palette, &mut y)?,
            Panel::ServiceMap { .. } => {
                render_service_map(scene, spans, panel, &colors, &mut prims, &palette, &mut y)?
            }
            Panel::Sankey { .. } => {
                render_sankey(scene, spans, panel, &colors, &mut prims, &palette, &mut y)?
            }
            Panel::AgentRun { .. } => {
                render_agent_run(scene, data, panel, &mut prims, &palette, &mut y)?
            }
            Panel::AgentState { .. } => {
                render_agent_state(scene, data, panel, &mut prims, &palette, &mut y)?
            }
        }
    }

    // Provenance caption.
    y += 10.0;
    prims.push(Prim::Line {
        x1: MARGIN,
        y1: y,
        x2: WIDTH - MARGIN,
        y2: y,
        stroke: palette.grid.to_string(),
        width: 1.0,
    });
    y += 16.0;
    text(
        &mut prims,
        MARGIN,
        y,
        11.0,
        palette.muted,
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
            palette.muted,
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
        palette.muted,
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
        background: palette.background.to_string(),
        prims,
    };
    Ok(list.to_svg())
}

#[cfg(test)]
mod tests {
    use super::{char_width, text_width, truncate_label};

    #[test]
    fn width_classes_and_range_edges() {
        // One column for Latin, and for the ambiguous class the
        // corpus itself depends on (INV-92).
        assert_eq!(char_width('a'), 1);
        assert_eq!(char_width('·'), 1);
        assert_eq!(char_width('…'), 1);
        assert_eq!(char_width('µ'), 1);
        // Two columns for wide and fullwidth characters, at the
        // edges of the named ranges.
        assert_eq!(char_width('支'), 2);
        assert_eq!(char_width('\u{1100}'), 2);
        assert_eq!(char_width('\u{115F}'), 2);
        assert_eq!(char_width('\u{303F}'), 1);
        assert_eq!(char_width('\u{FF21}'), 2);
        assert_eq!(char_width('\u{1F600}'), 2);
        // Zero for combining marks and zero width characters,
        // at the edges of the named ranges.
        assert_eq!(char_width('\u{0300}'), 0);
        assert_eq!(char_width('\u{036F}'), 0);
        assert_eq!(char_width('\u{0370}'), 1);
        assert_eq!(char_width('\u{064B}'), 0);
        assert_eq!(char_width('\u{200D}'), 0);
        assert_eq!(char_width('\u{FE0F}'), 0);
    }

    #[test]
    fn text_width_sums_by_class() {
        assert_eq!(text_width("checkout"), 8);
        assert_eq!(text_width("支払い"), 6);
        assert_eq!(text_width("خدمة"), 4);
        assert_eq!(text_width("cafe\u{301}"), 4);
        assert_eq!(text_width("a支b"), 4);
        assert_eq!(text_width(""), 0);
    }

    #[test]
    fn truncation_matches_the_count_model_for_latin() {
        assert_eq!(truncate_label("checkout", 36), "checkout");
        assert_eq!(truncate_label("abcdefghij", 5), "abcd…");
        assert_eq!(truncate_label("abc", 0), "");
        assert_eq!(truncate_label("abc", 1), "…");
    }

    #[test]
    fn truncation_counts_wide_characters_twice() {
        // Five columns hold two wide characters and the
        // ellipsis, not four characters and the ellipsis.
        assert_eq!(truncate_label("支払いサービス", 5), "支払…");
        // A wide character that straddles the budget is left
        // out whole rather than half counted.
        assert_eq!(truncate_label("ab支", 3), "ab…");
    }

    #[test]
    fn truncation_never_splits_a_combining_sequence() {
        // The combining acute rides its base character into
        // the kept prefix even at the exact budget boundary.
        assert_eq!(truncate_label("cafe\u{301}xyz", 5), "cafe\u{301}…");
    }
}
