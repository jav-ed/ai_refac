//! Choosing the symbol: ambiguity, line and column, collisions, and what is not a symbol.

use super::{assert_refused_untouched, assert_succeeded, assert_typechecks, rename, snapshot};
use crate::common;

#[test]
fn ambiguous_name_lists_candidates_and_changes_nothing() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    let before = snapshot(project);
    let output = rename(project, "src/lib/util.ts", "total", "grandTotal", &[]);

    assert_refused_untouched(project, &before, &output, "several different symbols");
    let stderr = common::stderr_text(&output);
    assert!(
        stderr.contains("1:14") && stderr.contains("3:11"),
        "both candidates listed:\n{stderr}"
    );
}

#[tokio::test]
async fn line_selects_the_shadowing_local_only() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    assert_succeeded(&rename(
        project,
        "src/lib/util.ts",
        "total",
        "sumTotal",
        &["--line", "3"],
    ));

    let util = common::read_file(project, "src/lib/util.ts");
    assert!(
        util.contains("export const total = 10;"),
        "module-level total untouched:\n{util}"
    );
    assert!(
        util.contains("const sumTotal = items.reduce") && util.contains("return sumTotal;"),
        "{util}"
    );
    assert!(
        common::read_file(project, "src/a.ts").contains("import { total,"),
        "importers untouched"
    );
    assert_typechecks(project).await;
}

#[test]
fn column_narrows_a_line_with_two_symbols() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    let before = snapshot(project);
    // Column 8 is not an occurrence of `total` on line 3, so nothing matches.
    let output = rename(
        project,
        "src/lib/util.ts",
        "total",
        "x",
        &["--line", "3", "--column", "8"],
    );
    assert_refused_untouched(
        project,
        &before,
        &output,
        "does not appear as an identifier at 3:8",
    );
}

#[test]
fn name_collision_is_refused_before_any_write() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    let before = snapshot(project);
    let output = rename(
        project,
        "src/lib/util.ts",
        "total",
        "computeSum",
        &["--line", "1"],
    );
    assert_refused_untouched(project, &before, &output, "not faithful");
}

#[test]
fn shadow_capture_that_still_typechecks_is_refused() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    let before = snapshot(project);
    // `inner` -> `outer` compiles (both numbers) but makes `outer` mean the local.
    let output = rename(project, "src/shadow.ts", "inner", "outer", &[]);
    assert_refused_untouched(project, &before, &output, "not faithful");
}

#[test]
fn standard_library_symbols_are_refused() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    let before = snapshot(project);
    let output = rename(project, "src/lib/util.ts", "reduce", "fold", &[]);
    assert_refused_untouched(project, &before, &output, "standard TypeScript library");
}

#[test]
fn text_inside_strings_is_not_a_symbol() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    let before = snapshot(project);
    let output = rename(project, "src/a.ts", "string", "text", &["--line", "13"]);
    assert_refused_untouched(project, &before, &output, "Cannot rename");
}
