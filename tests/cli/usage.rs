use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn cli_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_refac"))
}

fn run_cli(args: &[&str]) -> Output {
    Command::new(cli_binary())
        .args(args)
        .output()
        .expect("failed to execute CLI binary")
}

fn stdout_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn top_level_help_mentions_primary_commands() {
    let output = run_cli(&["--help"]);

    assert!(
        output.status.success(),
        "help should succeed: stderr={}",
        stderr_text(&output)
    );

    let stdout = stdout_text(&output);
    assert!(stdout.contains("Usage:"), "missing usage text: {stdout}");
    assert!(stdout.contains("move"), "missing move subcommand: {stdout}");
    assert!(
        stdout.contains("move-module"),
        "missing Rust module subcommand: {stdout}"
    );
    assert!(
        stdout.contains("completions") || stdout.contains("completion"),
        "missing completions command: {stdout}"
    );
    assert!(
        stdout.contains("man") || stdout.contains("manpage"),
        "missing manpage command: {stdout}"
    );
}

#[test]
fn move_module_help_exposes_logical_paths_and_workspace_root() {
    let output = run_cli(&["move-module", "--help"]);

    assert!(
        output.status.success(),
        "subcommand help should succeed: stderr={}",
        stderr_text(&output)
    );

    let stdout = stdout_text(&output);
    assert!(
        stdout.contains("--project-path"),
        "missing project path: {stdout}"
    );
    assert!(
        stdout.contains("<SOURCE_MODULE>"),
        "missing source module: {stdout}"
    );
    assert!(
        stdout.contains("<TARGET_MODULE>"),
        "missing target module: {stdout}"
    );
    assert!(
        stdout.contains("crate::"),
        "missing logical-path guidance: {stdout}"
    );
}

#[test]
fn move_help_exposes_core_flags() {
    let output = run_cli(&["move", "--help"]);

    assert!(
        output.status.success(),
        "subcommand help should succeed: stderr={}",
        stderr_text(&output)
    );

    let stdout = stdout_text(&output);
    assert!(
        stdout.contains("--project-path"),
        "missing project path flag: {stdout}"
    );
    assert!(
        stdout.contains("--source-path"),
        "missing source path flag: {stdout}"
    );
    assert!(
        stdout.contains("--target-path"),
        "missing target path flag: {stdout}"
    );
}

/// Passing a directory as --source-path must no longer be rejected at the CLI level.
/// Directory moves are routed to the TypeScript driver. A non-TS directory that doesn't
/// exist in a TS project context will fail at the driver level, not with a blanket
/// "Directory moves are not supported" validation error.
#[test]
fn directory_source_is_not_rejected_by_validation() {
    // Use a real temp dir as source — validation should pass it through.
    // We don't have a full TS project here so the driver will fail, but the error
    // must NOT be the old "Directory moves are not supported" message.
    let tmp = std::env::temp_dir();
    let output = run_cli(&[
        "move",
        "--project-path",
        tmp.to_str().unwrap(),
        "--source-path",
        tmp.to_str().unwrap(),
        "--target-path",
        "/tmp/refac_cli_test_dir_target_unreachable",
    ]);

    let combined = format!("{}\n{}", stdout_text(&output), stderr_text(&output));
    assert!(
        !combined.contains("Directory moves are not supported"),
        "validation should no longer reject directories with that message, got: {combined}"
    );
}

/// A request in which nothing could be moved is an error that names the skipped file,
/// not a success that claims "0 requested paths".
#[test]
fn unsupported_extension_alone_fails_and_names_the_file() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("refac_test_unsupported.xyz");
    fs::write(&src, "").unwrap();
    let target = tmp.path().join("refac_test_unsupported_target.xyz");

    let output = run_cli(&[
        "move",
        "--project-path",
        tmp.path().to_str().unwrap(),
        "--source-path",
        src.to_str().unwrap(),
        "--target-path",
        target.to_str().unwrap(),
    ]);

    assert!(
        !output.status.success(),
        "nothing moved, so the command must fail: {}",
        stdout_text(&output)
    );
    let error = stderr_text(&output);
    assert!(error.contains("// Nothing was moved."), "{error}");
    assert!(error.contains("Skipped (unsupported extension)"), "{error}");
    assert!(error.contains("refac_test_unsupported.xyz"), "{error}");
    assert!(src.exists() && !target.exists());
}

#[test]
fn mismatched_source_and_target_counts_fail_cleanly() {
    let output = run_cli(&[
        "move",
        "--project-path",
        ".",
        "--source-path",
        "src/a.ts",
        "--target-path",
        "src/b.ts",
        "--target-path",
        "src/c.ts",
    ]);

    assert!(!output.status.success(), "mismatched paths should fail");

    let stderr = stderr_text(&output);
    let combined = format!("{}\n{}", stdout_text(&output), stderr);
    assert!(
        combined.contains("mismatch") || combined.contains("count") || combined.contains("source"),
        "expected a validation-style error, got: {combined}"
    );
}
