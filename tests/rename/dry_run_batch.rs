use crate::batch::{dart_batch, dart_setup, go_batch, python_batch, rust_batch, rust_setup};
use crate::common;
use common::lsp::{assert_unchanged, at_line, refused_batch, request, require_server, snapshot};
use refac::drivers::symbol::rename::{RenameReport, RenameRequest};
use refac::logic::rename::handle_rename_batch;
use std::path::Path;
use tempfile::TempDir;

// `refac rename --dry-run --batch` against the real servers, on the tracked
// fixtures. The plan of a batch must be what the batch then does: each rename
// is planned on the files as the ones before it would leave them, so the same
// entries that work together for real work together in a dry run, and the
// plan names the same files with the same number of edits. A dry run that
// planned each entry on the untouched project would be wrong for exactly the
// entries that depend on each other.
//
// Run with the servers installed (`refac doctor` explains how):
//   cargo test --test rename dry_run_batch:: -- --ignored --test-threads=1

fn as_dry_run(requests: Vec<RenameRequest>) -> Vec<RenameRequest> {
    requests
        .into_iter()
        .map(|mut request| {
            request.dry_run = true;
            request
        })
        .collect()
}

async fn run_batch(requests: Vec<RenameRequest>) -> Vec<RenameReport> {
    handle_rename_batch(requests)
        .await
        .unwrap_or_else(|error| panic!("the batch failed: {error:#}"))
}

/// Plans the batch on one copy of the fixture and carries it out on another;
/// both must name the same files and edits, and the plan changes nothing.
async fn assert_plan_matches_batch(
    fixture: impl Fn() -> TempDir,
    requests: impl Fn(&Path) -> Vec<RenameRequest>,
) {
    let planned = fixture();
    let before = snapshot(planned.path());
    let plan = run_batch(as_dry_run(requests(planned.path()))).await;
    assert_unchanged(planned.path(), &before);
    assert!(plan.iter().all(|report| report.dry_run));

    let carried_out = fixture();
    let done = run_batch(requests(carried_out.path())).await;

    assert_eq!(plan.len(), done.len());
    for (index, (planned, done)) in plan.iter().zip(&done).enumerate() {
        assert_eq!(planned.files, done.files, "rename {}", index + 1);
        assert_eq!(planned.edits, done.edits, "rename {}", index + 1);
    }
}

fn go_fixture() -> TempDir {
    require_server("go");
    common::setup_fixture("go/rename_module")
}

fn python_fixture() -> TempDir {
    require_server("python");
    common::setup_fixture("python/rename_project")
}

#[tokio::test]
#[ignore = "needs gopls (run `refac doctor go`)"]
async fn a_go_batch_is_planned_as_it_is_carried_out() {
    assert_plan_matches_batch(go_fixture, go_batch).await;
}

#[tokio::test]
#[ignore = "needs rust-analyzer (run `refac doctor rust`)"]
async fn a_rust_batch_is_planned_as_it_is_carried_out() {
    assert_plan_matches_batch(rust_setup, rust_batch).await;
}

#[tokio::test]
#[ignore = "needs basedpyright (run `refac doctor python`)"]
async fn a_python_batch_is_planned_as_it_is_carried_out() {
    assert_plan_matches_batch(python_fixture, python_batch).await;
}

#[tokio::test]
#[ignore = "needs the Dart SDK (run `refac doctor dart`)"]
async fn a_dart_batch_is_planned_as_it_is_carried_out() {
    assert_plan_matches_batch(dart_setup, dart_batch).await;
}

#[tokio::test]
#[ignore = "needs gopls (run `refac doctor go`)"]
async fn a_dry_run_refuses_what_the_real_batch_refuses() {
    let project = go_fixture();
    let root = project.path();
    let before = snapshot(root);
    // The second entry still names `Area`, which the first has already renamed.
    let requests = as_dry_run(vec![
        at_line(request(root, "shape/shape.go", "Area", "Surface"), 8),
        at_line(request(root, "shape/shape.go", "Area", "Extent"), 8),
    ]);

    let message = refused_batch(requests).await;

    assert!(message.contains("Rename 2 of 2"), "{message}");
    assert!(message.contains("nothing was changed"), "{message}");
    assert_unchanged(root, &before);
}
