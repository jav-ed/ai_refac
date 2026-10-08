// `refac rename --batch`: what the command line accepts and refuses before any
// language server is started (so these tests need no server). The batch itself
// is tested against the real servers in rename_batch.rs.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

fn refac(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_refac"))
        .args(args)
        .output()
        .expect("failed to execute the CLI binary")
}

/// `refac rename --batch -` with `input` on stdin.
fn refac_with_stdin(args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_refac"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to execute the CLI binary");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn batch_file(dir: &Path, json: &str) -> String {
    let path = dir.join("batch.json");
    std::fs::write(&path, json).unwrap();
    path.to_string_lossy().into_owned()
}

#[test]
fn a_batch_cannot_be_combined_with_a_single_rename() {
    let output = refac(&["rename", "--batch", "b.json", "--file", "a.go"]);

    assert!(!output.status.success());
    assert!(
        text(&output).contains("cannot be used with"),
        "{}",
        text(&output)
    );
}

#[test]
fn a_rename_needs_a_file_or_a_batch() {
    let output = refac(&["rename", "--symbol", "a", "--new-name", "b"]);

    assert!(!output.status.success());
    assert!(text(&output).contains("--file"), "{}", text(&output));
}

#[test]
fn a_missing_batch_file_is_named() {
    let output = refac(&["rename", "--batch", "/no/such/batch.json"]);

    assert!(!output.status.success());
    assert!(
        text(&output).contains("Cannot read the batch file /no/such/batch.json"),
        "{}",
        text(&output)
    );
}

#[test]
fn a_batch_must_be_a_list_of_renames() {
    let dir = tempfile::tempdir().unwrap();
    for json in [
        "not json",
        "{\"file\": \"a.go\"}",
        "[{\"file\": \"a.go\", \"symbol\": \"A\"}]",
        "[{\"file\": \"a.go\", \"symbol\": \"A\", \"new_name\": \"B\", \"dry_run\": true}]",
    ] {
        let path = batch_file(dir.path(), json);

        let output = refac(&["rename", "--batch", &path]);

        assert!(!output.status.success(), "{json}");
        assert!(
            text(&output).contains("is not a batch"),
            "{json}: {}",
            text(&output)
        );
    }
}

#[test]
fn an_empty_batch_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = batch_file(dir.path(), "[]");

    let output = refac(&["rename", "--batch", &path]);

    assert!(!output.status.success());
    assert!(
        text(&output).contains("at least one rename"),
        "{}",
        text(&output)
    );
}

#[test]
fn a_batch_in_two_languages_is_refused_before_a_server_starts() {
    let dir = tempfile::tempdir().unwrap();
    let path = batch_file(
        dir.path(),
        r#"[{"file": "a.go", "symbol": "A", "new_name": "B"},
            {"file": "b.rs", "symbol": "C", "new_name": "D"}]"#,
    );

    let output = refac(&["rename", "--batch", &path, "--project-path", "."]);

    assert!(!output.status.success());
    assert!(
        text(&output).contains("one batch per language"),
        "{}",
        text(&output)
    );
}

#[test]
fn a_batch_can_come_from_stdin_and_errors_come_as_json() {
    let output = refac_with_stdin(
        &["rename", "--batch", "-", "--json"],
        r#"[{"file": "a.ts", "symbol": "A", "new_name": "B"},
            {"file": "b.ts", "symbol": "C", "new_name": "D"}]"#,
    );

    assert!(!output.status.success());
    let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["status"], "error");
    assert!(
        error["error"]
            .as_str()
            .unwrap()
            .contains("TypeScript/JavaScript rename"),
        "{error}"
    );
}

#[test]
fn the_help_describes_the_batch() {
    let output = refac(&["rename", "--help"]);

    let help = text(&output);
    assert!(help.contains("--batch"), "{help}");
    assert!(help.contains("all or nothing"), "{help}");
}
