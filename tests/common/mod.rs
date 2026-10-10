use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

/// The Dart analysis server does not like several runs over copies of the same
/// package at once, so every test that moves Dart files holds this lock. It is
/// shared because the moves, the batch and the package-config tests run in one
/// test program.
pub static DART_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub fn cli_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_refac"))
}

pub fn run_cli(args: &[&str]) -> Output {
    Command::new(cli_binary())
        .args(args)
        .output()
        .expect("failed to execute CLI binary")
}

pub fn stdout_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub fn stderr_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

pub fn assert_move_succeeded(output: &Output) {
    let combined = format!(
        "stdout:\n{}\n\nstderr:\n{}",
        stdout_text(output),
        stderr_text(output)
    );
    assert!(output.status.success(), "move should succeed:\n{combined}");
}

pub fn fixture_dir(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// Copy a fixture into a fresh tempdir and return the handle.
/// The fixture contents land directly at temp.path() (not in a subdirectory).
///
/// Uses a non-hidden prefix ("refac-test-") because gopls skips hidden directories
/// (names starting with ".") when cascading workspace edits to importer files.
pub fn setup_fixture(name: &str) -> TempDir {
    let temp = tempfile::Builder::new()
        .prefix("refac-test-")
        .tempdir()
        .expect("failed to create temp dir");
    copy_dir_all(&fixture_dir(name), temp.path()).expect("failed to copy fixture");
    temp
}

pub fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn read_file(root: &Path, rel: &str) -> String {
    let path = root.join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
}

#[allow(dead_code)]
pub mod disk;
#[allow(dead_code)]
pub mod dry_run;
#[allow(dead_code)]
pub mod kotlin;
#[allow(dead_code)]
pub mod links;
#[allow(dead_code)]
pub mod lsp;
#[allow(dead_code)]
pub mod pool;
#[allow(dead_code)]
pub mod project;
#[allow(dead_code)]
pub mod site;
