//! `move-module --dry-run`: the plan is made and checked, nothing is written.

use super::{run_json, write};
use crate::common;
use std::{collections::BTreeMap, fs, path::Path};

fn project(root: &Path) {
    write(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write(
        root,
        "src/lib.rs",
        "pub mod domain;\npub mod engine;\n\npub fn run() -> u32 {\n    engine::matching::value() + crate::engine::matching::value()\n}\n",
    );
    write(root, "src/domain/mod.rs", "");
    write(root, "src/engine/mod.rs", "pub mod matching;\n");
    write(
        root,
        "src/engine/matching.rs",
        "pub fn value() -> u32 { 7 }\n",
    );
}

/// Every file below `root` with its content, without Cargo's own folders.
fn snapshot(root: &Path) -> BTreeMap<String, String> {
    let mut found = BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                if path.file_name().unwrap() != "target" {
                    pending.push(path);
                }
            } else {
                let name = path.strip_prefix(root).unwrap().display().to_string();
                found.insert(name, fs::read_to_string(&path).unwrap_or_default());
            }
        }
    }
    found
}

fn dry_run(root: &Path, json: bool) -> std::process::Output {
    let mut arguments = vec!["move-module"];
    if json {
        arguments.push("--json");
    }
    arguments.extend(["--dry-run", "--project-path", root.to_str().unwrap()]);
    arguments.extend(["crate::engine::matching", "crate::domain::matching"]);
    common::run_cli(&arguments)
}

#[test]
fn a_dry_run_lists_the_plan_and_changes_no_file() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    project(root);
    let before = snapshot(root);

    let output = dry_run(root, false);
    common::assert_move_succeeded(&output);
    let text = String::from_utf8_lossy(&output.stdout);

    assert!(text.contains("Dry run: nothing was changed"), "{text}");
    assert!(
        text.contains("move src/engine/matching.rs -> src/domain/matching.rs"),
        "{text}"
    );
    assert!(text.contains("src/lib.rs (2 edits)"), "{text}");
    assert!(text.contains("without --dry-run"), "{text}");
    assert_eq!(snapshot(root), before, "a dry run must not write");
}

#[test]
fn the_dry_run_json_names_the_moves_and_the_edits_per_file() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    project(root);

    let output = dry_run(root, true);
    common::assert_move_succeeded(&output);
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(payload["dry_run"], true);
    assert_eq!(payload["moves"][0]["from"], "src/engine/matching.rs");
    assert_eq!(payload["moves"][0]["to"], "src/domain/matching.rs");
    let files = payload["files"].as_array().unwrap();
    let lib = files
        .iter()
        .find(|file| file["path"] == "src/lib.rs")
        .unwrap();
    assert_eq!(lib["edits"], 2);
    assert_eq!(
        payload["edits"],
        files
            .iter()
            .map(|f| f["edits"].as_u64().unwrap())
            .sum::<u64>()
    );
}

#[test]
fn a_dry_run_refuses_what_the_real_move_refuses() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    project(root);
    write(root, "src/domain/mod.rs", "pub mod matching;\n");
    write(root, "src/domain/matching.rs", "");
    let before = snapshot(root);

    let output = dry_run(root, false);

    assert!(!output.status.success());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("already exists"), "{error}");
    assert_eq!(snapshot(root), before);
}

#[test]
fn the_real_move_reports_no_dry_run() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    project(root);

    let output = run_json(root, "crate::engine::matching", "crate::domain::matching");
    common::assert_move_succeeded(&output);
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(payload["dry_run"], false);
    assert!(root.join("src/domain/matching.rs").exists());
}
