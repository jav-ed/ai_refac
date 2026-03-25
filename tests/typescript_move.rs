mod common;

// Fixture: tests/fixtures/typescript/project/  (20 files)
//
// Move under test:
//   src/utils/date_helpers.ts  ->  src/lib/date_helpers.ts
//
// Patterns exercised (positive — must be updated):
//   bootstrap/setup.ts       side-effect import (no bindings)
//   utils/string_helpers.ts  same-directory relative named import
//   utils/index.ts           named re-export + type re-export + export *
//   models/task.ts           namespace import (import * as) + import type of moved symbol
//   services/task_service.ts named import + static dynamic import()
//   services/user_service.ts second named import (verifies all files are visited)
//   api/handlers.ts          cross-layer named import (different depth)
//
// Patterns exercised (negative — must NOT be touched):
//   utils/math_helpers.ts    no dep on date_helpers
//   utils/index.ts           export * from './string_helpers' line must survive
//   models/user.ts           no dep on date_helpers
//   api/router.ts            imports only from services barrel
//   src/config.ts            plain config, no deps

fn run_move(project: &std::path::Path) -> std::process::Output {
    common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("src/utils/date_helpers.ts").to_str().unwrap(),
        "--target-path",
        project.join("src/lib/date_helpers.ts").to_str().unwrap(),
    ])
}

// ── file placement ────────────────────────────────────────────────────────────

#[test]
fn typescript_move_places_file_at_target_and_removes_source() {
    let temp = common::setup_fixture("typescript/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    assert!(
        project.join("src/lib/date_helpers.ts").exists(),
        "file must exist at target path"
    );
    assert!(
        !project.join("src/utils/date_helpers.ts").exists(),
        "file must be gone from source path"
    );
}

// ── positive assertions (importers that reference date_helpers) ───────────────

#[test]
fn typescript_move_updates_side_effect_import() {
    let temp = common::setup_fixture("typescript/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    let f = common::read_file(project, "src/bootstrap/setup.ts");
    assert!(!f.contains("'../utils/date_helpers'"), "old path gone:\n{f}");
    assert!(f.contains("'../lib/date_helpers'"), "new path present:\n{f}");
}

#[test]
fn typescript_move_updates_same_dir_sibling_import() {
    let temp = common::setup_fixture("typescript/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    let f = common::read_file(project, "src/utils/string_helpers.ts");
    assert!(!f.contains("'./date_helpers'"), "old sibling path gone:\n{f}");
    assert!(f.contains("'../lib/date_helpers'"), "new cross-dir path present:\n{f}");
}

#[test]
fn typescript_move_updates_barrel_named_reexport() {
    let temp = common::setup_fixture("typescript/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    let f = common::read_file(project, "src/utils/index.ts");
    assert!(!f.contains("'./date_helpers'"), "old barrel path gone:\n{f}");
    assert!(f.contains("'../lib/date_helpers'"), "new barrel path present:\n{f}");
    // The string_helpers re-export must not be corrupted
    assert!(
        f.contains("'./string_helpers'"),
        "string_helpers line must survive unchanged:\n{f}"
    );
}

#[test]
fn typescript_move_updates_namespace_import_and_import_type() {
    let temp = common::setup_fixture("typescript/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    let f = common::read_file(project, "src/models/task.ts");
    assert!(!f.contains("'../utils/date_helpers'"), "old path gone:\n{f}");
    assert!(f.contains("'../lib/date_helpers'"), "new path present:\n{f}");
}

#[test]
fn typescript_move_updates_named_and_dynamic_imports_in_service() {
    let temp = common::setup_fixture("typescript/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    let f = common::read_file(project, "src/services/task_service.ts");
    assert!(!f.contains("'../utils/date_helpers'"), "old path gone:\n{f}");
    assert!(f.contains("'../lib/date_helpers'"), "new path present:\n{f}");
}

#[test]
fn typescript_move_updates_all_importer_files() {
    // Verifies the tool visits every file, not just the first match.
    let temp = common::setup_fixture("typescript/project");
    let project = temp.path();
    common::assert_move_succeeded(&run_move(project));

    for rel in &[
        "src/services/user_service.ts",
        "src/api/handlers.ts",
    ] {
        let f = common::read_file(project, rel);
        assert!(
            !f.contains("'../utils/date_helpers'"),
            "old path should be gone in {rel}:\n{f}"
        );
        assert!(
            f.contains("'../lib/date_helpers'"),
            "new path missing in {rel}:\n{f}"
        );
    }
}

// ── negative assertions (control files must be untouched) ─────────────────────

#[test]
fn typescript_move_does_not_touch_unrelated_files() {
    let temp = common::setup_fixture("typescript/project");
    let project = temp.path();

    // Snapshot control files before
    let math_before = common::read_file(project, "src/utils/math_helpers.ts");
    let user_before = common::read_file(project, "src/models/user.ts");
    let config_before = common::read_file(project, "src/config.ts");
    let router_before = common::read_file(project, "src/api/router.ts");

    common::assert_move_succeeded(&run_move(project));

    assert_eq!(common::read_file(project, "src/utils/math_helpers.ts"), math_before,
        "math_helpers.ts must be byte-identical after move");
    assert_eq!(common::read_file(project, "src/models/user.ts"), user_before,
        "user.ts must be byte-identical after move");
    assert_eq!(common::read_file(project, "src/config.ts"), config_before,
        "config.ts must be byte-identical after move");
    assert_eq!(common::read_file(project, "src/api/router.ts"), router_before,
        "router.ts must be byte-identical after move");
}
