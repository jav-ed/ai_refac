//! TypeScript batches: unrelated files and a cross-importing pair.

use crate::common;

#[test]
fn typescript_batch_moves_two_unrelated_files() {
    // Two files with no import relationship move together — placement only.
    let temp = common::setup_fixture("typescript/project");
    let project = temp.path();

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project
            .join("src/utils/string_helpers.ts")
            .to_str()
            .unwrap(),
        "--source-path",
        project.join("src/utils/math_helpers.ts").to_str().unwrap(),
        "--target-path",
        project
            .join("src/moved/string_helpers.ts")
            .to_str()
            .unwrap(),
        "--target-path",
        project.join("src/moved/math_helpers.ts").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);
    assert!(
        project.join("src/moved/string_helpers.ts").exists(),
        "string_helpers.ts must be at target"
    );
    assert!(
        project.join("src/moved/math_helpers.ts").exists(),
        "math_helpers.ts must be at target"
    );
    assert!(
        !project.join("src/utils/string_helpers.ts").exists(),
        "string_helpers.ts must be gone from source"
    );
    assert!(
        !project.join("src/utils/math_helpers.ts").exists(),
        "math_helpers.ts must be gone from source"
    );
}

#[test]
fn typescript_batch_updates_cross_import_when_both_files_move_to_same_dir() {
    // task_service.ts imports date_helpers.ts via '../utils/date_helpers'.
    // Both move to src/core/.  Since they land in the same directory, Refac
    // must rewrite the import to './date_helpers' (same-dir relative path).
    let temp = common::setup_fixture("typescript/project");
    let project = temp.path();

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project
            .join("src/services/task_service.ts")
            .to_str()
            .unwrap(),
        "--source-path",
        project.join("src/utils/date_helpers.ts").to_str().unwrap(),
        "--target-path",
        project.join("src/core/task_service.ts").to_str().unwrap(),
        "--target-path",
        project.join("src/core/date_helpers.ts").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);
    assert!(
        project.join("src/core/task_service.ts").exists(),
        "task_service.ts must be at target"
    );
    assert!(
        project.join("src/core/date_helpers.ts").exists(),
        "date_helpers.ts must be at target"
    );
    assert!(
        !project.join("src/services/task_service.ts").exists(),
        "task_service.ts must be gone from source"
    );
    assert!(
        !project.join("src/utils/date_helpers.ts").exists(),
        "date_helpers.ts must be gone from source"
    );

    let svc = common::read_file(project, "src/core/task_service.ts");

    // Old path must be gone
    assert!(
        !svc.contains("../utils/date_helpers"),
        "old import path must be gone:\n{svc}"
    );

    // Both imports (named + dynamic) must resolve to the same-dir sibling
    assert!(
        svc.contains("'./date_helpers'") || svc.contains("\"./date_helpers\""),
        "import must be updated to './date_helpers' (same-dir after batch move):\n{svc}"
    );
}
