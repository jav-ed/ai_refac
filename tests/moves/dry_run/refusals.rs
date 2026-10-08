//! A dry run refuses what the real move refuses, with the same message, and
//! still changes nothing.

use crate::common::project::{Project, assert_same_tree};
use crate::common::{run_cli, stderr_text, stdout_text};

fn dry_run(project: &Project, source: &str, target: &str) -> std::process::Output {
    run_cli(&[
        "move",
        "--dry-run",
        "--project-path",
        project.path().to_str().unwrap(),
        "--source-path",
        source,
        "--target-path",
        target,
    ])
}

fn typescript_project() -> Project {
    let project = Project::empty();
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"target":"es2020","module":"commonjs"},"include":["src/**/*"]}"#,
    );
    project.write("src/a.ts", "export const a = 1;\n");
    project.write("src/b.ts", "export const b = 2;\n");
    project
}

#[test]
fn a_target_that_exists_is_refused_and_nothing_changes() {
    let project = typescript_project();
    let before = project.tree();
    let output = dry_run(&project, "src/a.ts", "src/b.ts");
    assert!(!output.status.success());
    assert!(
        stderr_text(&output).contains("already exists"),
        "{}",
        stderr_text(&output)
    );
    assert_same_tree(&before, &project.tree());
}

#[test]
fn a_request_in_which_nothing_can_move_exits_1() {
    let project = Project::empty();
    project.write("notes.txt", "x\n");
    let output = dry_run(&project, "notes.txt", "more/notes.txt");
    assert!(!output.status.success());
    let text = stderr_text(&output);
    assert!(text.contains("Dry run: nothing was changed"), "{text}");
    assert!(text.contains("Nothing would be moved"), "{text}");
    assert!(project.exists("notes.txt"));
}

#[test]
fn a_missing_source_is_refused_before_any_tool_starts() {
    let project = typescript_project();
    let output = dry_run(&project, "src/missing.ts", "src/c.ts");
    assert!(!output.status.success());
    assert!(stdout_text(&output).is_empty());
}

#[test]
fn the_error_of_a_json_dry_run_is_json_on_stderr() {
    let project = typescript_project();
    let output = run_cli(&[
        "move",
        "--dry-run",
        "--json",
        "--project-path",
        project.path().to_str().unwrap(),
        "--source-path",
        "src/a.ts",
        "--target-path",
        "src/b.ts",
    ]);
    assert!(!output.status.success());
    let error: serde_json::Value = serde_json::from_str(&stderr_text(&output)).unwrap();
    assert_eq!(error["status"], "error");
}
