use std::path::PathBuf;
use telorama_scene::Span;

fn workspace_file(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

fn load(scene: &str, traces: &str) -> (telorama_scene::Scene, Vec<Span>) {
    let scene =
        telorama_scene::parse_scene(&workspace_file(&format!("tests/fixtures/scenes/{scene}")))
            .expect("fixture scene must validate");
    let spans =
        telorama_ingest::parse_otlp_traces(&workspace_file(&format!("tests/fixtures/{traces}")))
            .expect("fixture traces must parse");
    (scene, spans)
}

fn golden(name: &str, scene: &str, traces: &str) {
    let (scene_obj, spans) = load(scene, traces);
    let svg = telorama_svg::render_scene(&scene_obj, &spans).expect("scene must render");
    let golden_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(format!("tests/golden/{name}"));
    if std::env::var_os("UPDATE_GOLDENS").is_some() {
        std::fs::write(&golden_path, &svg).expect("golden must be writable");
        return;
    }
    let expected = std::fs::read_to_string(&golden_path)
        .unwrap_or_else(|e| panic!("cannot read golden {}: {e}", golden_path.display()));
    assert_eq!(
        svg, expected,
        "rendered SVG differs from the golden file; if the change is intended, rerun with UPDATE_GOLDENS=1 and review the diff"
    );
}

#[test]
fn golden_checkout_scene() {
    golden(
        "checkout_scene.svg",
        "checkout_scene.json",
        "otel-demo-checkout.json",
    );
}

#[test]
fn golden_api_scene() {
    golden("api_scene.svg", "api_scene.json", "single-service-api.json");
}

#[test]
fn rendering_is_byte_identical_across_runs_and_input_orders() {
    let (scene, spans) = load("checkout_scene.json", "otel-demo-checkout.json");
    let first = telorama_svg::render_scene(&scene, &spans).unwrap();
    let second = telorama_svg::render_scene(&scene, &spans).unwrap();
    assert_eq!(first, second);
    let mut reversed = spans.clone();
    reversed.reverse();
    let third = telorama_svg::render_scene(&scene, &reversed).unwrap();
    assert_eq!(first, third, "input order must not leak into the render");
}

#[test]
fn unsupported_panel_type_is_a_specific_error() {
    let (mut scene, spans) = load("api_scene.json", "single-service-api.json");
    scene.panels.push(telorama_scene::Panel::Sankey {
        data: "trace".to_string(),
        residual: "other".to_string(),
    });
    let err = telorama_svg::render_scene(&scene, &spans).unwrap_err();
    assert!(
        err.message.contains("no renderer yet"),
        "got: {}",
        err.message
    );
}

#[test]
fn formatting_is_canonical() {
    assert_eq!(telorama_svg::fmt_num(960.0), "960");
    assert_eq!(telorama_svg::fmt_num(12.5), "12.5");
    assert_eq!(telorama_svg::fmt_num(0.25), "0.25");
    assert_eq!(telorama_svg::fmt_duration(500), "500 ns");
    assert_eq!(telorama_svg::fmt_duration(1_500), "1.5 µs");
    assert_eq!(telorama_svg::fmt_duration(420_000_000), "420 ms");
    assert_eq!(telorama_svg::fmt_duration(2_500_000_000), "2.5 s");
    assert_eq!(
        telorama_svg::fmt_time_ms(1_794_839_400_000),
        "2026-11-16T14:30:00Z"
    );
    assert_eq!(telorama_svg::fmt_time_ms(0), "1970-01-01T00:00:00Z");
}
