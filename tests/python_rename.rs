#[allow(dead_code)]
mod common;

// Real basedpyright against tests/fixtures/python/rename_project (package shop).
//   shop/shapes.py: line 9 class Shape, 12 Shape.area, 17 class Rect, 18 field
//   width, 21 Rect.area, 29 Circle.area, 33 total_area (parameter scale), 34 and
//   41 two locals named `total`.
//   shop/report.py: line 15 calls `.area()` on an untyped parameter.
// The judge of a refactor is the program: app.py and checks/test_shapes.py run
// before and after, with the same output.
// Run with basedpyright installed (`refac doctor python` explains how):
//   cargo test --test python_rename -- --ignored

use common::lsp::{assert_unchanged, at_line, refused, rename, request, require_server, snapshot};
use std::path::Path;
use std::process::Command;

const SHAPES: &str = "shop/shapes.py";
const REPORT: &str = "shop/report.py";

fn setup() -> tempfile::TempDir {
    require_server("python");
    common::setup_fixture("python/rename_project")
}

fn python(project: &Path, args: &[&str]) -> (bool, String) {
    let output = Command::new("python3")
        .args(args)
        .current_dir(project)
        .output()
        .expect("failed to run python3");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (output.status.success(), text)
}

/// The program still behaves as before: the script prints the same lines and
/// the checks pass.
fn assert_program_unchanged(project: &Path, expected_output: &str) {
    let (ok, output) = python(project, &["app.py"]);
    assert!(ok, "app.py failed:\n{output}");
    assert_eq!(output, expected_output);
    let (ok, output) = python(project, &["-m", "checks.test_shapes"]);
    assert!(ok, "the checks failed:\n{output}");
}

fn original_output() -> String {
    let project = common::setup_fixture("python/rename_project");
    python(project.path(), &["app.py"]).1
}

#[tokio::test]
#[ignore = "needs basedpyright (run `refac doctor python`)"]
async fn a_base_method_is_renamed_with_its_overrides_and_typed_callers() {
    let expected = original_output();
    let project = setup();

    let report = rename(at_line(
        request(project.path(), SHAPES, "area", "surface"),
        12,
    ))
    .await;

    // Shape.area, Rect.area, Circle.area and the two typed calls in shapes.py,
    // and the typed call in report.py.
    assert_eq!(report.files.len(), 2, "{:?}", report.files);
    let shapes = common::read_file(project.path(), SHAPES);
    assert_eq!(shapes.matches("def surface(self)").count(), 3, "{shapes}");
    assert!(shapes.contains("total += shape.surface()"), "{shapes}");
    assert!(common::read_file(project.path(), REPORT).contains("{shape.surface():.2f}"));

    // The call on an untyped parameter is not provable. refac does not touch
    // it and says where it is.
    let note = report
        .notes
        .iter()
        .find(|note| note.contains("is still written"))
        .unwrap_or_else(|| panic!("no leftover note in {:?}", report.notes));
    assert!(note.contains("shop/report.py:15"), "{note}");
    let (ok, output) = python(project.path(), &["-m", "checks.test_shapes"]);
    assert!(
        !ok,
        "the untyped call should be broken until fixed:\n{output}"
    );

    // Once that call is changed by hand, the program is as it was.
    let path = project.path().join(REPORT);
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, text.replace("thing.area()", "thing.surface()")).unwrap();
    assert_program_unchanged(project.path(), &expected);
}

#[tokio::test]
#[ignore = "needs basedpyright (run `refac doctor python`)"]
async fn a_function_is_renamed_through_imports_and_the_all_list() {
    let expected = original_output();
    let project = setup();

    let report = rename(at_line(
        request(project.path(), SHAPES, "total_area", "combined_area"),
        33,
    ))
    .await;

    assert!(report.files.len() >= 4, "{:?}", report.files);
    let init = common::read_file(project.path(), "shop/__init__.py");
    assert!(
        init.contains("from .shapes import Circle, Rect, Shape, combined_area"),
        "{init}"
    );
    assert!(init.contains("\"combined_area\""), "{init}");
    assert!(common::read_file(project.path(), "app.py").contains("combined_area(figures"));
    assert_program_unchanged(project.path(), &expected);
}

#[tokio::test]
#[ignore = "needs basedpyright (run `refac doctor python`)"]
async fn a_dataclass_field_is_renamed_with_its_keyword_arguments() {
    let expected = original_output();
    let project = setup();

    rename(at_line(
        request(project.path(), SHAPES, "width", "breadth"),
        18,
    ))
    .await;

    assert!(common::read_file(project.path(), SHAPES).contains("self.breadth * self.height"));
    assert!(common::read_file(project.path(), REPORT).contains("Rect(breadth=unit()"));
    assert!(common::read_file(project.path(), "checks/test_shapes.py").contains("Rect(breadth=3"));
    assert_program_unchanged(project.path(), &expected);
}

#[tokio::test]
#[ignore = "needs basedpyright (run `refac doctor python`)"]
async fn a_class_is_renamed_through_imports_and_annotations() {
    let expected = original_output();
    let project = setup();

    rename(at_line(
        request(project.path(), SHAPES, "Rect", "Rectangle"),
        17,
    ))
    .await;

    assert!(common::read_file(project.path(), REPORT).contains("import Rectangle, Shape"));
    assert!(common::read_file(project.path(), "app.py").contains("Rectangle(2, 3)"));
    // The script prints class names (`type(shape).__name__`), so its output
    // changes by exactly the new name: a rename cannot know about reflection.
    assert_program_unchanged(project.path(), &expected.replace("Rect ", "Rectangle "));
}

#[tokio::test]
#[ignore = "needs basedpyright (run `refac doctor python`)"]
async fn a_parameter_is_renamed_with_the_keyword_at_every_call() {
    let expected = original_output();
    let project = setup();

    rename(at_line(
        request(project.path(), SHAPES, "scale", "factor"),
        33,
    ))
    .await;

    assert!(common::read_file(project.path(), SHAPES).contains("return total * factor"));
    assert!(common::read_file(project.path(), REPORT).contains("total_area(items, factor=2.0)"));
    assert!(common::read_file(project.path(), "app.py").contains("factor=0.5"));
    assert_program_unchanged(project.path(), &expected);
}

#[tokio::test]
#[ignore = "needs basedpyright (run `refac doctor python`)"]
async fn a_name_shared_by_two_locals_is_listed_and_chosen_by_line() {
    let project = setup();

    let listing = refused(request(project.path(), SHAPES, "total", "subtotal")).await;
    assert!(listing.contains("several different symbols"), "{listing}");
    assert!(listing.contains("34:"), "{listing}");
    assert!(listing.contains("41:"), "{listing}");

    rename(at_line(
        request(project.path(), SHAPES, "total", "subtotal"),
        41,
    ))
    .await;
    let shapes = common::read_file(project.path(), SHAPES);
    assert!(shapes.contains("subtotal = max(shapes"), "{shapes}");
    assert!(shapes.contains("return subtotal\n"), "{shapes}");
    assert!(shapes.contains("total = 0.0"), "{shapes}");
}

#[tokio::test]
#[ignore = "needs basedpyright (run `refac doctor python`)"]
async fn a_name_that_would_capture_another_binding_is_refused_and_nothing_changes() {
    let project = setup();
    let before = snapshot(project.path());

    // In `total_area` the parameter `shapes` is iterated; a local named
    // `shapes` would replace it.
    let error = refused(at_line(
        request(project.path(), SHAPES, "total", "shapes"),
        34,
    ))
    .await;

    assert!(error.contains("not faithful"), "{error}");
    assert_unchanged(project.path(), &before);
}

/// Renaming from an override cannot find the base method; the note shows the
/// other definitions that are left, so the caller can rename them too.
#[tokio::test]
#[ignore = "needs basedpyright (run `refac doctor python`)"]
async fn an_override_renamed_alone_leaves_the_family_and_says_so() {
    let project = setup();

    let report = rename(at_line(
        request(project.path(), SHAPES, "area", "surface"),
        21,
    ))
    .await;

    assert_eq!(report.files.len(), 1, "{:?}", report.files);
    let note = report
        .notes
        .iter()
        .find(|note| note.contains("is still written"))
        .unwrap_or_else(|| panic!("no leftover note in {:?}", report.notes));
    assert!(note.contains("shop/shapes.py:12"), "{note}");
    assert!(note.contains("shop/shapes.py:29"), "{note}");
}

#[tokio::test]
#[ignore = "needs basedpyright (run `refac doctor python`)"]
async fn renaming_a_module_is_refused_and_points_to_move() {
    let project = setup();
    let before = snapshot(project.path());

    let error = refused(at_line(
        request(project.path(), "app.py", "report", "reporting"),
        2,
    ))
    .await;

    assert!(error.contains("not a renameable symbol"), "{error}");
    assert!(error.contains("refac move"), "{error}");
    assert_unchanged(project.path(), &before);
}

#[tokio::test]
#[ignore = "needs basedpyright (run `refac doctor python`)"]
async fn a_dry_run_plans_and_verifies_but_writes_nothing() {
    let project = setup();
    let before = snapshot(project.path());

    let mut dry = at_line(request(project.path(), SHAPES, "area", "surface"), 12);
    dry.dry_run = true;
    let report = rename(dry).await;

    assert!(report.dry_run);
    assert_eq!(report.edits, 6, "{:?}", report.files);
    assert_unchanged(project.path(), &before);
}
