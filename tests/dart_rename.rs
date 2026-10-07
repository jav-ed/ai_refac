#[allow(dead_code)]
mod common;

// Real Dart analysis server against tests/fixtures/dart/rename_package
// (package shop; `dart pub get` runs in the temporary copy).
//   lib/shapes.dart: line 6 class Shape, 7 Shape.area, 10 class Rect, 13 field
//   width, 17 Rect.area, 26 Circle.area, 29 totalArea (named parameter scale),
//   30 and 38 two locals named `total`.
//   lib/report.dart: line 9 calls `.area()` on a `dynamic` parameter.
// The judge of a refactor is the program: bin/main.dart and
// test/report_test.dart run before and after.
// Run with the Dart SDK installed (`refac doctor dart` explains how):
//   cargo test --test dart_rename -- --ignored

use common::lsp::{assert_unchanged, at_line, refused, rename, request, require_server, snapshot};
use std::path::Path;
use std::process::Command;

const SHAPES: &str = "lib/shapes.dart";
const REPORT: &str = "lib/report.dart";

fn pub_get(project: &Path) {
    let output = Command::new("dart")
        .args(["pub", "get", "--offline"])
        .current_dir(project)
        .output()
        .expect("failed to run dart");
    assert!(
        output.status.success(),
        "dart pub get failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn setup() -> tempfile::TempDir {
    require_server("dart");
    let project = common::setup_fixture("dart/rename_package");
    pub_get(project.path());
    project
}

fn dart(project: &Path, script: &str) -> (bool, String) {
    let output = Command::new("dart")
        .args(["run", script])
        .current_dir(project)
        .output()
        .expect("failed to run dart");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (output.status.success(), text)
}

/// `dart analyze` finds nothing to complain about.
fn assert_analyzes_clean(project: &Path) {
    let output = Command::new("dart")
        .args(["analyze", "--fatal-infos"])
        .current_dir(project)
        .output()
        .expect("failed to run dart");
    assert!(
        output.status.success(),
        "dart analyze failed:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

/// The program still behaves as before: the script prints the same lines and
/// the checks pass.
fn assert_program_unchanged(project: &Path, expected_output: &str) {
    let (ok, output) = dart(project, "bin/main.dart");
    assert!(ok, "main.dart failed:\n{output}");
    assert_eq!(output, expected_output);
    let (ok, output) = dart(project, "test/report_test.dart");
    assert!(ok, "the checks failed:\n{output}");
}

fn original_output() -> String {
    let project = setup();
    dart(project.path(), "bin/main.dart").1
}

#[tokio::test]
#[ignore = "needs the Dart SDK (run `refac doctor dart`)"]
async fn a_method_is_renamed_with_its_overrides_and_the_doc_comment_link() {
    let expected = original_output();
    let project = setup();

    let report = rename(at_line(
        request(project.path(), SHAPES, "area", "surface"),
        7,
    ))
    .await;

    assert_eq!(report.files.len(), 2, "{:?}", report.files);
    let shapes = common::read_file(project.path(), SHAPES);
    assert_eq!(shapes.matches("double surface()").count(), 3, "{shapes}");
    assert!(shapes.contains("double surface() => width"), "{shapes}");
    assert!(shapes.contains("[Shape.surface]"), "{shapes}");
    assert!(
        shapes.contains("shape.surface() > total.surface()"),
        "{shapes}"
    );
    assert!(common::read_file(project.path(), REPORT).contains("shape.surface().toStringAsFixed"));

    // The call on a `dynamic` value is not provable. refac does not touch it
    // and says where it is.
    let note = report
        .notes
        .iter()
        .find(|note| note.contains("is still written"))
        .unwrap_or_else(|| panic!("no leftover note in {:?}", report.notes));
    assert!(note.contains("lib/report.dart:9"), "{note}");
    let (ok, _) = dart(project.path(), "test/report_test.dart");
    assert!(!ok, "the dynamic call should be broken until fixed");

    let path = project.path().join(REPORT);
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, text.replace("thing.area()", "thing.surface()")).unwrap();
    assert_analyzes_clean(project.path());
    assert_program_unchanged(project.path(), &expected);
}

#[tokio::test]
#[ignore = "needs the Dart SDK (run `refac doctor dart`)"]
async fn a_function_is_renamed_through_exports_with_show() {
    let expected = original_output();
    let project = setup();

    let report = rename(at_line(
        request(project.path(), SHAPES, "totalArea", "combinedArea"),
        29,
    ))
    .await;

    assert!(report.files.len() >= 4, "{:?}", report.files);
    let barrel = common::read_file(project.path(), "lib/shop.dart");
    assert!(
        barrel.contains("show Circle, Rect, Shape, combinedArea;"),
        "{barrel}"
    );
    assert!(common::read_file(project.path(), "bin/main.dart").contains("combinedArea(figures"));
    assert_analyzes_clean(project.path());
    assert_program_unchanged(project.path(), &expected);
}

#[tokio::test]
#[ignore = "needs the Dart SDK (run `refac doctor dart`)"]
async fn a_field_is_renamed_with_its_field_formal_and_named_arguments() {
    let expected = original_output();
    let project = setup();

    rename(at_line(
        request(project.path(), SHAPES, "width", "breadth"),
        13,
    ))
    .await;

    let shapes = common::read_file(project.path(), SHAPES);
    assert!(shapes.contains("required this.breadth"), "{shapes}");
    assert!(common::read_file(project.path(), REPORT).contains("Rect(breadth: unit"));
    assert!(common::read_file(project.path(), "bin/main.dart").contains("Rect(breadth: 2"));
    assert_analyzes_clean(project.path());
    assert_program_unchanged(project.path(), &expected);
}

#[tokio::test]
#[ignore = "needs the Dart SDK (run `refac doctor dart`)"]
async fn a_class_is_renamed_through_exports_and_imports() {
    let expected = original_output();
    let project = setup();

    rename(at_line(
        request(project.path(), SHAPES, "Rect", "Rectangle"),
        10,
    ))
    .await;

    let barrel = common::read_file(project.path(), "lib/shop.dart");
    assert!(barrel.contains("show Circle, Rectangle, Shape"), "{barrel}");
    assert_analyzes_clean(project.path());
    // The script prints class names (`runtimeType`), so its output changes by
    // exactly the new name: a rename cannot know about reflection.
    assert_program_unchanged(project.path(), &expected.replace("Rect ", "Rectangle "));
}

#[tokio::test]
#[ignore = "needs the Dart SDK (run `refac doctor dart`)"]
async fn a_named_parameter_is_renamed_with_the_label_at_every_call() {
    let expected = original_output();
    let project = setup();

    rename(at_line(
        request(project.path(), SHAPES, "scale", "factor"),
        29,
    ))
    .await;

    assert!(common::read_file(project.path(), SHAPES).contains("{double factor = 1.0}"));
    assert!(!common::read_file(project.path(), REPORT).contains("scale: 2.0"));
    assert!(common::read_file(project.path(), REPORT).contains("factor: 2.0"));
    assert!(common::read_file(project.path(), "bin/main.dart").contains("factor: 0.5"));
    assert_analyzes_clean(project.path());
    assert_program_unchanged(project.path(), &expected);
}

#[tokio::test]
#[ignore = "needs the Dart SDK (run `refac doctor dart`)"]
async fn a_name_shared_by_two_locals_is_listed_and_chosen_by_line() {
    let project = setup();

    let listing = refused(request(project.path(), SHAPES, "total", "subtotal")).await;
    assert!(listing.contains("several different symbols"), "{listing}");
    assert!(listing.contains("30:"), "{listing}");
    assert!(listing.contains("38:"), "{listing}");

    rename(at_line(
        request(project.path(), SHAPES, "total", "subtotal"),
        38,
    ))
    .await;
    let shapes = common::read_file(project.path(), SHAPES);
    assert!(shapes.contains("var subtotal = shapes.first;"), "{shapes}");
    assert!(shapes.contains("return subtotal;"), "{shapes}");
    assert!(shapes.contains("var total = 0.0;"), "{shapes}");
    assert_analyzes_clean(project.path());
}

#[tokio::test]
#[ignore = "needs the Dart SDK (run `refac doctor dart`)"]
async fn a_name_that_would_capture_another_binding_is_refused_and_nothing_changes() {
    let project = setup();
    let before = snapshot(project.path());

    // In `totalArea` the parameter `shapes` is iterated; a local named
    // `shapes` would redeclare it.
    let error = refused(at_line(
        request(project.path(), SHAPES, "total", "shapes"),
        30,
    ))
    .await;

    assert!(!error.is_empty());
    assert_unchanged(project.path(), &before);
}

#[tokio::test]
#[ignore = "needs the Dart SDK (run `refac doctor dart`)"]
async fn a_dry_run_plans_and_verifies_but_writes_nothing() {
    let project = setup();
    let before = snapshot(project.path());

    let mut dry = at_line(request(project.path(), SHAPES, "area", "surface"), 7);
    dry.dry_run = true;
    let report = rename(dry).await;

    assert!(report.dry_run);
    assert_eq!(report.edits, 8, "{:?}", report.files);
    assert_unchanged(project.path(), &before);
}

/// Needs no server: the project is refused before one is looked for.
#[tokio::test]
async fn a_project_without_a_package_config_is_refused_with_the_command_that_fixes_it() {
    let project = common::setup_fixture("dart/rename_package");

    let error = refused(at_line(
        request(project.path(), SHAPES, "area", "surface"),
        7,
    ))
    .await;

    assert!(error.contains("dart pub get"), "{error}");
    assert!(error.contains("package_config.json"), "{error}");
}
