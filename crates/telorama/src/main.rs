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
    "usage:\n  telorama inspect <traces.json>\n  telorama validate <scene.json>\n  telorama render --scene <scene.json> [--traces <traces.json>] [--profile <stacks.txt>] [--session-logs <logs.json>] [--session-spans <spans.json>] [--dialect <dialect>] --out <scene.svg>\n  telorama render-html --scene <scene.json> [--traces <traces.json>] [--profile <stacks.txt>] [--session-logs <logs.json>] [--session-spans <spans.json>] [--dialect <dialect>] --out <scene.html>\n  telorama session [--logs <logs.json>] [--spans <spans.json>] [--dialect <dialect>]\n  telorama incident --traces <traces.json> [--session-logs <logs.json>] [--session-spans <spans.json>] [--dialect <dialect>] [--out <scene.json>] [--render <scene.svg>]\n  telorama schema"
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

const RENDER_FLAGS: [&str; 7] = [
    "--scene",
    "--traces",
    "--profile",
    "--session-logs",
    "--session-spans",
    "--dialect",
    "--out",
];

/// Build an agent run from a session export in the named dialect.
fn parse_session(
    logs_path: Option<&str>,
    spans_path: Option<&str>,
    dialect: &str,
) -> Result<telorama_scene::AgentRun, String> {
    let spans = match spans_path {
        Some(path) => {
            telorama_ingest::parse_otlp_traces(&read_file(path)?).map_err(|e| e.to_string())?
        }
        None => Vec::new(),
    };
    match dialect {
        "genai" | "openinference" => {
            if spans_path.is_none() {
                return Err(format!("session --dialect {dialect} needs --spans"));
            }
            match dialect {
                "genai" => telorama_ingest::parse_genai_session(&spans).map_err(|e| e.to_string()),
                _ => {
                    telorama_ingest::parse_openinference_session(&spans).map_err(|e| e.to_string())
                }
            }
        }
        "claude_code" | "codex" => {
            let Some(path) = logs_path else {
                return Err("session needs --logs for the log dialects".to_string());
            };
            let logs = read_file(path)?;
            match dialect {
                "claude_code" => telorama_ingest::parse_claude_code_session(&logs, &spans)
                    .map_err(|e| e.to_string()),
                _ => telorama_ingest::parse_codex_session(&logs, &spans).map_err(|e| e.to_string()),
            }
        }
        other => Err(format!(
            "unknown dialect {other:?}: expected claude_code, codex, genai, or openinference"
        )),
    }
}

fn cmd_render(args: &[String]) -> Result<String, String> {
    cmd_render_like(args, false)
}

fn cmd_render_html(args: &[String]) -> Result<String, String> {
    cmd_render_like(args, true)
}

fn cmd_render_like(args: &[String], html: bool) -> Result<String, String> {
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
    let logs_flag = flag(args, "--session-logs")?;
    let spans_flag = flag(args, "--session-spans")?;
    let run = if logs_flag.is_some() || spans_flag.is_some() {
        Some(parse_session(
            logs_flag,
            spans_flag,
            flag(args, "--dialect")?.unwrap_or("claude_code"),
        )?)
    } else {
        None
    };
    let data = telorama_svg::SceneData {
        spans: &spans,
        stacks: &stacks,
        agent_run: run.as_ref(),
    };
    let svg = telorama_svg::render_scene_with_data(&scene, &data).map_err(|e| e.to_string())?;
    let out_text = if html {
        telorama_svg::standalone_html(&scene.title, &svg)
    } else {
        svg
    };
    std::fs::write(out_path, &out_text).map_err(|e| format!("cannot write {out_path}: {e}"))?;
    Ok(format!(
        "wrote {} ({} bytes, fnv1a64 {})\n",
        out_path,
        out_text.len(),
        telorama_scene::content_hash(out_text.as_bytes())
    ))
}

const INCIDENT_FLAGS: [&str; 6] = [
    "--traces",
    "--session-logs",
    "--session-spans",
    "--dialect",
    "--out",
    "--render",
];

fn cmd_incident(args: &[String]) -> Result<String, String> {
    let mut i = 0;
    while i < args.len() {
        if INCIDENT_FLAGS.contains(&args[i].as_str()) {
            i += 2;
        } else {
            return Err(format!(
                "unknown incident argument {:?}\n{}",
                args[i],
                usage()
            ));
        }
    }
    let Some(traces_path) = flag(args, "--traces")? else {
        return Err(format!("incident needs --traces\n{}", usage()));
    };
    let spans =
        telorama_ingest::parse_otlp_traces(&read_file(traces_path)?).map_err(|e| e.to_string())?;
    let logs_flag = flag(args, "--session-logs")?;
    let spans_flag = flag(args, "--session-spans")?;
    let run = if logs_flag.is_some() || spans_flag.is_some() {
        Some(parse_session(
            logs_flag,
            spans_flag,
            flag(args, "--dialect")?.unwrap_or("claude_code"),
        )?)
    } else {
        None
    };
    let scene = telorama_scene::incident_scene(&spans, run.as_ref(), "telorama-cli")?;
    let mut text = serde_json::to_string_pretty(&scene).map_err(|e| e.to_string())?;
    text.push('\n');
    if let Some(out_path) = flag(args, "--out")? {
        std::fs::write(out_path, &text).map_err(|e| format!("cannot write {out_path}: {e}"))?;
    }
    if let Some(render_path) = flag(args, "--render")? {
        let svg = telorama_svg::render_scene_with_data(
            &scene,
            &telorama_svg::SceneData {
                spans: &spans,
                stacks: &[],
                agent_run: run.as_ref(),
            },
        )?;
        std::fs::write(render_path, svg).map_err(|e| format!("cannot write {render_path}: {e}"))?;
    }
    Ok(text)
}

fn cmd_session(args: &[String]) -> Result<String, String> {
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--logs" | "--spans" | "--dialect" => i += 2,
            other => {
                return Err(format!("unknown session argument {other:?}\n{}", usage()));
            }
        }
    }
    let run = parse_session(
        flag(args, "--logs")?,
        flag(args, "--spans")?,
        flag(args, "--dialect")?.unwrap_or("claude_code"),
    )?;
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
        Some("render-html") => cmd_render_html(&args[1..]),
        Some("session") => cmd_session(&args[1..]),
        Some("incident") => cmd_incident(&args[1..]),
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
