//! A Rust batch that crosses modules is refused before anything is written.

use crate::common;

#[test]
fn rust_batch_rejects_cross_module_file_moves_before_mutation() {
    // Rust module boundaries are semantic. A batch of independent file moves
    // must not recreate the former #[path] shim behavior.
    let temp = common::setup_fixture("rust/project");
    let project = temp.path();

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("src/types.rs").to_str().unwrap(),
        "--source-path",
        project.join("src/error.rs").to_str().unwrap(),
        "--target-path",
        project.join("src/shared/types.rs").to_str().unwrap(),
        "--target-path",
        project.join("src/shared/error.rs").to_str().unwrap(),
    ]);

    assert!(!output.status.success());
    assert!(common::stderr_text(&output).contains("move-module"));
    assert!(project.join("src/types.rs").exists());
    assert!(project.join("src/error.rs").exists());
    assert!(!project.join("src/shared").exists());
}
