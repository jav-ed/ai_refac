use crate::common;

// Real gopls against tests/fixtures/go/rename_module (module example.com/shop).
//   shape/shape.go: line 8 interface method Area, 13 fields W and H, 17 method
//   Rect.Area, 20 func NewRect, 23 var Count, 33 scale(n, factor), 35 double(n).
// Run with gopls installed (`refac doctor go` explains how):
//   cargo test --test rename go:: -- --ignored

use common::lsp::{assert_unchanged, at_line, refused, rename, request, require_server, snapshot};
use std::path::Path;
use std::process::Command;

const SHAPE: &str = "shape/shape.go";

fn setup() -> tempfile::TempDir {
    require_server("go");
    common::setup_fixture("go/rename_module")
}

/// The judge of a refactor: the module still builds and passes vet.
fn assert_builds(project: &Path) {
    for args in [["build", "./..."], ["vet", "./..."]] {
        let output = Command::new("go")
            .args(args)
            .current_dir(project)
            .output()
            .expect("failed to run go");
        assert!(
            output.status.success(),
            "go {} failed:\n{}{}",
            args[0],
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[tokio::test]
#[ignore = "needs gopls (run `refac doctor go`)"]
async fn an_interface_method_is_renamed_with_its_implementations_and_callers() {
    let project = setup();

    let report = rename(at_line(
        request(project.path(), SHAPE, "Area", "Surface"),
        8,
    ))
    .await;

    assert_eq!(report.files.len(), 4, "{:?}", report.files);
    let shape = common::read_file(project.path(), SHAPE);
    assert!(shape.contains("\tSurface() float64"), "{shape}");
    assert!(shape.contains("func (r Rect) Surface() float64"), "{shape}");
    // gopls also rewrites the doc comment that starts with the name.
    assert!(
        shape.contains("// Surface is the surface of the Rect."),
        "{shape}"
    );
    // A string and an unrelated comment keep the old word.
    assert!(shape.contains("\"Area=%v total=%v\""), "{shape}");
    assert!(common::read_file(project.path(), "cmd/app/main.go").contains("r.Surface()"),);
    assert!(common::read_file(project.path(), "shape/shape_test.go").contains("r.Surface()"),);
    assert!(
        report
            .notes
            .iter()
            .any(|note| note.contains("still written")),
        "{:?}",
        report.notes
    );
    assert_builds(project.path());
}

#[tokio::test]
#[ignore = "needs gopls (run `refac doctor go`)"]
async fn a_method_that_would_stop_implementing_its_interface_is_refused() {
    let project = setup();
    let before = snapshot(project.path());

    let error = refused(at_line(
        request(project.path(), SHAPE, "Area", "Surface"),
        17,
    ))
    .await;

    assert!(
        error.contains("no longer assignable to interface"),
        "{error}"
    );
    assert_unchanged(project.path(), &before);
}

#[tokio::test]
#[ignore = "needs gopls (run `refac doctor go`)"]
async fn a_function_is_renamed_in_every_package_that_calls_it() {
    let project = setup();

    let report = rename(request(project.path(), SHAPE, "NewRect", "MakeRect")).await;

    assert_eq!(report.edits, 4, "{:?}", report.files);
    assert!(common::read_file(project.path(), "cmd/app/main.go").contains("shape.MakeRect(3, 4)"));
    assert!(
        common::read_file(project.path(), "shape/shape_test.go").contains("r := MakeRect(2, 3)")
    );
    assert_builds(project.path());
}

#[tokio::test]
#[ignore = "needs gopls (run `refac doctor go`)"]
async fn a_variable_and_a_struct_field_are_renamed_across_packages() {
    let project = setup();

    rename(request(project.path(), SHAPE, "Count", "Total")).await;
    rename(at_line(request(project.path(), SHAPE, "W", "Width"), 13)).await;

    assert!(common::read_file(project.path(), "cmd/app/main.go").contains("shape.Total"));
    let shape = common::read_file(project.path(), SHAPE);
    assert!(shape.contains("Width, H float64"), "{shape}");
    assert!(shape.contains("Rect{Width: w, H: h}"), "{shape}");
    assert_builds(project.path());
}

#[tokio::test]
#[ignore = "needs gopls (run `refac doctor go`)"]
async fn a_name_shared_by_several_symbols_is_listed_and_chosen_by_line() {
    let project = setup();

    let listing = refused(request(project.path(), SHAPE, "n", "count")).await;
    assert!(listing.contains("several different symbols"), "{listing}");
    assert!(listing.contains("33:"), "{listing}");
    assert!(listing.contains("35:"), "{listing}");

    // The parameter of `scale` only.
    rename(at_line(request(project.path(), SHAPE, "n", "count"), 33)).await;
    let shape = common::read_file(project.path(), SHAPE);
    assert!(
        shape.contains(
            "func scale(count float64, factor float64) float64 { return count * factor }"
        ),
        "{shape}"
    );
    assert!(shape.contains("func double(n float64)"), "{shape}");
    assert_builds(project.path());
}

#[tokio::test]
#[ignore = "needs gopls (run `refac doctor go`)"]
async fn a_name_that_collides_in_scope_is_refused_and_nothing_changes() {
    let project = setup();
    let before = snapshot(project.path());

    // `Describe` already exists in the package.
    let error = refused(request(project.path(), SHAPE, "NewRect", "Describe")).await;

    assert!(
        error.contains("Describe")
            && (error.contains("conflict")
                || error.contains("already")
                || error.contains("redeclared")),
        "{error}"
    );
    assert_unchanged(project.path(), &before);
}

#[tokio::test]
#[ignore = "needs gopls (run `refac doctor go`)"]
async fn renaming_a_package_is_refused_and_points_to_move() {
    let project = setup();
    let before = snapshot(project.path());

    let error = refused(at_line(
        request(project.path(), SHAPE, "shape", "geometry"),
        2,
    ))
    .await;

    assert!(error.contains("refac move"), "{error}");
    assert_unchanged(project.path(), &before);
}

#[tokio::test]
#[ignore = "needs gopls (run `refac doctor go`)"]
async fn a_dry_run_plans_and_verifies_but_writes_nothing() {
    let project = setup();
    let before = snapshot(project.path());

    let mut dry = request(project.path(), SHAPE, "NewRect", "MakeRect");
    dry.dry_run = true;
    let report = rename(dry).await;

    assert!(report.dry_run);
    assert_eq!(report.edits, 4);
    assert_unchanged(project.path(), &before);
}

#[tokio::test]
#[ignore = "needs gopls (run `refac doctor go`)"]
async fn text_that_is_not_a_symbol_is_refused() {
    let project = setup();
    let before = snapshot(project.path());

    // Line 25 is a comment that mentions Area; the word is no symbol there.
    let error = refused(at_line(
        request(project.path(), SHAPE, "Area", "Surface"),
        25,
    ))
    .await;

    assert!(error.contains("Cannot rename"), "{error}");
    assert_unchanged(project.path(), &before);
}
