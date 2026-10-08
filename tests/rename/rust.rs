use crate::common;

// Real rust-analyzer against tests/fixtures/rust/rename_crate (package shop).
//   src/shapes.rs: line 4 trait method area, 12 struct Rect, 13 field width, 36
//   variant Square, 50 and 58 two locals named `total`.
//   src/report.rs: macro describe! calls .area() inside its definition.
// Run with rust-analyzer installed (`refac doctor rust` explains how):
//   cargo test --test rename rust:: -- --ignored

use common::lsp::{assert_unchanged, at_line, refused, rename, request, require_server, snapshot};
use std::path::Path;
use std::process::Command;

const SHAPES: &str = "src/shapes.rs";

/// The project is copied to a temporary folder, where rustup would pick the
/// default toolchain; the pin of this repository makes it pick one that has
/// rust-analyzer.
fn setup() -> tempfile::TempDir {
    let project = common::setup_fixture("rust/rename_crate");
    let pin = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("rust-toolchain.toml"))
        .expect("the repository pins its toolchain");
    std::fs::write(project.path().join("rust-toolchain.toml"), pin).unwrap();
    require_server_in(project.path());
    project
}

fn require_server_in(project: &Path) {
    let server = refac::servers::for_language("rust").unwrap();
    if let Err(not_found) = refac::servers::locate(server, project) {
        panic!("{not_found}");
    }
    require_server("rust");
}

/// The judge of a refactor: every target still compiles.
fn assert_compiles(project: &Path) {
    let output = Command::new("cargo")
        .args(["check", "--all-targets", "--offline"])
        .current_dir(project)
        .output()
        .expect("failed to run cargo");
    assert!(
        output.status.success(),
        "cargo check failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
#[ignore = "needs rust-analyzer (run `refac doctor rust`)"]
async fn a_trait_method_is_renamed_in_impls_callers_and_tests() {
    let project = setup();

    let report = rename(at_line(
        request(project.path(), SHAPES, "area", "surface"),
        4,
    ))
    .await;

    assert_eq!(report.files.len(), 3, "{:?}", report.files);
    let shapes = common::read_file(project.path(), SHAPES);
    assert!(shapes.contains("fn surface(&self) -> f64;"), "{shapes}");
    assert!(shapes.contains("shape.surface()"), "{shapes}");
    assert!(common::read_file(project.path(), "tests/report.rs").contains("rect.surface()"));

    // rust-analyzer never edits a macro_rules! body: the call inside the macro
    // definition is left behind, and refac says so loudly.
    let attention = report
        .notes
        .iter()
        .find(|note| note.starts_with("ATTENTION"))
        .unwrap_or_else(|| panic!("no attention note in {:?}", report.notes));
    assert!(attention.contains("src/report.rs:5"), "{attention}");
    assert!(attention.contains("macro_rules!"), "{attention}");

    // Once the macro body is edited by hand, the project builds again.
    let path = project.path().join("src/report.rs");
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, text.replace("$shape.area()", "$shape.surface()")).unwrap();
    assert_compiles(project.path());
}

#[tokio::test]
#[ignore = "needs rust-analyzer (run `refac doctor rust`)"]
async fn a_field_with_shorthand_initialisers_is_renamed() {
    let project = setup();

    rename(at_line(request(project.path(), SHAPES, "width", "w"), 13)).await;

    let shapes = common::read_file(project.path(), SHAPES);
    assert!(shapes.contains("pub w: f64,"), "{shapes}");
    // The shorthand `Rect { width, height }` ties the field to the parameter
    // `width`; rust-analyzer renames the parameter along with the field.
    assert!(
        shapes.contains("pub fn new(w: f64, height: f64)"),
        "{shapes}"
    );
    assert!(shapes.contains("Rect { w, height }"), "{shapes}");
    assert_compiles(project.path());
}

#[tokio::test]
#[ignore = "needs rust-analyzer (run `refac doctor rust`)"]
async fn a_struct_is_renamed_through_use_trees_and_re_exports() {
    let project = setup();

    let report = rename(at_line(
        request(project.path(), SHAPES, "Rect", "Rectangle"),
        12,
    ))
    .await;

    assert!(report.files.len() >= 4, "{:?}", report.files);
    assert!(
        common::read_file(project.path(), "src/lib.rs")
            .contains("pub use shapes::{Rectangle, Shape};")
    );
    assert!(
        common::read_file(project.path(), "src/main.rs").contains("use shop::{Rectangle, Shape};")
    );
    assert_compiles(project.path());
}

#[tokio::test]
#[ignore = "needs rust-analyzer (run `refac doctor rust`)"]
async fn an_enum_variant_is_renamed_in_every_match() {
    let project = setup();

    rename(at_line(
        request(project.path(), SHAPES, "Square", "Cube"),
        36,
    ))
    .await;

    assert!(common::read_file(project.path(), "src/report.rs").contains("Kind::Cube =>"));
    assert!(common::read_file(project.path(), "tests/report.rs").contains("Kind::Cube"));
    assert_compiles(project.path());
}

#[tokio::test]
#[ignore = "needs rust-analyzer (run `refac doctor rust`)"]
async fn a_name_shared_by_two_locals_is_listed_and_chosen_by_line() {
    let project = setup();

    let listing = refused(request(project.path(), SHAPES, "total", "sum")).await;
    assert!(listing.contains("several different symbols"), "{listing}");
    assert!(listing.contains("50:"), "{listing}");
    assert!(listing.contains("58:"), "{listing}");

    rename(at_line(request(project.path(), SHAPES, "total", "sum"), 58)).await;
    let shapes = common::read_file(project.path(), SHAPES);
    assert!(
        shapes.contains("let sum = value * 2.0;\n    sum\n"),
        "{shapes}"
    );
    assert!(shapes.contains("let mut total = 0.0;"), "{shapes}");
    assert_compiles(project.path());
}

#[tokio::test]
#[ignore = "needs rust-analyzer (run `refac doctor rust`)"]
async fn a_name_that_would_capture_another_binding_is_refused_and_nothing_changes() {
    let project = setup();
    let before = snapshot(project.path());

    // In `total_area` the parameter `shapes` is used on line 51; a local
    // named `shapes` would shadow it.
    let error = refused(at_line(
        request(project.path(), SHAPES, "total", "shapes"),
        50,
    ))
    .await;

    assert!(error.contains("not faithful"), "{error}");
    assert!(error.contains("newly refer"), "{error}");
    assert_unchanged(project.path(), &before);
}

#[tokio::test]
#[ignore = "needs rust-analyzer (run `refac doctor rust`)"]
async fn renaming_a_module_is_refused_and_points_to_move_module() {
    let project = setup();
    let before = snapshot(project.path());

    let error = refused(at_line(
        request(project.path(), "src/lib.rs", "shapes", "figures"),
        4,
    ))
    .await;

    assert!(error.contains("refac move-module"), "{error}");
    assert_unchanged(project.path(), &before);
}

#[tokio::test]
#[ignore = "needs rust-analyzer (run `refac doctor rust`)"]
async fn a_dry_run_plans_and_verifies_but_writes_nothing() {
    let project = setup();
    let before = snapshot(project.path());

    let mut dry = at_line(request(project.path(), SHAPES, "Square", "Cube"), 36);
    dry.dry_run = true;
    let report = rename(dry).await;

    assert!(report.dry_run);
    assert_eq!(report.edits, 4, "{:?}", report.files);
    assert_unchanged(project.path(), &before);
}
