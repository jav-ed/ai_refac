//! `move-module --dry-run`: the plan is made and checked, nothing is written.

use super::{run, run_json, write};
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

fn dry_run_checked(root: &Path, json: bool) -> std::process::Output {
    let mut arguments = vec!["move-module"];
    if json {
        arguments.push("--json");
    }
    arguments.extend(["--dry-run", "--check", "--project-path"]);
    arguments.extend([root.to_str().unwrap()]);
    arguments.extend(["crate::engine::matching", "crate::domain::matching"]);
    common::run_cli(&arguments)
}

#[test]
fn a_checked_dry_run_compiles_the_move_on_a_copy_and_plans_what_the_real_move_does() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    project(root);
    let before = snapshot(root);

    let planned = dry_run_checked(root, true);
    common::assert_move_succeeded(&planned);
    let plan: serde_json::Value = serde_json::from_slice(&planned.stdout).unwrap();
    assert_eq!(plan["dry_run"], true);
    assert_eq!(plan["compiled"], true, "{plan}");
    assert_eq!(snapshot(root), before, "a dry run must not write");
    assert!(
        !root.join("target").exists() && !root.join("Cargo.lock").exists(),
        "the compile happened on the copy, not in the project"
    );

    let real = run_json(root, "crate::engine::matching", "crate::domain::matching");
    common::assert_move_succeeded(&real);
    let done: serde_json::Value = serde_json::from_slice(&real.stdout).unwrap();
    assert_eq!(done["compiled"], true);
    for key in ["moves", "files", "edits", "edited_files", "moved_paths"] {
        assert_eq!(plan[key], done[key], "{key}");
    }
}

#[test]
fn a_plain_dry_run_says_that_it_did_not_compile() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    project(root);

    let output = dry_run(root, true);
    common::assert_move_succeeded(&output);
    let plan: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(plan["compiled"], false, "{plan}");

    let text = dry_run(root, false);
    let text = String::from_utf8_lossy(&text.stdout).into_owned();
    assert!(text.contains("add --check"), "{text}");
}

#[test]
fn a_checked_dry_run_refuses_a_move_that_would_not_compile_like_the_real_move() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    project(root);
    // A file path in a macro argument is not a module path, so the move leaves
    // it alone and only the compiler sees that it now points nowhere.
    write(
        root,
        "src/domain/mod.rs",
        "pub const SOURCE: &str = include_str!(\"../engine/matching.rs\");\n",
    );
    let before = snapshot(root);

    let plain = dry_run(root, false);
    common::assert_move_succeeded(&plain);

    let checked = dry_run_checked(root, false);
    assert!(!checked.status.success());
    let error = String::from_utf8_lossy(&checked.stderr);
    assert!(error.contains("cargo check"), "{error}");
    assert!(error.contains("copy of the workspace"), "{error}");
    assert!(
        !error.contains("refac-dry-run-"),
        "the copy's folder is named as the project: {error}"
    );
    assert_eq!(snapshot(root), before);

    let real = run(root, "crate::engine::matching", "crate::domain::matching");
    assert!(!real.status.success());
    assert!(String::from_utf8_lossy(&real.stderr).contains("cargo check"));
    assert_eq!(snapshot(root), before);
}

#[test]
fn check_without_dry_run_is_refused_and_says_so() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    project(root);

    let output = common::run_cli(&[
        "move-module",
        "--check",
        "--project-path",
        root.to_str().unwrap(),
        "crate::engine::matching",
        "crate::domain::matching",
    ]);
    assert!(!output.status.success());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("--dry-run"), "{error}");
    assert!(root.join("src/engine/matching.rs").exists());
}
