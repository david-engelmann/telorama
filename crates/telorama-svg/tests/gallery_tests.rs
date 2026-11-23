//! The gallery page at docs/gallery.html and its freshness gate
//! (INV-62 to INV-64). The test assembles the page from the shell
//! at docs/gallery-shell.html and one render per golden scene
//! family, each through the tool render path, so the findings
//! strip is drawn in whenever the investigation layer finds
//! something. Run with UPDATE_GALLERY=1 it writes the assembled
//! page. Run normally it asserts the committed page is that
//! assembly, byte for byte.
//!
//! The findings rule, stated once here because the tools wire
//! it in two places. Session scenes follow the session wiring
//! both tools share, findings ranked over the run alone, the
//! way the CLI session invocation and the MCP session_run tool
//! compute them. The GenAI scene is the exception. Its spans
//! are the scene's declared data, they sit in the render's
//! data, and the investigation ranks them alongside the run,
//! as the MCP render path does with its input spans. The CLI
//! session invocation ranks the run alone even there, because
//! its session spans never reach the investigation. That
//! divergence is recorded in DECISIONS.md with the Round 2
//! entry.

use std::collections::BTreeSet;
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

/// Render one section through the tool render path. The
/// findings are computed over `findings_over`, the data the
/// render carries for investigation, and the scene renders
/// over the spans, stacks, run, and state it is given.
#[allow(clippy::too_many_arguments)]
fn tool_render(
    family: &'static str,
    scene: Scene,
    spans: &[Span],
    stacks: &[telorama_scene::StackSample],
    run: Option<&AgentRun>,
    state: &[telorama_scene::AgentStateEntry],
    findings_over: &[Span],
) -> Section {
    let findings = telorama_scene::investigate(findings_over, run, scene.context.range.seconds());
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
        family,
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
    tool_render(family, scene, &spans, &[], None, &[], &spans)
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
        out.push(tool_render(
            "Agent runs",
            scene,
            &[],
            &[],
            Some(&run),
            &[],
            &[],
        ));
    }
    for (scene_file, logs_file) in [
        ("agent_run_codex_scene.json", "codex-session-logs.json"),
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
        out.push(tool_render(
            "Agent runs",
            scene,
            &[],
            &[],
            Some(&run),
            &[],
            &[],
        ));
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
        out.push(tool_render(
            "Agent runs",
            scene,
            &spans,
            &[],
            Some(&run),
            &[],
            &spans,
        ));
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
        out.push(tool_render("Agent state", scene, &[], &[], None, &state, &[]));
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
        out.push(tool_render(
            "Incident composition",
            scene,
            &spans,
            &[],
            None,
            &[],
            &spans,
        ));
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
        let scene =
            telorama_scene::parse_scene(&workspace_file("tests/fixtures/scenes/flame_scene.json"))
                .expect("flame scene must validate");
        let stacks = telorama_ingest::parse_folded_stacks(&workspace_file(
            "tests/fixtures/profiles/checkout-cpu.txt",
        ))
        .expect("profile fixture must parse");
        out.push(tool_render(
            "Trace diagrams",
            scene,
            &[],
            &stacks,
            None,
            &[],
            &[],
        ));
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

/// Every fixture scene's title and question pair, read from
/// the fixture directory itself, so a scene added to the
/// corpus without a gallery section fails this gate (INV-63).
fn fixture_pairs() -> BTreeSet<(String, String)> {
    let dir = repo_path("tests/fixtures/scenes");
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .filter_map(|entry| {
            let name = entry.ok()?.file_name().into_string().ok()?;
            name.ends_with(".json").then_some(name)
        })
        .collect();
    names.sort();
    names
        .iter()
        .map(|name| {
            let scene = telorama_scene::parse_scene(&workspace_file(&format!(
                "tests/fixtures/scenes/{name}"
            )))
            .expect("fixture scene must validate");
            (
                scene.title.clone(),
                scene.context.provenance.question.clone(),
            )
        })
        .collect()
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
        !committed.to_lowercase().contains("<script"),
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
    // Coverage. Every fixture scene appears as a section, and
    // the sections carry exactly one more pair, the incident
    // composition (INV-63).
    let built = sections();
    let section_pairs: BTreeSet<(String, String)> = built
        .iter()
        .map(|s| {
            (
                s.scene.title.clone(),
                s.scene.context.provenance.question.clone(),
            )
        })
        .collect();
    let fixtures = fixture_pairs();
    assert!(
        fixtures.is_subset(&section_pairs),
        "a fixture scene has no gallery section"
    );
    assert_eq!(
        section_pairs.len(),
        fixtures.len() + 1,
        "the sections are the fixture scenes plus the incident composition"
    );
    assert_eq!(
        committed.matches("<section class=\"card\">").count(),
        built.len(),
        "one card per built section"
    );
    // The README's first screen links the page (INV-64).
    let readme = workspace_file("README.md");
    assert!(
        readme.contains("(docs/gallery.html)"),
        "the README links the gallery"
    );
}
