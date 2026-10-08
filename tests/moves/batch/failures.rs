//! A group that fails, and a batch in which everything fails.

use crate::common;

fn gopls_is_available() -> bool {
    // Find gopls the same way the Go driver does.
    let in_path = std::process::Command::new("gopls")
        .arg("version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if in_path {
        return true;
    }
    if let Ok(home) = std::env::var("HOME") {
        let gopath_bin = std::path::PathBuf::from(home).join("go/bin/gopls");
        if gopath_bin.exists() {
            return std::process::Command::new(&gopath_bin)
                .arg("version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
        }
    }
    false
}

#[test]
fn partial_failure_exits_non_zero_and_reports_success_and_failure_in_one_report() {
    // Skip when gopls is not installed: without it, check_availability() bails
    // before move_files is called, turning a partial failure into a total failure.
    if !gopls_is_available() {
        eprintln!("gopls not found — skipping partial failure test");
        return;
    }

    use std::fs;

    let temp = tempfile::tempdir().expect("failed to create temp dir");
    let project = temp.path();

    // Markdown move: always succeeds (pure filesystem, no LSP).
    fs::write(project.join("readme.md"), "# Readme\n").unwrap();
    fs::create_dir_all(project.join("docs")).unwrap();

    // Go move: cross-directory, no go.mod → build_go_target_package_path returns
    // Err → move_files returns Err → goes to failed_batches (partial failure).
    fs::create_dir_all(project.join("pkg/utils")).unwrap();
    fs::write(
        project.join("pkg/utils/helpers.go"),
        "package utils\n\nfunc Helper() {}\n",
    )
    .unwrap();
    fs::create_dir_all(project.join("pkg/helpers")).unwrap();

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("readme.md").to_str().unwrap(),
        "--source-path",
        project.join("pkg/utils/helpers.go").to_str().unwrap(),
        "--target-path",
        project.join("docs/readme.md").to_str().unwrap(),
        "--target-path",
        project.join("pkg/helpers/helpers.go").to_str().unwrap(),
    ]);

    // Partial failure: the group that moved stays moved, but the request was not
    // carried out in full, so the exit code says so and the whole report is the error.
    assert!(
        !output.status.success(),
        "partial failure must exit non-zero:\nstdout: {}\nstderr: {}",
        common::stdout_text(&output),
        common::stderr_text(&output),
    );

    let report = common::stderr_text(&output);
    assert!(
        report.contains("// Partly done: 1 requested path moved, 1 failed."),
        "report must say the move was only partly done:\n{report}"
    );

    // Success section must be present for Markdown.
    assert!(
        report.contains("Markdown"),
        "report must contain the Markdown success section:\n{report}"
    );

    // Failure section must be present for Go.
    assert!(
        report.contains("// Failed:") && report.contains("helpers.go"),
        "report must contain a Failed section naming the Go file:\n{report}"
    );

    // Markdown file must have moved.
    assert!(
        project.join("docs/readme.md").exists(),
        "readme.md must be at target after partial success"
    );
    assert!(
        !project.join("readme.md").exists(),
        "readme.md must be gone from source after partial success"
    );
}

#[test]
fn all_failed_batch_exits_nonzero_with_error_message() {
    // When every language batch fails, the CLI must exit non-zero and include
    // a human-readable error.  Trigger this by attempting a Go move without
    // go.mod in a project that has ONLY Go files (no fallback successes).
    //
    // Skip when gopls is not installed (same reason as above).
    if !gopls_is_available() {
        eprintln!("gopls not found — skipping all-failed test");
        return;
    }

    use std::fs;

    let temp = tempfile::tempdir().expect("failed to create temp dir");
    let project = temp.path();

    fs::create_dir_all(project.join("pkg/utils")).unwrap();
    fs::write(
        project.join("pkg/utils/helpers.go"),
        "package utils\n\nfunc Helper() {}\n",
    )
    .unwrap();
    fs::create_dir_all(project.join("pkg/helpers")).unwrap();

    let output = common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("pkg/utils/helpers.go").to_str().unwrap(),
        "--target-path",
        project.join("pkg/helpers/helpers.go").to_str().unwrap(),
    ]);

    assert!(
        !output.status.success(),
        "all-failed batch must exit non-zero"
    );

    let stderr = common::stderr_text(&output);
    assert!(
        !stderr.is_empty() || !common::stdout_text(&output).is_empty(),
        "some error output must be present when all batches fail"
    );
}
