//! A single Kotlin change is refused by default, before any server starts;
//! `--allow-single` and `REFAC_KOTLIN_BATCH_ONLY=0` let it through. These tests
//! need no Kotlin server: a refused command never gets that far, and one that
//! is let through stops at a source file that does not exist.

use std::path::Path;
use std::process::{Command, Output};

fn refac(project: &Path, batch_only: Option<&str>, args: &[&str]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_refac"));
    command.current_dir(project).args(args);
    command.env_remove("REFAC_KOTLIN_BATCH_ONLY");
    if let Some(value) = batch_only {
        command.env("REFAC_KOTLIN_BATCH_ONLY", value);
    }
    command.output().expect("failed to execute the CLI binary")
}

fn text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

const MOVE: &[&str] = &["move", "--source-path", "A.kt", "--target-path", "pkg/A.kt"];
const RENAME: &[&str] = &[
    "rename",
    "--file",
    "A.kt",
    "--symbol",
    "old",
    "--new-name",
    "new",
];

fn with(base: &[&'static str], extra: &[&'static str]) -> Vec<&'static str> {
    let mut args = base.to_vec();
    args.extend(extra);
    args
}

#[test]
fn a_single_kotlin_move_is_refused_with_the_batch_command_and_the_way_through() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("A.kt"), "class A\n").unwrap();

    // The variable is not set: the refusal is the default.
    let output = refac(dir.path(), None, MOVE);

    assert_eq!(output.status.code(), Some(1));
    let message = text(&output);
    assert!(message.contains("single Kotlin move"), "{message}");
    assert!(
        message.contains("--source-path A.kt --source-path <next.kt>"),
        "{message}"
    );
    assert!(message.contains("--allow-single"), "{message}");
    assert!(message.contains("REFAC_KOTLIN_BATCH_ONLY=0"), "{message}");
    assert!(message.contains("Nothing was changed"), "{message}");
    assert!(dir.path().join("A.kt").exists());
    assert!(!dir.path().join("pkg").exists());
}

#[test]
fn a_single_kotlin_rename_is_refused_and_the_error_is_json_with_json() {
    let dir = tempfile::tempdir().unwrap();

    let output = refac(dir.path(), None, &with(RENAME, &["--json"]));

    assert_eq!(output.status.code(), Some(1));
    let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    let message = error["error"].as_str().unwrap();
    assert!(message.contains("single Kotlin rename"), "{message}");
    assert!(message.contains("refac rename --batch -"), "{message}");
    assert!(message.contains("--allow-single"), "{message}");
}

#[test]
fn a_dry_run_of_a_single_kotlin_change_is_refused_too() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("A.kt"), "class A\n").unwrap();

    for value in [None, Some("1"), Some("")] {
        for command in [with(MOVE, &["--dry-run"]), with(RENAME, &["--dry-run"])] {
            let output = refac(dir.path(), value, &command);
            assert_eq!(output.status.code(), Some(1), "{value:?} {command:?}");
            assert!(
                text(&output).contains("--allow-single"),
                "{}",
                text(&output)
            );
        }
    }
}

#[test]
fn allow_single_zero_and_a_batch_get_past_the_refusal() {
    let dir = tempfile::tempdir().unwrap();
    let two = [
        "move",
        "--source-path",
        "A.kt",
        "--source-path",
        "B.kt",
        "--target-path",
        "p/A.kt",
        "--target-path",
        "p/B.kt",
    ];
    let cases: Vec<(Option<&str>, Vec<&str>)> = vec![
        (None, with(MOVE, &["--allow-single"])),
        (None, with(RENAME, &["--allow-single"])),
        (Some("1"), with(MOVE, &["--allow-single"])),
        (Some("0"), MOVE.to_vec()),
        (Some("0"), RENAME.to_vec()),
        (None, two.to_vec()),
        (Some("1"), two.to_vec()),
    ];
    for (value, args) in cases {
        let message = text(&refac(dir.path(), value, &args));
        // Whatever stops the command now, it is not the refusal.
        assert!(
            !message.contains("refuses those"),
            "{value:?} {args:?}: {message}"
        );
    }
}

#[test]
fn other_languages_are_not_refused() {
    let dir = tempfile::tempdir().unwrap();
    let output = refac(
        dir.path(),
        None,
        &["move", "--source-path", "a.py", "--target-path", "p/a.py"],
    );
    assert!(
        !text(&output).contains("refuses those"),
        "{}",
        text(&output)
    );
}

#[test]
fn a_value_that_is_not_one_or_zero_is_an_error_not_a_silent_off() {
    let dir = tempfile::tempdir().unwrap();

    let output = refac(dir.path(), Some("yes"), MOVE);

    assert_eq!(output.status.code(), Some(1));
    let message = text(&output);
    assert!(
        message.contains("REFAC_KOTLIN_BATCH_ONLY must be 1"),
        "{message}"
    );
}

#[test]
fn the_help_names_the_variable_and_the_argument() {
    let dir = tempfile::tempdir().unwrap();
    for command in ["move", "rename"] {
        let help = text(&refac(dir.path(), None, &[command, "--help"]));
        assert!(help.contains("--allow-single"), "{command}: {help}");
        assert!(
            help.contains("REFAC_KOTLIN_BATCH_ONLY"),
            "{command}: {help}"
        );
    }
}

#[test]
fn the_refusal_explains_why_kotlin_is_slow_and_lists_the_options() {
    let dir = tempfile::tempdir().unwrap();
    for command in [MOVE, RENAME] {
        let message = text(&refac(dir.path(), None, command));
        assert!(message.contains("WHY IT IS SLOW"), "{message}");
        assert!(message.contains("THE OPTIONS, BEST FIRST"), "{message}");
        assert!(
            message.contains("REFAC_KOTLIN_GRADLE_IDLE_SECS"),
            "{message}"
        );
        assert!(message.contains("--allow-single"), "{message}");
    }
}

#[test]
fn the_kotlin_guide_topic_prints_the_same_explanation() {
    let dir = tempfile::tempdir().unwrap();

    let output = refac(dir.path(), None, &["guide", "kotlin"]);

    assert!(output.status.success(), "{}", text(&output));
    let guide = text(&output);
    assert!(
        guide.starts_with("KOTLIN: why a Kotlin change is slow"),
        "{guide}"
    );
    assert!(guide.contains("IF YOU ARE AN AGENT"), "{guide}");
    assert!(
        text(&refac(dir.path(), None, &["guide"])).contains("kotlin"),
        "the topic list lacks kotlin"
    );
}
