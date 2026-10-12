//! A server that dies says how, with the real process: `sh` stands in for it.

use super::super::{LspSession, SessionConfig};
use serde_json::json;
use std::path::Path;

/// The error of starting a "server" that runs `script` in `sh`.
async fn start(script: &str) -> String {
    let dir = tempfile::tempdir().unwrap();
    let result = LspSession::start(SessionConfig {
        name: "test server",
        executable: Path::new("sh"),
        args: vec!["-c".to_string(), script.to_string()],
        cwd: dir.path(),
        root: dir.path(),
        capabilities: json!({}),
        keep_notifications: &[],
        language_id: |_| "plaintext",
        env: Vec::new(),
        initialization_options: None,
    })
    .await;
    match result {
        Ok(_) => panic!("a server that exits cannot start"),
        Err(error) => format!("{error:#}"),
    }
}

#[tokio::test]
async fn a_server_killed_by_a_signal_names_the_signal_and_what_it_wrote() {
    let error = start("echo 'java.lang.OutOfMemoryError: Java heap space' >&2; kill -9 $$").await;

    assert!(
        error.contains("The test server closed its connection during initialize")
            || error.contains("The test server stopped reading its input"),
        "{error}"
    );
    assert!(error.contains("killed by signal 9 (SIGKILL"), "{error}");
    assert!(error.contains("out-of-memory killer"), "{error}");
    assert!(error.contains("  java.lang.OutOfMemoryError"), "{error}");
}

#[tokio::test]
async fn a_server_that_exits_gives_its_status_and_its_last_lines() {
    let error = start("for n in 1 2 3; do echo \"line $n\" >&2; done; exit 3").await;

    assert!(error.contains("exited with status 3"), "{error}");
    assert!(error.contains("  line 1\n  line 2\n  line 3"), "{error}");
}

#[tokio::test]
async fn a_server_that_writes_nothing_says_so() {
    let error = start("exit 0").await;

    assert!(error.contains("exited with status 0"), "{error}");
    assert!(error.contains("wrote nothing to stderr"), "{error}");
}

#[tokio::test]
async fn only_the_last_lines_of_a_long_log_are_shown() {
    let error =
        start("i=1; while [ $i -le 200 ]; do echo \"log $i\" >&2; i=$((i+1)); done; exit 1").await;

    assert!(error.contains("  log 200"), "{error}");
    assert!(error.contains("  log 181"), "{error}");
    assert!(!error.contains("  log 180\n"), "{error}");
    assert!(!error.contains("  log 1\n"), "{error}");
}
