use telorama_scene::{parse_scene, validate_scene_json};
use serde_json::{Value, json};

fn valid_scene() -> Value {
    let text = include_str!("../../../tests/fixtures/scenes/checkout_scene.json");
    serde_json::from_str(text).expect("fixture scene must be JSON")
}

fn messages(errors: &[telorama_scene::ValidationError]) -> String {
    errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
}

#[test]
fn valid_fixture_scene_passes() {
    let scene = valid_scene();
    let errors = validate_scene_json(&scene);
    assert!(errors.is_empty(), "unexpected errors: {}", messages(&errors));
    let text = include_str!("../../../tests/fixtures/scenes/checkout_scene.json");
    let parsed = parse_scene(text).expect("fixture scene must parse");
    assert_eq!(parsed.panels.len(), 3);
    assert_eq!(parsed.data.len(), 1);
}

#[test]
fn average_duration_statistic_is_rejected() {
    let mut scene = valid_scene();
    scene["panels"][1]["stat"] = json!("average");
    let errors = validate_scene_json(&scene);
    assert!(
        errors.iter().any(|e| e.message.contains("percentiles, never an average")),
        "expected the honesty rule to fire, got: {}",
        messages(&errors)
    );
}

#[test]
fn missing_sampling_state_is_rejected() {
    let mut scene = valid_scene();
    scene["data"][0].as_object_mut().unwrap().remove("sampled");
    let errors = validate_scene_json(&scene);
    assert!(errors.iter().any(|e| e.path == "$.data[0].sampled"));
}

#[test]
fn panel_reading_undeclared_data_is_rejected() {
    let mut scene = valid_scene();
    scene["panels"][0]["data"] = json!("nope");
    let errors = validate_scene_json(&scene);
    assert!(errors.iter().any(|e| e.message.contains("undeclared data id")));
}

#[test]
fn duplicate_data_id_is_rejected() {
    let mut scene = valid_scene();
    let dup = scene["data"][0].clone();
    scene["data"].as_array_mut().unwrap().push(dup);
    let errors = validate_scene_json(&scene);
    assert!(errors.iter().any(|e| e.message.contains("duplicate data id")));
}

#[test]
fn sankey_without_residual_is_rejected_and_with_residual_passes() {
    let mut scene = valid_scene();
    scene["panels"].as_array_mut().unwrap().push(json!({
        "type": "sankey", "data": "trace"
    }));
    let errors = validate_scene_json(&scene);
    assert!(errors.iter().any(|e| e.message.contains("residual")));

    let mut scene = valid_scene();
    scene["panels"].as_array_mut().unwrap().push(json!({
        "type": "sankey", "data": "trace", "residual": "other"
    }));
    let errors = validate_scene_json(&scene);
    assert!(errors.is_empty(), "unexpected errors: {}", messages(&errors));
}

#[test]
fn flame_below_sample_floor_is_rejected() {
    let mut scene = valid_scene();
    scene["panels"].as_array_mut().unwrap().push(json!({
        "type": "flame", "data": "trace", "sample_type": "cpu",
        "min_samples": 100, "sample_count": 10
    }));
    let errors = validate_scene_json(&scene);
    assert!(errors.iter().any(|e| e.message.contains("below its stated floor")));
}

#[test]
fn heatmap_needs_real_bucket_edges() {
    let mut scene = valid_scene();
    scene["panels"].as_array_mut().unwrap().push(json!({
        "type": "heatmap", "data": "trace", "unit": "duration"
    }));
    let errors = validate_scene_json(&scene);
    assert!(errors.iter().any(|e| e.message.contains("bucket edges")));

    let mut scene = valid_scene();
    scene["data"][0]["source"]["buckets"] = json!([0.001, 0.01, 0.1, 1.0]);
    scene["panels"].as_array_mut().unwrap().push(json!({
        "type": "heatmap", "data": "trace", "unit": "duration"
    }));
    let errors = validate_scene_json(&scene);
    assert!(errors.is_empty(), "unexpected errors: {}", messages(&errors));
}

#[test]
fn narrative_anchor_must_exist() {
    let mut scene = valid_scene();
    scene["narrative"][0]["panel"] = json!(99);
    let errors = validate_scene_json(&scene);
    assert!(errors.iter().any(|e| e.path == "$.narrative[0].panel"));
}

#[test]
fn unsupported_version_and_backward_range_are_rejected() {
    let mut scene = valid_scene();
    scene["scene"] = json!("9.9");
    let errors = validate_scene_json(&scene);
    assert!(errors.iter().any(|e| e.message.contains("unsupported scene version")));

    let mut scene = valid_scene();
    scene["context"]["range"]["to_unix_ms"] = json!(1);
    let errors = validate_scene_json(&scene);
    assert!(errors.iter().any(|e| e.message.contains("must run forward")));
}

#[test]
fn source_without_query_or_trace_is_rejected() {
    let mut scene = valid_scene();
    scene["data"][0]["source"].as_object_mut().unwrap().remove("trace_id");
    let errors = validate_scene_json(&scene);
    assert!(errors.iter().any(|e| e.message.contains("not reproducible")));
}
