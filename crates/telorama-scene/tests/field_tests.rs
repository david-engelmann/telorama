//! The field system: one formatting language for every value, and
//! thresholds as the shared verdict channel.

use telorama_scene::{
    ThresholdStep, Thresholds, Unit, format_bytes, format_duration_nanos, format_ratio,
    format_value,
};

#[test]
fn format_value_duration_scales_by_magnitude() {
    assert_eq!(format_value(Unit::Duration, 500.0), "500 ns");
    assert_eq!(format_value(Unit::Duration, 1_500.0), "1.5 µs");
    assert_eq!(format_value(Unit::Duration, 420_000_000.0), "420 ms");
    assert_eq!(format_value(Unit::Duration, 2_500_000_000.0), "2.5 s");
}

#[test]
fn format_value_ratio_is_a_one_decimal_percent() {
    assert_eq!(format_value(Unit::Ratio, 0.0), "0.0%");
    assert_eq!(format_value(Unit::Ratio, 0.125), "12.5%");
    assert_eq!(format_value(Unit::Ratio, 1.0), "100.0%");
    assert_eq!(format_ratio(0.05), "5.0%");
}

#[test]
fn format_value_count_bytes_and_rate() {
    assert_eq!(format_value(Unit::Count, 3.0), "3");
    assert_eq!(format_value(Unit::Bytes, 500.0), "500 B");
    assert_eq!(format_value(Unit::Bytes, 2_048.0), "2 KiB");
    assert_eq!(format_value(Unit::RequestsPerSecond, 1.5), "1.5");
    assert_eq!(format_duration_nanos(420_000_000), "420 ms");
    assert_eq!(format_bytes(3_145_728), "3 MiB");
}

#[test]
fn thresholds_pick_the_highest_step_at_or_below_the_value() {
    let thresholds = Thresholds::new(vec![
        ThresholdStep {
            above: 0.5,
            color: "#E69F00".to_string(),
        },
        ThresholdStep {
            above: 0.8,
            color: "#D55E00".to_string(),
        },
    ]);
    assert_eq!(thresholds.color_for(0.49, "#1A1A1A"), "#1A1A1A");
    assert_eq!(thresholds.color_for(0.5, "#1A1A1A"), "#E69F00");
    assert_eq!(thresholds.color_for(0.79, "#1A1A1A"), "#E69F00");
    assert_eq!(thresholds.color_for(0.8, "#1A1A1A"), "#D55E00");
    assert_eq!(thresholds.color_for(2.0, "#1A1A1A"), "#D55E00");
}

#[test]
fn error_ratio_default_matches_the_m1_rule() {
    let thresholds = Thresholds::error_ratio();
    assert_eq!(thresholds.color_for(0.049, "#1A1A1A"), "#1A1A1A");
    assert_eq!(thresholds.color_for(0.05, "#1A1A1A"), "#D55E00");
    assert_eq!(thresholds.color_for(1.0, "#1A1A1A"), "#D55E00");
}
