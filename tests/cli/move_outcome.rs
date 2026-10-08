//! What a `move` says and how it exits when only part of the request could be
//! carried out. Needs no language server: Markdown is built in and a Rust file
//! moved across modules is refused before anything starts.

use crate::common;
use std::fs;

/// A Markdown file moves, the Rust file next to it is refused: the Markdown file
/// stays moved, the answer says which group failed, and the exit code is not 0.
#[test]
fn a_failed_group_next_to_a_moved_one_exits_non_zero_with_the_whole_report() {
    let temp = common::setup_fixture("rust/project");
    let project = temp.path();
    fs::write(project.join("notes.md"), "# Notes\n").unwrap();
    let rust_before = common::read_file(project, "src/types.rs");

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        "notes.md",
        "--source-path",
        "src/types.rs",
        "--target-path",
        "docs/notes.md",
        "--target-path",
        "src/shared/types.rs",
    ]);

    assert!(!output.status.success(), "{}", common::stdout_text(&output));
    let error = common::stderr_text(&output);
    assert!(
        error.contains("// Partly done: 1 requested path moved, 1 failed."),
        "{error}"
    );
    assert!(error.contains("// Markdown results:"), "{error}");
    assert!(error.contains("notes.md -> docs/notes.md"), "{error}");
    assert!(error.contains("// Failed:"), "{error}");
    assert!(error.contains("move-module"), "{error}");
    assert!(project.join("docs/notes.md").exists());
    assert!(!project.join("notes.md").exists());
    assert_eq!(common::read_file(project, "src/types.rs"), rust_before);
}

/// The same report is the `error` of the JSON document, so a script sees it too.
#[test]
fn the_partial_report_is_the_json_error() {
    let temp = common::setup_fixture("rust/project");
    let project = temp.path();

    let output = common::run_cli(&[
        "move",
        "--json",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        "src/types.rs",
        "--target-path",
        "src/shared/types.rs",
    ]);

    assert!(!output.status.success());
    let document: serde_json::Value =
        serde_json::from_str(&common::stderr_text(&output)).expect("stderr is one JSON document");
    assert_eq!(document["status"], "error");
    assert!(
        document["error"]
            .as_str()
            .unwrap()
            .contains("// Nothing was moved."),
        "{document}"
    );
}

/// A skipped file next to a moved one is reported, and the move succeeds.
#[test]
fn a_skipped_file_next_to_a_moved_one_still_succeeds() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path();
    fs::write(project.join("notes.md"), "# Notes\n").unwrap();
    fs::write(project.join("data.xyz"), "").unwrap();

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        "notes.md",
        "--source-path",
        "data.xyz",
        "--target-path",
        "docs/notes.md",
        "--target-path",
        "docs/data.xyz",
    ]);

    common::assert_move_succeeded(&output);
    let text = common::stdout_text(&output);
    assert!(text.contains("Skipped (unsupported extension)"), "{text}");
    assert!(project.join("docs/notes.md").exists());
    assert!(project.join("data.xyz").exists());
}
