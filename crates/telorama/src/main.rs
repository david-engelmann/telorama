//! The telorama command line tool.
//!
//! Three commands cover the M1 loop an agent runs: inspect a trace file
//! to learn what data exists, validate a scene spec against the schema
//! and the honesty rules, and render a scene to a deterministic SVG.

use std::process::ExitCode;

fn usage() -> &'static str {
    "usage:\n  telorama inspect <traces.json>\n  telorama validate <scene.json>\n  telorama render --scene <scene.json> --traces <traces.json> --out <scene.svg>\n  telorama schema"
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

fn cmd_render(args: &[String]) -> Result<String, String> {
    let mut scene_path: Option<&str> = None;
    let mut traces_path: Option<&str> = None;
    let mut out_path: Option<&str> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--scene" => {
                scene_path = args.get(i + 1).map(|s| s.as_str());
                i += 2;
            }
            "--traces" => {
                traces_path = args.get(i + 1).map(|s| s.as_str());
                i += 2;
            }
            "--out" => {
                out_path = args.get(i + 1).map(|s| s.as_str());
                i += 2;
            }
            other => return Err(format!("unknown render argument {other:?}\n{}", usage())),
        }
    }
    let (Some(scene_path), Some(traces_path), Some(out_path)) = (scene_path, traces_path, out_path)
    else {
        return Err(format!(
            "render needs --scene, --traces, and --out\n{}",
            usage()
        ));
    };
    let scene_text = read_file(scene_path)?;
    let scene = telorama_scene::parse_scene(&scene_text).map_err(|errors| {
        errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let traces_text = read_file(traces_path)?;
    let spans = telorama_ingest::parse_otlp_traces(&traces_text).map_err(|e| e.to_string())?;
    let svg = telorama_svg::render_scene(&scene, &spans).map_err(|e| e.to_string())?;
    std::fs::write(out_path, &svg).map_err(|e| format!("cannot write {out_path}: {e}"))?;
    Ok(format!(
        "wrote {} ({} bytes, fnv1a64 {})\n",
        out_path,
        svg.len(),
        telorama_scene::content_hash(svg.as_bytes())
    ))
}

fn run() -> Result<String, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(|s| s.as_str()) {
        Some("inspect") if args.len() == 2 => cmd_inspect(&args[1]),
        Some("validate") if args.len() == 2 => cmd_validate(&args[1]),
        Some("render") => cmd_render(&args[1..]),
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
