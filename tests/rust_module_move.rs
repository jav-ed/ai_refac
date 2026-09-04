mod common;

use std::{fs, path::Path, process::Output};

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn run(project: &Path, source: &str, target: &str) -> Output {
    common::run_cli(&[
        "move-module",
        "--project-path",
        project.to_str().unwrap(),
        source,
        target,
    ])
}

fn run_json(project: &Path, source: &str, target: &str) -> Output {
    common::run_cli(&[
        "move-module",
        "--json",
        "--project-path",
        project.to_str().unwrap(),
        source,
        target,
    ])
}

#[test]
fn moves_flat_module_and_rewrites_external_and_super_paths() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write(
        root,
        "src/lib.rs",
        "pub mod consumer;\npub mod domain;\npub mod engine;\n",
    );
    write(root, "src/domain/mod.rs", "");
    write(
        root,
        "src/engine/mod.rs",
        "pub mod matching;\npub mod sibling;\n",
    );
    write(
        root,
        "src/engine/sibling.rs",
        "pub fn value() -> u32 { 7 }\n",
    );
    write(
        root,
        "src/engine/matching.rs",
        "use super::sibling;\n\npub fn execute() -> u32 { sibling::value() }\n\n#[cfg(test)]\nmod tests {\n    use super::super::super::engine::sibling;\n    #[test]\n    fn reads_sibling() { assert_eq!(sibling::value(), 7); }\n}\n",
    );
    write(
        root,
        "src/consumer.rs",
        "use crate::engine::matching::execute;\npub fn consume() -> u32 { execute() }\n",
    );

    let output = run(root, "crate::engine::matching", "crate::domain::matching");
    common::assert_move_succeeded(&output);

    assert!(!root.join("src/engine/matching.rs").exists());
    assert!(root.join("src/domain/matching.rs").exists());
    let engine = common::read_file(root, "src/engine/mod.rs");
    let domain = common::read_file(root, "src/domain/mod.rs");
    let consumer = common::read_file(root, "src/consumer.rs");
    let moved = common::read_file(root, "src/domain/matching.rs");
    assert!(!engine.contains("mod matching"), "{engine}");
    assert!(domain.contains("pub mod matching;"), "{domain}");
    assert!(
        consumer.contains("crate::domain::matching::execute"),
        "{consumer}"
    );
    assert!(moved.contains("use crate::engine::sibling;"), "{moved}");
    assert!(!moved.contains("#[path"));
}

#[test]
fn moves_mod_rs_subtree_and_creates_missing_parent() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        "Cargo.toml",
        "[package]\nname = \"subtree\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write(
        root,
        "src/lib.rs",
        "pub mod feature;\npub mod shared;\npub use feature::run;\n",
    );
    write(root, "src/shared.rs", "pub fn value() -> u32 { 3 }\n");
    write(
        root,
        "src/feature/mod.rs",
        "use super::shared;\npub mod child;\npub fn run() -> u32 { shared::value() + child::read() }\n",
    );
    write(
        root,
        "src/feature/child.rs",
        "use super::super::shared;\npub fn read() -> u32 { shared::value() }\n",
    );

    let output = run(root, "crate::feature", "crate::domain::feature");
    common::assert_move_succeeded(&output);

    assert!(!root.join("src/feature").exists());
    assert!(root.join("src/domain/feature/mod.rs").exists());
    let lib = common::read_file(root, "src/lib.rs");
    let domain = common::read_file(root, "src/domain/mod.rs");
    let feature = common::read_file(root, "src/domain/feature/mod.rs");
    let child = common::read_file(root, "src/domain/feature/child.rs");
    assert!(lib.contains("pub mod domain;"), "{lib}");
    assert!(
        lib.contains("pub use crate::domain::feature::run;"),
        "{lib}"
    );
    assert!(domain.contains("pub mod feature;"), "{domain}");
    assert!(feature.contains("use crate::shared;"), "{feature}");
    assert!(child.contains("use crate::shared;"), "{child}");
    assert!(!lib.contains("#[path"));
}

#[test]
fn moves_flat_module_with_companion_subtree() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        "Cargo.toml",
        "[package]\nname = \"companion\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write(
        root,
        "src/lib.rs",
        "pub mod domain;\npub mod feature;\npub mod shared;\n",
    );
    write(root, "src/domain/mod.rs", "");
    write(root, "src/shared.rs", "pub fn value() -> u32 { 5 }\n");
    write(
        root,
        "src/feature.rs",
        "use super::shared;\npub mod child;\npub fn run() -> u32 { shared::value() + child::read() }\n",
    );
    write(
        root,
        "src/feature/child.rs",
        "use super::super::shared;\npub fn read() -> u32 { shared::value() }\n",
    );

    let output = run(root, "crate::feature", "crate::domain::feature");
    common::assert_move_succeeded(&output);

    assert!(!root.join("src/feature.rs").exists());
    assert!(!root.join("src/feature").exists());
    assert!(root.join("src/domain/feature.rs").exists());
    assert!(root.join("src/domain/feature/child.rs").exists());
    let child = common::read_file(root, "src/domain/feature/child.rs");
    assert!(child.contains("use crate::shared;"), "{child}");
}

#[test]
fn rewrites_references_in_dependent_workspace_crates() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"core\", \"app\"]\nresolver = \"3\"\n",
    );
    write(
        root,
        "core/Cargo.toml",
        "[package]\nname = \"core_lib\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write(root, "core/src/lib.rs", "pub mod matching;\n");
    write(
        root,
        "core/src/matching.rs",
        "pub fn value() -> u32 { 11 }\n",
    );
    write(
        root,
        "app/Cargo.toml",
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n[dependencies]\ncore_lib = { path = \"../core\" }\n",
    );
    write(
        root,
        "app/src/main.rs",
        "use core_lib::matching::value;\nfn main() { println!(\"{}\", value()); }\n",
    );

    let output = run(root, "crate::matching", "crate::domain::matching");
    common::assert_move_succeeded(&output);

    let app = common::read_file(root, "app/src/main.rs");
    assert!(app.contains("core_lib::domain::matching::value"), "{app}");
    assert!(root.join("core/src/domain/matching.rs").exists());
}

#[test]
fn moves_nested_module_to_crate_root() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        "Cargo.toml",
        "[package]\nname = \"root_move\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write(
        root,
        "src/lib.rs",
        "pub mod nested;\npub use nested::value::read;\n",
    );
    write(root, "src/nested/mod.rs", "pub mod value;\n");
    write(root, "src/nested/value.rs", "pub fn read() -> u32 { 1 }\n");

    let output = run(root, "crate::nested::value", "crate::value");
    common::assert_move_succeeded(&output);

    assert!(root.join("src/value.rs").exists());
    assert!(!root.join("src/nested/value.rs").exists());
    let lib = common::read_file(root, "src/lib.rs");
    assert!(lib.contains("pub mod value;"), "{lib}");
    assert!(lib.contains("pub use crate::value::read;"), "{lib}");
}

#[test]
fn json_success_reports_semantic_move_counts() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        "Cargo.toml",
        "[package]\nname = \"json_report\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write(root, "src/lib.rs", "pub mod source;\n");
    write(root, "src/source.rs", "pub fn value() {}\n");

    let output = run_json(root, "crate::source", "crate::domain::source");
    common::assert_move_succeeded(&output);
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["operation"], "move-module");
    assert_eq!(payload["source_module"], "crate::source");
    assert_eq!(payload["target_module"], "crate::domain::source");
    assert_eq!(payload["moved_paths"], 1);
    assert!(payload["edited_files"].as_u64().unwrap() >= 2);
}
