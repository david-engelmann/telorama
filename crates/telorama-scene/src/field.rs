//! The field system: one formatting language for every value a scene
//! shows, and thresholds as the shared verdict channel.
//!
//! The quality bar is Grafana's field-configuration layer. Data and
//! display stay separate: a field carries its unit, and one formatter
//! renders a value the same way in an axis, a stat, a table cell, and a
//! caption, so the surfaces of a scene never disagree about what a
//! number says. Thresholds are stepped mappings from value to color;
//! the same step definition colors a stat value, a RED error cell, and
//! a heatmap cell, so color is one verdict language across panels.

use crate::spec::Unit;
use serde::{Deserialize, Serialize};

/// One threshold step: a value at or above `above` takes `color`, until
/// a higher step wins. Steps sit in strictly increasing order of
/// `above`; the validator enforces that on specs, and
/// [`Thresholds::color_for`] relies on it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThresholdStep {
    pub above: f64,
    pub color: String,
}

/// The threshold steps for one field. In the spec this is a bare array
/// of steps, so a panel writes `"thresholds": [{"above": 0.05,
/// "color": "#D55E00"}]`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Thresholds {
    pub steps: Vec<ThresholdStep>,
}

impl Thresholds {
    pub fn new(steps: Vec<ThresholdStep>) -> Self {
        Self { steps }
    }

    /// The standing error-ratio verdict from M1: a ratio at or above
    /// five percent is vermillion. A red_row panel with no thresholds
    /// of its own renders with exactly this rule.
    pub fn error_ratio() -> Self {
        Self::new(vec![ThresholdStep {
            above: 0.05,
            color: "#D55E00".to_string(),
        }])
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// The color a value earns: the highest step at or below the value,
    /// or `base` when no step applies.
    pub fn color_for(&self, value: f64, base: &str) -> String {
        let mut color = base.to_string();
        for step in &self.steps {
            if value >= step.above {
                color = step.color.clone();
            }
        }
        color
    }
}

/// Canonical number formatting: two decimal places, trailing zeros
/// trimmed. One function, one rounding rule, everywhere a plain number
/// is printed.
pub fn format_number(value: f64) -> String {
    let raw = format!("{value:.2}");
    if raw.contains('.') {
        raw.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        raw
    }
}

/// Format a nanosecond duration with automatic magnitude scaling.
pub fn format_duration_nanos(nanos: i64) -> String {
    let ns = nanos.max(0) as f64;
    if ns < 1_000.0 {
        format!("{} ns", nanos.max(0))
    } else if ns < 1_000_000.0 {
        format!("{} µs", format_number(ns / 1_000.0))
    } else if ns < 1_000_000_000.0 {
        format!("{} ms", format_number(ns / 1_000_000.0))
    } else {
        format!("{} s", format_number(ns / 1_000_000_000.0))
    }
}

/// Format a byte count with automatic magnitude scaling.
pub fn format_bytes(bytes: i64) -> String {
    let b = bytes.max(0) as f64;
    if b < 1024.0 {
        format!("{} B", bytes.max(0))
    } else if b < 1024.0 * 1024.0 {
        format!("{} KiB", format_number(b / 1024.0))
    } else {
        format!("{} MiB", format_number(b / (1024.0 * 1024.0)))
    }
}

/// Format a ratio as a percentage with one decimal: 0.05 is "5.0%".
pub fn format_ratio(ratio: f64) -> String {
    format!("{:.1}%", ratio * 100.0)
}

/// Format a value in its unit. The value's scale follows the unit:
/// durations are nanoseconds, ratios are fractions, bytes are bytes,
/// counts and rates are plain numbers. Threshold `above` values use
/// this same scale, so a threshold and the value it judges are always
/// in the same units.
pub fn format_value(unit: Unit, value: f64) -> String {
    match unit {
        Unit::Duration => format_duration_nanos(value as i64),
        Unit::Bytes => format_bytes(value as i64),
        Unit::Ratio => format_ratio(value),
        Unit::Count | Unit::RequestsPerSecond => format_number(value),
    }
}
