//! The telorama command line tool.
//!
//! The commands cover the loop an agent runs: inspect a trace file to
//! learn what data exists, validate a scene spec against the schema
//! and the honesty rules, render a scene to a deterministic SVG from
//! whatever data its panels read (traces, folded-stack profiles, an
//! agent session export), and print an agent run as JSON for tools
//! that read text rather than pictures.

use std::process::ExitCode;

fn usage() -> &'static str {
    "usage:\n  telorama inspect <traces.json>\n  telorama validate <scene.json>\n  telorama render --scene <scene.json> [--traces <traces.json>] [--profile <stacks.txt>] [--session-logs <logs.json>] [--session-spans <spans.json>] --out <scene.svg>\n  telorama session --logs <logs.json> [--spans <spans.json>]\n  telorama schema"
}

fn read_file(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))
}

fn cmd_inspect(path: &str) -> Result<String, String> {
    let text = read_file(path)?;
    let spans = telorama_ingest::parse_otlp_traces(&text).map_err(|e| e.to_string())?;
    let summary = telorama_ingest::summarize(&spans);
    let mut out = String::new();
    out.push_str(&format!("spans: {}\n", summary.span_count));
    out.push_str(&format!("traces: {}\n", summary.trace_count));
    out.push_str(&format!(
        "window: {} .. {}\n",
        telorama_svg::fmt_time_ms(summary.from_unix_nano / 1_000_000),
        telorama_svg::fmt_time_ms(summary.to_unix_nano / 1_000_000)
    ));
    for service in &summary.services {
        out.push_str(&format!(
            "service {}: {} spans, {} errors\n",
            service.service, service.spans, service.errors
        ));
    }
    Ok(out)
}

fn cmd_validate(path: &str) -> Result<String, String> {
    let text = read_file(path)?;
    match telorama_scene::parse_scene(&text) {
        Ok(scene) => Ok(format!(
            "valid: {} ({} panels, {} data sources)\n",
            scene.title,
            scene.panels.len(),
            scene.data.len()
        )),
        Err(errors) => {
            let mut out = String::new();
            for e in &errors {
                out.push_str(&format!("{e}\n"));
            }
            Err(out.trim_end().to_string())
        }
    }
}

/// Read a flag's value from a render-style argument list.
fn flag<'a>(args: &'a [String], name: &str) -> Result<Option<&'a str>, String> {
    let mut value = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == name {
            value = args.get(i + 1).map(|s| s.as_str());
            i += 2;
        } else {
            i += 1;
        }
    }
    Ok(value)
}

const RENDER_FLAGS: [&str; 6] = [
    "--scene",
    "--traces",
    "--profile",
    "--session-logs",
    "--session-spans",
    "--out",
];

fn cmd_render(args: &[String]) -> Result<String, String> {
    let mut i = 0;
    while i < args.len() {
        if RENDER_FLAGS.contains(&args[i].as_str()) {
            i += 2;
        } else {
            return Err(format!(
                "unknown render argument {:?}\n{}",
                args[i],
                usage()
            ));
        }
    }
    let (Some(scene_path), Some(out_path)) = (flag(args, "--scene")?, flag(args, "--out")?) else {
        return Err(format!("render needs --scene and --out\n{}", usage()));
    };
    let scene_text = read_file(scene_path)?;
    let scene = telorama_scene::parse_scene(&scene_text).map_err(|errors| {
        errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let spans = match flag(args, "--traces")? {
        Some(path) => {
            telorama_ingest::parse_otlp_traces(&read_file(path)?).map_err(|e| e.to_string())?
        }
        None => Vec::new(),
    };
    let stacks = match flag(args, "--profile")? {
        Some(path) => {
            telorama_ingest::parse_folded_stacks(&read_file(path)?).map_err(|e| e.to_string())?
        }
        None => Vec::new(),
    };
    let run = match flag(args, "--session-logs")? {
        Some(path) => {
            let fusion_spans = match flag(args, "--session-spans")? {
                Some(spans_path) => telorama_ingest::parse_otlp_traces(&read_file(spans_path)?)
                    .map_err(|e| e.to_string())?,
                None => Vec::new(),
            };
            Some(
                telorama_ingest::parse_claude_code_session(&read_file(path)?, &fusion_spans)
                    .map_err(|e| e.to_string())?,
            )
        }
        None => None,
    };
    let data = telorama_svg::SceneData {
        spans: &spans,
        stacks: &stacks,
        agent_run: run.as_ref(),
    };
    let svg = telorama_svg::render_scene_with_data(&scene, &data).map_err(|e| e.to_string())?;
    std::fs::write(out_path, &svg).map_err(|e| format!("cannot write {out_path}: {e}"))?;
    Ok(format!(
        "wrote {} ({} bytes, fnv1a64 {})\n",
        out_path,
        svg.len(),
        telorama_scene::content_hash(svg.as_bytes())
    ))
}

fn cmd_session(args: &[String]) -> Result<String, String> {
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--logs" | "--spans" => i += 2,
            other => {
                return Err(format!("unknown session argument {other:?}\n{}", usage()));
            }
        }
    }
    let Some(logs_path) = flag(args, "--logs")? else {
        return Err(format!("session needs --logs\n{}", usage()));
    };
    let spans = match flag(args, "--spans")? {
        Some(path) => {
            telorama_ingest::parse_otlp_traces(&read_file(path)?).map_err(|e| e.to_string())?
        }
        None => Vec::new(),
    };
    let run = telorama_ingest::parse_claude_code_session(&read_file(logs_path)?, &spans)
        .map_err(|e| e.to_string())?;
    let mut out = serde_json::to_string_pretty(&run).map_err(|e| e.to_string())?;
    out.push('\n');
    Ok(out)
}

fn run() -> Result<String, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(|s| s.as_str()) {
        Some("inspect") if args.len() == 2 => cmd_inspect(&args[1]),
        Some("validate") if args.len() == 2 => cmd_validate(&args[1]),
        Some("render") => cmd_render(&args[1..]),
        Some("session") => cmd_session(&args[1..]),
        Some("schema") if args.len() == 1 => Ok(telorama_scene::scene_schema_text()),
        _ => Err(usage().to_string()),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(out) => {
            print!("{out}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
