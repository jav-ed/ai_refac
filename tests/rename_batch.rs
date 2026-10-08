#[allow(dead_code)]
mod common;

// Several renames in one language-server session (`refac rename --batch`)
// against the real servers. Each language renames three symbols of the rename
// fixture, one of them twice (the second time by the name the first gave it) and
// one in a second file that an earlier rename has already edited, and the
// program must still build or run as before. A second test per language
// puts a rename that cannot work after one that can: the whole batch is
// refused and the project is byte for byte what it was.
//
// Run with the servers installed (`refac doctor` explains how):
//   cargo test --test rename_batch -- --ignored --test-threads=1

use common::lsp::{
    assert_unchanged, at_line, batch, refused_batch, request, require_server, snapshot,
};
use refac::drivers::symbol_rename::RenameRequest;
use std::path::Path;
use std::process::Command;

fn run(project: &Path, program: &str, args: &[&str]) -> (bool, String) {
    let output = Command::new(program)
        .args(args)
        .current_dir(project)
        .output()
        .unwrap_or_else(|error| panic!("failed to run {program}: {error}"));
    (
        output.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    )
}

fn assert_runs(project: &Path, program: &str, args: &[&str]) {
    let (ok, output) = run(project, program, args);
    assert!(ok, "{program} {} failed:\n{output}", args.join(" "));
}

/// A rename of a symbol that the file does not contain.
fn missing(project: &Path, file: &str) -> RenameRequest {
    request(project, file, "no_such_symbol", "something_else")
}

// ---- Go: shape/shape.go line 8 Area, 20 NewRect, 23 Count --------------------

const GO_SHAPE: &str = "shape/shape.go";
const GO_REPORT: &str = "internal/report/report.go";

fn go_batch(project: &Path) -> Vec<RenameRequest> {
    vec![
        at_line(request(project, GO_SHAPE, "Area", "Surface"), 8),
        request(project, GO_SHAPE, "NewRect", "MakeRect"),
        request(project, GO_SHAPE, "Count", "Total"),
        // Another package, a file the first rename edited.
        request(project, GO_REPORT, "Lines", "Summarize"),
        // By the name the first rename gave it.
        at_line(request(project, GO_SHAPE, "Surface", "Extent"), 8),
    ]
}

#[tokio::test]
#[ignore = "needs gopls (run `refac doctor go`)"]
async fn go_batch_renames_five_times_in_one_session() {
    require_server("go");
    let project = common::setup_fixture("go/rename_module");
    let root = project.path();

    let reports = batch(go_batch(root)).await;

    assert_eq!(reports.len(), 5);
    let report = common::read_file(root, GO_REPORT);
    assert!(report.contains("func Summarize("), "{report}");
    assert!(report.contains("s.Extent()"), "{report}");
    let main = common::read_file(root, "cmd/app/main.go");
    assert!(main.contains("report.Summarize("), "{main}");
    let shape = common::read_file(root, GO_SHAPE);
    assert!(shape.contains("\tExtent() float64"), "{shape}");
    assert!(shape.contains("func MakeRect("), "{shape}");
    assert!(shape.contains("var Total"), "{shape}");
    assert!(!shape.contains("Area()"), "{shape}");
    assert_runs(root, "go", &["build", "./..."]);
    assert_runs(root, "go", &["vet", "./..."]);
}

#[tokio::test]
#[ignore = "needs gopls (run `refac doctor go`)"]
async fn go_batch_with_a_missing_symbol_changes_nothing() {
    require_server("go");
    let project = common::setup_fixture("go/rename_module");
    let root = project.path();
    let before = snapshot(root);
    let mut requests = go_batch(root);
    requests.insert(3, missing(root, GO_SHAPE));

    let message = refused_batch(requests).await;

    assert!(message.contains("Rename 4 of 6"), "{message}");
    assert!(
        message.contains("3 earlier rename(s) were undone"),
        "{message}"
    );
    assert_unchanged(root, &before);
}

#[tokio::test]
#[ignore = "needs gopls (run `refac doctor go`)"]
async fn go_batch_dry_runs_are_planned_one_independent_of_the_other() {
    require_server("go");
    let project = common::setup_fixture("go/rename_module");
    let root = project.path();
    let before = snapshot(root);
    let mut first = at_line(request(root, GO_SHAPE, "Area", "Surface"), 8);
    let mut second = at_line(request(root, GO_SHAPE, "Area", "Extent"), 8);
    first.dry_run = true;
    second.dry_run = true;

    let reports = batch(vec![first, second]).await;

    // Both find all the places: the second is not planned on the first's text.
    assert_eq!(reports[0].files, reports[1].files);
    assert_eq!(reports[0].edits, reports[1].edits);
    assert_unchanged(root, &before);
}

// ---- Rust: src/shapes.rs line 8 SCALE, 36 variant Square, fn kind_of ---------

const RUST_SHAPES: &str = "src/shapes.rs";
const RUST_REPORT: &str = "src/report.rs";

fn rust_setup() -> tempfile::TempDir {
    let project = common::setup_fixture("rust/rename_crate");
    let pin = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("rust-toolchain.toml"))
        .expect("the repository pins its toolchain");
    std::fs::write(project.path().join("rust-toolchain.toml"), pin).unwrap();
    let server = refac::servers::for_language("rust").unwrap();
    if let Err(not_found) = refac::servers::locate(server, project.path()) {
        panic!("{not_found}");
    }
    project
}

fn rust_batch(project: &Path) -> Vec<RenameRequest> {
    vec![
        request(project, RUST_SHAPES, "SCALE", "FACTOR"),
        at_line(request(project, RUST_SHAPES, "Square", "Cube"), 36),
        request(project, RUST_SHAPES, "kind_of", "classify"),
        // Another file, one the earlier renames edited (`classify`, `Cube`).
        at_line(request(project, RUST_REPORT, "lines", "report_lines"), 9),
        request(project, RUST_SHAPES, "FACTOR", "MULTIPLIER"),
    ]
}

#[tokio::test]
#[ignore = "needs rust-analyzer (run `refac doctor rust`)"]
async fn rust_batch_renames_five_times_in_one_session() {
    let project = rust_setup();
    let root = project.path();

    let reports = batch(rust_batch(root)).await;

    assert_eq!(reports.len(), 5);
    let report = common::read_file(root, RUST_REPORT);
    assert!(report.contains("pub fn report_lines("), "{report}");
    assert!(report.contains("classify(rect)"), "{report}");
    let shapes = common::read_file(root, RUST_SHAPES);
    assert!(shapes.contains("pub const MULTIPLIER: f64"), "{shapes}");
    assert!(shapes.contains("Cube,"), "{shapes}");
    assert!(shapes.contains("pub fn classify("), "{shapes}");
    assert_runs(root, "cargo", &["check", "--all-targets", "--offline"]);
}

#[tokio::test]
#[ignore = "needs rust-analyzer (run `refac doctor rust`)"]
async fn rust_batch_with_a_missing_symbol_changes_nothing() {
    let project = rust_setup();
    let root = project.path();
    let before = snapshot(root);
    let mut requests = rust_batch(root);
    requests.push(missing(root, RUST_SHAPES));

    let message = refused_batch(requests).await;

    assert!(message.contains("Rename 6 of 6"), "{message}");
    assert!(
        message.contains("5 earlier rename(s) were undone"),
        "{message}"
    );
    assert_unchanged(root, &before);
}

// ---- Python: shop/shapes.py line 33 total_area, 40 largest -------------------

const PY_SHAPES: &str = "shop/shapes.py";
const PY_REPORT: &str = "shop/report.py";

fn python_batch(project: &Path) -> Vec<RenameRequest> {
    vec![
        request(project, PY_SHAPES, "total_area", "sum_area"),
        request(project, PY_SHAPES, "largest", "biggest"),
        // Another file, one the first rename edited (it imports `total_area`).
        request(project, PY_REPORT, "summary", "overview"),
        request(project, PY_SHAPES, "sum_area", "grand_total"),
    ]
}

#[tokio::test]
#[ignore = "needs basedpyright (run `refac doctor python`)"]
async fn python_batch_renames_four_times_in_one_session() {
    require_server("python");
    let project = common::setup_fixture("python/rename_project");
    let root = project.path();
    let (_, expected) = run(root, "python3", &["app.py"]);

    let reports = batch(python_batch(root)).await;

    assert_eq!(reports.len(), 4);
    let report = common::read_file(root, PY_REPORT);
    assert!(report.contains("def overview("), "{report}");
    assert!(report.contains("grand_total(items, scale=2.0)"), "{report}");
    let shapes = common::read_file(root, PY_SHAPES);
    assert!(shapes.contains("def grand_total("), "{shapes}");
    assert!(shapes.contains("def biggest("), "{shapes}");
    let (ok, output) = run(root, "python3", &["app.py"]);
    assert!(ok, "app.py failed:\n{output}");
    assert_eq!(output, expected);
    assert_runs(root, "python3", &["-m", "checks.test_shapes"]);
}

#[tokio::test]
#[ignore = "needs basedpyright (run `refac doctor python`)"]
async fn python_batch_with_a_missing_symbol_changes_nothing() {
    require_server("python");
    let project = common::setup_fixture("python/rename_project");
    let root = project.path();
    let before = snapshot(root);
    let mut requests = python_batch(root);
    requests.insert(1, missing(root, PY_SHAPES));

    let message = refused_batch(requests).await;

    assert!(message.contains("Rename 2 of 5"), "{message}");
    assert!(
        message.contains("1 earlier rename(s) were undone"),
        "{message}"
    );
    assert_unchanged(root, &before);
}

// ---- Dart: lib/shapes.dart line 3 unit, 29 totalArea, 37 largest -------------

const DART_SHAPES: &str = "lib/shapes.dart";
const DART_REPORT: &str = "lib/report.dart";

fn dart_setup() -> tempfile::TempDir {
    require_server("dart");
    let project = common::setup_fixture("dart/rename_package");
    assert_runs(project.path(), "dart", &["pub", "get", "--offline"]);
    project
}

fn dart_batch(project: &Path) -> Vec<RenameRequest> {
    vec![
        request(project, DART_SHAPES, "unit", "side"),
        request(project, DART_SHAPES, "totalArea", "sumArea"),
        request(project, DART_SHAPES, "largest", "biggest"),
        // Another file, one the earlier renames edited (it calls `totalArea`).
        request(project, DART_REPORT, "summary", "overview"),
        request(project, DART_SHAPES, "sumArea", "grandTotal"),
    ]
}

#[tokio::test]
#[ignore = "needs the Dart SDK (run `refac doctor dart`)"]
async fn dart_batch_renames_five_times_in_one_session() {
    let project = dart_setup();
    let root = project.path();
    let (_, expected) = run(root, "dart", &["run", "bin/main.dart"]);

    let reports = batch(dart_batch(root)).await;

    assert_eq!(reports.len(), 5);
    let report = common::read_file(root, DART_REPORT);
    assert!(report.contains("String overview("), "{report}");
    assert!(report.contains("grandTotal(items, scale: 2.0)"), "{report}");
    let shapes = common::read_file(root, DART_SHAPES);
    assert!(shapes.contains("double grandTotal("), "{shapes}");
    assert!(shapes.contains("const double side"), "{shapes}");
    assert_runs(root, "dart", &["analyze", "--fatal-infos"]);
    let (ok, output) = run(root, "dart", &["run", "bin/main.dart"]);
    assert!(ok, "main.dart failed:\n{output}");
    assert_eq!(output, expected);
}

#[tokio::test]
#[ignore = "needs the Dart SDK (run `refac doctor dart`)"]
async fn dart_batch_with_a_missing_symbol_changes_nothing() {
    let project = dart_setup();
    let root = project.path();
    let before = snapshot(root);
    let mut requests = dart_batch(root);
    requests.insert(2, missing(root, DART_SHAPES));

    let message = refused_batch(requests).await;

    assert!(message.contains("Rename 3 of 6"), "{message}");
    assert!(
        message.contains("2 earlier rename(s) were undone"),
        "{message}"
    );
    assert_unchanged(root, &before);
}
