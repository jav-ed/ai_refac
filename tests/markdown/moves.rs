//! Moving a Markdown file through the CLI: the links in other files and inside the moved file follow it.

use std::fs;
use std::path::{Path, PathBuf};
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

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent directories");
    }

    fs::write(path, content).expect("failed to write test file");
}

fn assert_move_succeeded(output: &Output) {
    let combined = format!(
        "stdout:\n{}\n\nstderr:\n{}",
        stdout_text(output),
        stderr_text(output)
    );
    assert!(
        output.status.success(),
        "markdown move should succeed:\n{combined}"
    );
}

fn contains_either(haystack: &str, left: &str, right: &str) -> bool {
    haystack.contains(left) || haystack.contains(right)
}

mod code;
mod definitions;
mod inline;
