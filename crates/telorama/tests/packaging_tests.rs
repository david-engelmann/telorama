//! Packaging shape, INV-78 and INV-79. The five crates
//! version together, each manifest names its own readme,
//! and no crate readme carries a relative image or link,
//! because the registries rewrite relative paths against
//! the crate directory, where the files do not live.

use std::path::PathBuf;

fn workspace_file(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel)
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(workspace_file(rel)).unwrap_or_else(|_| panic!("{rel} must exist"))
}

const CRATES: [(&str, &str); 5] = [
    ("telorama", "crates/telorama"),
    ("telorama-scene", "crates/telorama-scene"),
    ("telorama-ingest", "crates/telorama-ingest"),
    ("telorama-svg", "crates/telorama-svg"),
    ("telorama-mcp", "crates/telorama-mcp"),
];

fn manifest_version(manifest: &str) -> String {
    for line in manifest.lines() {
        if let Some(rest) = line.strip_prefix("version = \"") {
            return rest.trim_end_matches('"').to_string();
        }
    }
    panic!("manifest must carry a version");
}

#[test]
fn the_five_crates_version_together() {
    let versions: Vec<String> = CRATES
        .iter()
        .map(|(_, dir)| manifest_version(&read(&format!("{dir}/Cargo.toml"))))
        .collect();
    assert!(
        versions.iter().all(|v| *v == versions[0]),
        "crate versions disagree: {versions:?}"
    );
    let root = read("Cargo.toml");
    for name in ["telorama-scene", "telorama-ingest", "telorama-svg"] {
        let pin = format!(
            "{name} = {{ path = \"crates/{name}\", version = \"{}\" }}",
            versions[0]
        );
        assert!(root.contains(&pin), "workspace pin for {name} is stale");
    }
}

#[test]
fn every_crate_readme_is_its_own_and_absolute() {
    for (name, dir) in CRATES {
        let manifest = read(&format!("{dir}/Cargo.toml"));
        assert!(
            manifest.contains("readme = \"README.md\""),
            "{name} must name its own readme"
        );
        let readme = read(&format!("{dir}/README.md"));
        assert!(
            readme.starts_with(&format!("# {name}\n")),
            "{name} readme must open with its own title"
        );
        let mut targets: Vec<String> = Vec::new();
        let mut rest = readme.as_str();
        while let Some(idx) = rest.find("](") {
            let after = &rest[idx + 2..];
            targets.push(
                after
                    .chars()
                    .take_while(|c| *c != ')' && *c != ' ')
                    .collect(),
            );
            rest = after;
        }
        for line in readme.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[')
                && let Some((_, target)) = trimmed.split_once("]: ")
            {
                targets.push(target.chars().take_while(|c| !c.is_whitespace()).collect());
            }
            if let Some(at) = trimmed.find("src=\"") {
                let after = &trimmed[at + 5..];
                targets.push(after.chars().take_while(|c| *c != '"').collect());
            }
        }
        for target in targets {
            if !target.starts_with('#') {
                assert!(
                    target.starts_with("https://"),
                    "{name} readme carries a relative link target: {target}"
                );
            }
        }
    }
}
