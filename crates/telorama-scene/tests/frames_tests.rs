use telorama_scene::{
    session_frames, AgentRun, AgentStep, AgentTurn, Coverage, ModelCall, Pricing,
};

fn call(model: &str, cost: f64) -> AgentStep {
    AgentStep::ModelCall(ModelCall {
        model: model.to_string(),
        start_unix_nano: 0,
        end_unix_nano: 1_000_000_000,
        ttft_nanos: None,
        attempts: 1,
        is_error: false,
        input_tokens: 100,
        output_tokens: 10,
        cache_read_tokens: 0,
        cache_creation_tokens: 0,
        cost_usd: cost,
    })
}

fn two_turn_run() -> AgentRun {
    AgentRun {
        session_id: "sess-frames".to_string(),
        dialect: "codex".to_string(),
        turns: vec![
            AgentTurn {
                prompt_id: "t1".to_string(),
                start_unix_nano: 0,
                end_unix_nano: 40_000_000_000,
                steps: vec![call("gpt-5.1", 0.50)],
                input_tokens: 100,
                output_tokens: 10,
                cache_read_tokens: 0,
                cache_creation_tokens: 0,
                cost_usd: 0.50,
                coverage: Coverage {
                    log_events: 5,
                    spans_seen: 0,
                    spans_merged: 0,
                },
            },
            AgentTurn {
                prompt_id: "t2".to_string(),
                start_unix_nano: 40_000_000_000,
                end_unix_nano: 100_000_000_000,
                steps: vec![call("gpt-unknown-9", 0.0)],
                input_tokens: 60,
                output_tokens: 6,
                cache_read_tokens: 0,
                cache_creation_tokens: 0,
                cost_usd: 0.0,
                coverage: Coverage {
                    log_events: 7,
                    spans_seen: 0,
                    spans_merged: 0,
                },
            },
        ],
        coverage: Coverage {
            log_events: 12,
            spans_seen: 3,
            spans_merged: 0,
        },
        pricing: Some(Pricing {
            table_as_of: "2026-11-16".to_string(),
            priced_models: vec!["gpt-5.1".to_string()],
            unpriced_models: vec!["gpt-unknown-9".to_string()],
        }),
    }
}

#[test]
fn frames_cover_prefixes_in_order() {
    let frames = session_frames(&two_turn_run(), "test");
    assert_eq!(frames.len(), 2);
    assert_eq!(frames[0].turns, 1);
    assert_eq!(frames[1].turns, 2);
    assert_eq!(frames[0].run.turns.len(), 1);
    assert_eq!(frames[1].run.turns.len(), 2);
    assert_eq!(frames[0].run.coverage.log_events, 5);
    assert_eq!(frames[1].run.coverage.log_events, 12);
    // The supplied span count names the export and rides every frame.
    assert_eq!(frames[0].run.coverage.spans_seen, 3);
    assert!(frames[0].scene.headline.contains("After turn 1 of 2"));
    assert!(frames[1].scene.headline.contains("$0.5000 so far"));
}

#[test]
fn frame_pricing_names_only_prefix_models() {
    let frames = session_frames(&two_turn_run(), "test");
    let first = frames[0].run.pricing.as_ref().expect("pricing");
    assert_eq!(first.priced_models, vec!["gpt-5.1".to_string()]);
    assert!(
        first.unpriced_models.is_empty(),
        "turn 2's unpriced model must not leak into frame 1"
    );
    let second = frames[1].run.pricing.as_ref().expect("pricing");
    assert_eq!(second.unpriced_models, vec!["gpt-unknown-9".to_string()]);
}

#[test]
fn every_frame_scene_validates_and_frames_are_deterministic() {
    let run = two_turn_run();
    let frames = session_frames(&run, "test");
    for frame in &frames {
        let text = serde_json::to_string(&frame.scene).expect("scene serializes");
        telorama_scene::parse_scene(&text).expect("every frame scene must validate");
    }
    assert_eq!(frames, session_frames(&run, "test"));
    let cost = frames[1]
        .findings
        .iter()
        .find(|f| f.kind == "cost")
        .expect("the full session carries a cost finding");
    assert!(cost.statement.contains("$0.5000"), "{}", cost.statement);
}
