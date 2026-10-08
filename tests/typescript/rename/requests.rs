//! The request itself: early rejections and the dry run.

use super::{assert_refused_untouched, assert_succeeded, rename, snapshot};
use crate::common;

#[test]
fn invalid_requests_are_rejected_early() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    let before = snapshot(project);
    for (new_name, expected) in [
        ("class", "reserved word"),
        ("2fast", "not a valid identifier"),
        ("total", "equals the current name"),
    ] {
        let output = rename(
            project,
            "src/lib/util.ts",
            "total",
            new_name,
            &["--line", "1"],
        );
        assert_refused_untouched(project, &before, &output, expected);
    }
    let output = rename(
        project,
        "src/lib/util.ts",
        "total",
        "x",
        &["--column", "14"],
    );
    assert!(!output.status.success(), "--column needs --line");
    let output = rename(project, "tsconfig.json", "compilerOptions", "x", &[]);
    assert_refused_untouched(
        project,
        &before,
        &output,
        "Python (.py) and Dart (.dart) files",
    );
}

#[test]
fn dry_run_reports_json_and_writes_nothing() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    let before = snapshot(project);
    let output = rename(
        project,
        "src/lib/util.ts",
        "total",
        "grandTotal",
        &["--line", "1", "--dry-run", "--json"],
    );
    assert_succeeded(&output);
    assert_eq!(snapshot(project), before, "dry run must not write");

    let json: serde_json::Value = serde_json::from_str(&common::stdout_text(&output)).unwrap();
    assert_eq!(json["status"], "ok");
    assert_eq!(json["operation"], "rename");
    assert_eq!(json["dry_run"], true);
    assert_eq!(json["edits"], 9);
    assert_eq!(json["edited_files"], 4);
}

#[test]
fn dry_run_text_says_nothing_changed_and_how_to_apply_it() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    let before = snapshot(project);
    let output = rename(
        project,
        "src/lib/util.ts",
        "total",
        "grandTotal",
        &["--line", "1", "--dry-run"],
    );
    assert_succeeded(&output);
    assert_eq!(snapshot(project), before, "dry run must not write");

    let text = common::stdout_text(&output);
    assert!(
        text.contains("// Dry run: nothing was changed. Planned and verified rename:"),
        "{text}"
    );
    assert!(text.contains("total -> grandTotal"), "{text}");
    assert!(
        text.contains("// Run the same command without --dry-run to write these edits."),
        "{text}"
    );
}
