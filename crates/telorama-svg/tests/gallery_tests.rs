//! The gallery page at docs/gallery.html and its freshness gate
//! (INV-62 to INV-64). The test assembles the page from the shell
//! at docs/gallery-shell.html and one render per golden scene
//! family, each through the tool render path the CLI uses, so
//! the findings strip is drawn in whenever the investigation
//! layer finds something. Run with UPDATE_GALLERY=1 it writes
//! the assembled page; run normally it asserts the committed
//! page is that assembly, byte for byte.

use std::path::PathBuf;
use telorama_scene::{AgentRun, Scene, Span};

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

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

struct Section {
    family: &'static str,
    scene: Scene,
    svg: String,
}

/// Render one section exactly the way the CLI render command
/// does: findings computed over the data the render carries,
/// then render_scene_with_data.
fn tool_render(
    scene: Scene,
    spans: &[Span],
    stacks: &[telorama_scene::StackSample],
    run: Option<&AgentRun>,
    state: &[telorama_scene::AgentStateEntry],
) -> Section {
    let findings = telorama_scene::investigate(spans, run, scene.context.range.seconds());
    let svg = telorama_svg::render_scene_with_data(
        &scene,
        &telorama_svg::SceneData {
            spans,
            stacks,
            agent_run: run,
            agent_state: state,
            findings: &findings,
        },
    )
    .expect("gallery scene must render");
    Section {
        family: "",
        scene,
        svg,
    }
}

fn trace_section(family: &'static str, scene_file: &str, traces_file: &str) -> Section {
    let scene = telorama_scene::parse_scene(&workspace_file(&format!(
        "tests/fixtures/scenes/{scene_file}"
    )))
    .expect("fixture scene must validate");
    let spans = telorama_ingest::parse_otlp_traces(&workspace_file(&format!(
        "tests/fixtures/{traces_file}"
    )))
    .expect("fixture traces must parse");
    let mut section = tool_render(scene, &spans, &[], None, &[]);
    section.family = family;
    section
}

fn sections() -> Vec<Section> {
    let mut out: Vec<Section> = Vec::new();
    // Agent runs, the flagship family.
    {
        let scene = telorama_scene::parse_scene(&workspace_file(
            "tests/fixtures/scenes/agent_run_scene.json",
        ))
        .expect("agent run scene must validate");
        let logs = workspace_file("tests/fixtures/agents/claude-code-session-logs.json");
        let spans = telorama_ingest::parse_otlp_traces(&workspace_file(
            "tests/fixtures/agents/claude-code-session-spans.json",
        ))
        .expect("session spans must parse");
        let run = telorama_ingest::parse_claude_code_session(&logs, &spans)
            .expect("session fixture must adapt");
        let findings =
            telorama_scene::investigate(&[], Some(&run), scene.context.range.seconds());
        let svg = telorama_svg::render_scene_with_data(
            &scene,
            &telorama_svg::SceneData {
                spans: &[],
                stacks: &[],
                agent_run: Some(&run),
                agent_state: &[],
                findings: &findings,
            },
        )
        .expect("agent run scene must render");
        out.push(Section {
            family: "Agent runs",
            scene,
            svg,
        });
    }
    for (scene_file, logs_file) in [
        (
            "agent_run_codex_scene.json",
            "codex-session-logs.json",
        ),
        (
            "agent_run_codex_unpriced_scene.json",
            "codex-session-unpriced-logs.json",
        ),
    ] {
        let scene = telorama_scene::parse_scene(&workspace_file(&format!(
            "tests/fixtures/scenes/{scene_file}"
        )))
        .expect("codex scene must validate");
        let logs = workspace_file(&format!("tests/fixtures/agents/{logs_file}"));
        let run =
            telorama_ingest::parse_codex_session(&logs, &[]).expect("codex fixture must adapt");
        let findings =
            telorama_scene::investigate(&[], Some(&run), scene.context.range.seconds());
        let svg = telorama_svg::render_scene_with_data(
            &scene,
            &telorama_svg::SceneData {
                spans: &[],
                stacks: &[],
                agent_run: Some(&run),
                agent_state: &[],
                findings: &findings,
            },
        )
        .expect("codex scene must render");
        out.push(Section {
            family: "Agent runs",
            scene,
            svg,
        });
    }
    {
        let scene = telorama_scene::parse_scene(&workspace_file(
            "tests/fixtures/scenes/agent_run_genai_scene.json",
        ))
        .expect("genai scene must validate");
        let spans = telorama_ingest::parse_otlp_traces(&workspace_file(
            "tests/fixtures/agents/genai-session-spans.json",
        ))
        .expect("genai fixture must parse");
        let run = telorama_ingest::parse_genai_session(&spans).expect("spans must adapt");
        let mut section = tool_render(scene, &spans, &[], Some(&run), &[]);
        section.family = "Agent runs";
        out.push(section);
    }
    // Agent state.
    {
        let scene = telorama_scene::parse_scene(&workspace_file(
            "tests/fixtures/scenes/agent_state_scene.json",
        ))
        .expect("agent state scene must validate");
        let state = telorama_ingest::parse_agent_state(&workspace_file(
            "tests/fixtures/agents/agent-state.json",
        ))
        .expect("state fixture must parse");
        let mut section = tool_render(scene, &[], &[], None, &state);
        section.family = "Agent state";
        out.push(section);
    }
    // The incident composition, the pair of incident goldens
    // shown once with its findings strip.
    {
        let spans = telorama_ingest::parse_otlp_traces(&workspace_file(
            "tests/fixtures/otel-demo-checkout.json",
        ))
        .expect("checkout fixture must parse");
        let scene = telorama_scene::incident_scene(&spans, None, "agent")
            .expect("composition must succeed");
        let mut section = tool_render(scene, &spans, &[], None, &[]);
        section.family = "Incident composition";
        out.push(section);
    }
    // Trace diagrams.
    out.push(trace_section(
        "Trace diagrams",
        "checkout_scene.json",
        "otel-demo-checkout.json",
    ));
    out.push(trace_section(
        "Trace diagrams",
        "api_scene.json",
        "single-service-api.json",
    ));
    out.push(trace_section(
        "Trace diagrams",
        "heatmap_scene.json",
        "otel-demo-checkout.json",
    ));
    out.push(trace_section(
        "Trace diagrams",
        "service_map_scene.json",
        "otel-demo-checkout.json",
    ));
    out.push(trace_section(
        "Trace diagrams",
        "sankey_scene.json",
        "otel-demo-checkout.json",
    ));
    {
        let scene = telorama_scene::parse_scene(&workspace_file(
            "tests/fixtures/scenes/flame_scene.json",
        ))
        .expect("flame scene must validate");
        let stacks = telorama_ingest::parse_folded_stacks(&workspace_file(
            "tests/fixtures/profiles/checkout-cpu.txt",
        ))
        .expect("profile fixture must parse");
        let mut section = tool_render(scene, &[], &stacks, None, &[]);
        section.family = "Trace diagrams";
        out.push(section);
    }
    out
}

fn assemble_gallery() -> String {
    let shell = workspace_file("docs/gallery-shell.html");
    let mut body = String::new();
    let mut last_family = "";
    for section in sections() {
        if section.family != last_family {
            body.push_str(&format!(
                "<h2 class=\"family\">{}</h2>\n",
                escape_html(section.family)
            ));
            last_family = section.family;
        }
        body.push_str(&format!(
            "<section class=\"card\">\n<h3>{}</h3>\n<p class=\"question\">{}</p>\n<div class=\"scene\">\n{}\n</div>\n</section>\n",
            escape_html(&section.scene.title),
            escape_html(&section.scene.context.provenance.question),
            section.svg
        ));
    }
    shell.replace("<!--GALLERY_SECTIONS-->", &body)
}

#[test]
fn gallery_page_matches_the_current_renders() {
    let assembled = assemble_gallery();
    let page_path = repo_path("docs/gallery.html");
    if std::env::var_os("UPDATE_GALLERY").is_some() {
        std::fs::write(&page_path, &assembled).expect("gallery must be writable");
        return;
    }
    let committed = std::fs::read_to_string(&page_path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", page_path.display()));
    assert_eq!(
        assembled, committed,
        "the gallery page differs from the current renders; rerun with UPDATE_GALLERY=1 and review the diff"
    );
    assert!(
        !committed.contains("<script"),
        "the gallery carries no script (INV-64)"
    );
    assert!(
        committed.contains("media/frames.gif"),
        "the head carries the frame stream"
    );
    assert!(
        committed.contains("media/incident.png"),
        "the head carries the incident still"
    );
    assert_eq!(
        committed.matches("<section class=\"card\">").count(),
        12,
        "one section per scene family, the incident pair shown once"
    );
}
