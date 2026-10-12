// `refac move --batch`: the list of moves in a file or on stdin. What the
// command line accepts and refuses before any server starts, and a Markdown
// move (built in, no server) made from a list to show it is the same call as
// the flags.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

fn refac(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_refac"))
        .args(args)
        .output()
        .expect("failed to execute the CLI binary")
}

/// `refac` with `input` on stdin.
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

fn list_file(dir: &Path, json: &str) -> String {
    let path = dir.join("moves.json");
    std::fs::write(&path, json).unwrap();
    path.to_string_lossy().into_owned()
}

/// A project with two notes and an index that links to both.
fn notes_project() -> tempfile::TempDir {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("a.md"), "# A\n").unwrap();
    std::fs::write(project.path().join("b.md"), "# B\n").unwrap();
    std::fs::write(project.path().join("index.md"), "[a](a.md) and [b](b.md)\n").unwrap();
    project
}

const TWO_MOVES: &str = r#"[{"source_path": "a.md", "target_path": "docs/a.md"},
                            {"source_path": "b.md", "target_path": "docs/b.md"}]"#;

#[test]
fn a_batch_cannot_be_combined_with_the_flags() {
    for flags in [["--source-path", "a.md"], ["--target-path", "b.md"]] {
        let output = refac(&["move", "--batch", "m.json", flags[0], flags[1]]);

        assert!(!output.status.success());
        assert!(
            text(&output).contains("cannot be used with"),
            "{}",
            text(&output)
        );
    }
}

#[test]
fn a_move_needs_the_flags_or_a_batch() {
    let output = refac(&["move", "--project-path", "."]);

    assert!(!output.status.success());
    let message = text(&output);
    assert!(message.contains("--source-path"), "{message}");
    assert!(message.contains("--batch"), "{message}");
}

#[test]
fn a_missing_batch_file_is_named() {
    let output = refac(&["move", "--batch", "/no/such/moves.json"]);

    assert!(!output.status.success());
    assert!(
        text(&output).contains("Cannot read the batch file /no/such/moves.json"),
        "{}",
        text(&output)
    );
}

#[test]
fn a_batch_must_be_a_list_of_moves_and_the_message_shows_the_shape() {
    let dir = tempfile::tempdir().unwrap();
    for json in [
        "not json",
        r#"{"source_path": "a.md", "target_path": "b.md"}"#,
        r#"[{"source_path": "a.md"}]"#,
        r#"[{"source_path": "a.md", "target_path": "b.md", "dry_run": true}]"#,
        r#"[{"from": "a.md", "to": "b.md"}]"#,
    ] {
        let path = list_file(dir.path(), json);

        let output = refac(&["move", "--batch", &path]);

        assert!(!output.status.success(), "{json}");
        let message = text(&output);
        assert!(
            message.contains("is not a list of moves"),
            "{json}: {message}"
        );
        assert!(
            message.contains(r#"{"source_path": "src/a.kt", "target_path": "src/pkg/a.kt"}"#),
            "{json}: {message}"
        );
    }
}

#[test]
fn an_empty_list_is_refused_and_nothing_is_moved() {
    let dir = tempfile::tempdir().unwrap();
    let path = list_file(dir.path(), "[]");

    let output = refac(&["move", "--batch", &path]);

    assert!(!output.status.success());
    assert!(
        text(&output).contains("lists no moves"),
        "{}",
        text(&output)
    );
}

#[test]
fn a_batch_moves_what_the_flags_would_and_rewrites_the_links() {
    let project = notes_project();
    let root = project.path().to_str().unwrap();
    let lists = tempfile::tempdir().unwrap();
    let path = list_file(lists.path(), TWO_MOVES);

    let output = refac(&["move", "--project-path", root, "--batch", &path]);

    assert!(output.status.success(), "{}", text(&output));
    assert!(project.path().join("docs/a.md").exists());
    assert!(project.path().join("docs/b.md").exists());
    assert!(!project.path().join("a.md").exists());
    assert_eq!(
        std::fs::read_to_string(project.path().join("index.md")).unwrap(),
        "[a](docs/a.md) and [b](docs/b.md)\n"
    );
    let report = text(&output);
    assert!(report.contains("a.md -> docs/a.md"), "{report}");
    assert!(report.contains("b.md -> docs/b.md"), "{report}");
}

#[test]
fn a_batch_can_come_from_stdin_and_the_json_answer_lists_its_moves() {
    let project = notes_project();
    let root = project.path().to_str().unwrap();

    let output = refac_with_stdin(
        &["move", "--json", "--project-path", root, "--batch", "-"],
        TWO_MOVES,
    );

    assert!(output.status.success(), "{}", text(&output));
    let answer: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(answer["source_path"], serde_json::json!(["a.md", "b.md"]));
    assert_eq!(
        answer["target_path"],
        serde_json::json!(["docs/a.md", "docs/b.md"])
    );
    assert!(project.path().join("docs/b.md").exists());
}

#[test]
fn a_dry_run_of_a_batch_plans_the_moves_and_changes_nothing() {
    let project = notes_project();
    let root = project.path().to_str().unwrap();
    let lists = tempfile::tempdir().unwrap();
    let path = list_file(lists.path(), TWO_MOVES);

    let output = refac(&[
        "move",
        "--dry-run",
        "--project-path",
        root,
        "--batch",
        &path,
    ]);

    assert!(output.status.success(), "{}", text(&output));
    let plan = text(&output);
    assert!(plan.contains("docs/a.md"), "{plan}");
    assert!(plan.contains("docs/b.md"), "{plan}");
    assert!(project.path().join("a.md").exists());
    assert!(!project.path().join("docs").exists());
}

#[test]
fn the_help_and_the_guide_tell_about_the_batch() {
    for args in [["move", "-h"], ["move", "--help"], ["guide", "batching"]] {
        let help = text(&refac(&args));
        assert!(help.contains("--batch"), "{args:?}: {help}");
    }
    let long = text(&refac(&["move", "--help"]));
    assert!(long.contains("source_path"), "{long}");
    assert!(long.contains("moves.json"), "{long}");
    assert!(
        text(&refac(&["--help"])).contains("--batch moves.json"),
        "the top-level help does not show the list file"
    );
}
