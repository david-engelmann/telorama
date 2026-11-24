//! The viewer record test, the freshness gate for the verdict
//! viewer page (INV-91). The record under web/data is assembled
//! here from the committed fixtures through the same library paths
//! the tools use, the incident composition, the investigation, the
//! frame builder, and the renderer. Run normally the test asserts
//! the committed record equals that assembly byte for byte, so the
//! page cannot drift from the product it shows. Run with
//! UPDATE_RECORD=1 on a machine that can write the tree, it writes
//! the record instead. The pattern is the gallery test's, carried
//! to the viewer surface.

use serde::Serialize;
use std::path::PathBuf;

const GENERATED_BY: &str = "telorama-viewer";

fn workspace_file(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

fn repo_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel)
}

/// One finding's anchor in its scene, the panel the finding is
/// about. Findings carry no panel field, so the generator derives
/// the anchor from the finding kind and the scene's panel list and
/// the record carries it as data. The page marks by this reference
/// and derives nothing itself.
#[derive(Serialize)]
struct Anchor {
    rank: usize,
    panel: usize,
}

#[derive(Serialize)]
struct Summary {
    title: String,
    headline: String,
    generated_by: String,
    question: String,
    from_unix_ms: i64,
    to_unix_ms: i64,
    panels: Vec<String>,
}

#[derive(Serialize)]
struct IncidentRecord {
    scene: telorama_scene::Scene,
    svg: String,
    svg_fnv1a64: String,
    investigation: Vec<telorama_scene::Finding>,
    anchors: Vec<Anchor>,
    summary: Summary,
}

#[derive(Serialize)]
struct FrameRecord {
    frame: usize,
    turns: usize,
    scene: telorama_scene::Scene,
    findings: Vec<telorama_scene::Finding>,
    anchors: Vec<Anchor>,
    svg: String,
    svg_fnv1a64: String,
}

#[derive(Serialize)]
struct SessionRecord {
    session_id: String,
    dialect: String,
    frames: Vec<FrameRecord>,
}

#[derive(Serialize)]
struct RecordMeta {
    generator: String,
    engine_version: String,
    scene_version: String,
    fixtures: Vec<String>,
}

#[derive(Serialize)]
struct Record {
    record: RecordMeta,
    incident: IncidentRecord,
    session: SessionRecord,
}

/// The panel a finding kind is about, by the panel that draws its
/// subject. Errors are read off the error rate row, latency off the
/// waterfall that carries the exemplar trace, and the session kinds
/// off the agent run panel. A kind with no matching panel in the
/// scene gets no anchor.
fn anchor_panel(scene: &telorama_scene::Scene, kind: &str) -> Option<usize> {
    let wanted = match kind {
        "errors" => "red",
        "latency" => "waterfall",
        "wait" | "cost" => "agent",
        _ => return None,
    };
    scene.panels.iter().position(|panel| {
        matches!(
            (wanted, panel),
            ("red", telorama_scene::Panel::RedRow { .. })
                | ("waterfall", telorama_scene::Panel::Waterfall { .. })
                | ("agent", telorama_scene::Panel::AgentRun { .. })
        )
    })
}

fn anchors_for(scene: &telorama_scene::Scene, findings: &[telorama_scene::Finding]) -> Vec<Anchor> {
    findings
        .iter()
        .filter_map(|finding| {
            anchor_panel(scene, &finding.kind).map(|panel| Anchor {
                rank: finding.rank,
                panel,
            })
        })
        .collect()
}

fn panel_type_names(scene: &telorama_scene::Scene) -> Vec<String> {
    scene
        .panels
        .iter()
        .map(|panel| {
            serde_json::to_value(panel)
                .ok()
                .and_then(|value| value.get("type").and_then(|t| t.as_str()).map(String::from))
                .unwrap_or_else(|| "unknown".to_string())
        })
        .collect()
}

fn assemble_record() -> Record {
    let spans = telorama_ingest::parse_otlp_traces(&workspace_file(
        "tests/fixtures/otel-demo-checkout.json",
    ))
    .expect("the checkout fixture must parse");
    let scene = telorama_scene::incident_scene(&spans, None, GENERATED_BY)
        .expect("the incident scene must compose");
    let window = scene.context.range.seconds();
    let findings = telorama_scene::investigate(&spans, None, window);
    let svg = telorama_svg::render_scene_with_data(
        &scene,
        &telorama_svg::SceneData {
            spans: &spans,
            stacks: &[],
            agent_run: None,
            agent_state: &[],
            findings: &findings,
        },
    )
    .expect("the incident scene must render");
    let incident = IncidentRecord {
        anchors: anchors_for(&scene, &findings),
        summary: Summary {
            title: scene.title.clone(),
            headline: scene.headline.clone(),
            generated_by: scene.context.provenance.generated_by.clone(),
            question: scene.context.provenance.question.clone(),
            from_unix_ms: scene.context.range.from_unix_ms,
            to_unix_ms: scene.context.range.to_unix_ms,
            panels: panel_type_names(&scene),
        },
        svg_fnv1a64: telorama_scene::content_hash(svg.as_bytes()),
        svg,
        investigation: findings,
        scene,
    };

    let logs = workspace_file("tests/fixtures/agents/claude-code-session-logs.json");
    let session_spans = telorama_ingest::parse_otlp_traces(&workspace_file(
        "tests/fixtures/agents/claude-code-session-spans.json",
    ))
    .expect("the session span fixture must parse");
    let run = telorama_ingest::parse_claude_code_session(&logs, &session_spans)
        .expect("the session fixture must adapt");
    let frames = telorama_scene::session_frames(&run, GENERATED_BY)
        .into_iter()
        .map(|frame| {
            let svg = telorama_svg::render_scene_with_data(
                &frame.scene,
                &telorama_svg::SceneData {
                    spans: &[],
                    stacks: &[],
                    agent_run: Some(&frame.run),
                    agent_state: &[],
                    findings: &frame.findings,
                },
            )
            .expect("each frame must render");
            FrameRecord {
                frame: frame.index + 1,
                turns: frame.turns,
                anchors: anchors_for(&frame.scene, &frame.findings),
                svg_fnv1a64: telorama_scene::content_hash(svg.as_bytes()),
                svg,
                findings: frame.findings,
                scene: frame.scene,
            }
        })
        .collect();
    let session = SessionRecord {
        session_id: run.session_id.clone(),
        dialect: run.dialect.clone(),
        frames,
    };

    Record {
        record: RecordMeta {
            generator: GENERATED_BY.to_string(),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            scene_version: telorama_scene::SCENE_VERSION.to_string(),
            fixtures: vec![
                "tests/fixtures/otel-demo-checkout.json".to_string(),
                "tests/fixtures/agents/claude-code-session-logs.json".to_string(),
                "tests/fixtures/agents/claude-code-session-spans.json".to_string(),
            ],
        },
        incident,
        session,
    }
}

fn assemble_record_json() -> String {
    let record = assemble_record();
    let mut text = serde_json::to_string(&record).expect("the record must serialize");
    text.push('\n');
    text
}

#[test]
fn viewer_record_matches_the_committed_record() {
    let assembled = assemble_record_json();
    let record_path = repo_path("web/data/record.json");
    if std::env::var_os("UPDATE_RECORD").is_some() {
        std::fs::create_dir_all(record_path.parent().expect("the record has a parent"))
            .expect("the record directory must be creatable");
        std::fs::write(&record_path, &assembled).expect("the record must be writable");
        return;
    }
    let committed = std::fs::read_to_string(&record_path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", record_path.display()));
    assert_eq!(
        assembled, committed,
        "the viewer record differs from the current tool output. Rerun with UPDATE_RECORD=1 and review the diff"
    );
}

#[test]
fn viewer_record_covers_every_finding_kind_and_the_quiet_panel() {
    let record = assemble_record();
    let incident_kinds: Vec<&str> = record
        .incident
        .investigation
        .iter()
        .map(|finding| finding.kind.as_str())
        .collect();
    assert_eq!(incident_kinds, ["errors", "latency"]);
    let anchor_of = |rank: usize| {
        record
            .incident
            .anchors
            .iter()
            .find(|anchor| anchor.rank == rank)
            .map(|anchor| anchor.panel)
    };
    assert_eq!(
        anchor_of(1),
        Some(1),
        "the errors finding anchors to the red row"
    );
    assert_eq!(
        anchor_of(2),
        Some(2),
        "the latency finding anchors to the waterfall"
    );
    assert!(
        record
            .incident
            .anchors
            .iter()
            .all(|anchor| anchor.panel != 3),
        "the heatmap panel carries no finding and stays the quiet panel"
    );
    assert_eq!(
        telorama_scene::content_hash(record.incident.svg.as_bytes()),
        record.incident.svg_fnv1a64,
        "the incident hash must be the hash of the incident svg"
    );

    assert_eq!(record.session.session_id, "sess-2026-1116-demo");
    assert_eq!(record.session.frames.len(), 2);
    for frame in &record.session.frames {
        let kinds: Vec<&str> = frame.findings.iter().map(|f| f.kind.as_str()).collect();
        assert!(
            kinds.contains(&"wait"),
            "every frame carries the wait finding"
        );
        assert!(
            kinds.contains(&"cost"),
            "every frame carries the cost finding"
        );
        assert_eq!(
            frame.anchors.len(),
            frame.findings.len(),
            "every session finding carries an anchor"
        );
        assert!(
            frame.anchors.iter().all(|anchor| anchor.panel == 0),
            "session findings anchor to the frame's one panel"
        );
        assert_eq!(
            telorama_scene::content_hash(frame.svg.as_bytes()),
            frame.svg_fnv1a64,
            "each frame hash must be the hash of its svg"
        );
    }
}
