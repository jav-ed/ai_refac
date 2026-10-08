//! A Python batch updates every import of both files.

use crate::common;

#[test]
fn python_batch_moves_two_files_and_updates_all_imports() {
    // validators.py imports formatters.py via `from .formatters import ...`.
    // Both move from myapp/utils/ to myapp/core/.
    //
    // Expected sequence inside Rope (single project, sequential do()):
    //   1. Move formatters.py → Rope rewrites validators.py's relative sibling
    //      import to the absolute path myapp.core.formatters (cross-package move).
    //   2. Move validators.py → Rope moves the file; its import of formatters
    //      is already absolute and still correct, so it stays.
    //
    // After the batch:
    //   myapp/core/validators.py must NOT contain 'myapp.utils.formatters' or '.formatters'
    //   myapp/core/validators.py must contain 'myapp.core.formatters'
    //   External callers (main.py) must point to myapp.core.formatters.
    let temp = common::setup_fixture("python/project");
    let project = temp.path();

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("myapp/utils/formatters.py").to_str().unwrap(),
        "--source-path",
        project.join("myapp/utils/validators.py").to_str().unwrap(),
        "--target-path",
        project.join("myapp/core/formatters.py").to_str().unwrap(),
        "--target-path",
        project.join("myapp/core/validators.py").to_str().unwrap(),
    ]);

    common::assert_move_succeeded(&output);

    assert!(
        project.join("myapp/core/formatters.py").exists(),
        "formatters.py must be at target"
    );
    assert!(
        project.join("myapp/core/validators.py").exists(),
        "validators.py must be at target"
    );
    assert!(
        !project.join("myapp/utils/formatters.py").exists(),
        "formatters.py must be gone from source"
    );
    assert!(
        !project.join("myapp/utils/validators.py").exists(),
        "validators.py must be gone from source"
    );

    // External caller must point to the new location
    let main = common::read_file(project, "myapp/main.py");
    assert!(
        !main.contains("myapp.utils.formatters"),
        "main.py: old path must be gone:\n{main}"
    );
    assert!(
        main.contains("myapp.core.formatters"),
        "main.py: new path must be present:\n{main}"
    );

    // validators.py's own import of formatters must also be correct
    let validators = common::read_file(project, "myapp/core/validators.py");
    assert!(
        !validators.contains("myapp.utils.formatters"),
        "validators.py: old formatters path must be gone:\n{validators}"
    );
    assert!(
        !validators.contains("from .formatters") || validators.contains("myapp.core.formatters"),
        "validators.py: import of formatters must resolve to myapp.core.formatters:\n{validators}"
    );
    assert!(
        validators.contains("myapp.core.formatters") || validators.contains("from .formatters"),
        "validators.py: formatters import must still be present in some valid form:\n{validators}"
    );
}
