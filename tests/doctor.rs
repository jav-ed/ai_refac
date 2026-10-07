//! `refac doctor`, and what a command says when its language server is not
//! there. These tests run the real binary in an environment that has no
//! server at all (an empty PATH and an empty HOME), so they need no installed
//! tool and pass on any machine.

#[allow(dead_code)]
mod common;

use common::{cli_binary, stderr_text, stdout_text};
use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

/// The environment variables of every server; none may leak in.
const SERVER_VARIABLES: &[&str] = &[
    "REFAC_GOPLS",
    "REFAC_RUST_ANALYZER",
    "REFAC_PYTHON_SERVER",
    "REFAC_DART",
    "REFAC_KOTLIN_SERVER",
];

/// A machine with refac and nothing else: PATH and HOME are empty folders.
struct BareMachine {
    path: TempDir,
    home: TempDir,
}

impl BareMachine {
    fn new() -> Self {
        Self {
            path: TempDir::new().unwrap(),
            home: TempDir::new().unwrap(),
        }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(cli_binary());
        command.env_clear();
        command.env("PATH", self.path.path());
        command.env("HOME", self.home.path());
        command
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command().args(args).output().unwrap()
    }

    fn run_with(&self, variable: &str, value: &str, args: &[&str]) -> Output {
        self.command()
            .env(variable, value)
            .args(args)
            .output()
            .unwrap()
    }
}

fn write(root: &Path, relative: &str, text: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn go_project() -> TempDir {
    let project = TempDir::new().unwrap();
    write(
        project.path(),
        "go.mod",
        "module example.com/demo\n\ngo 1.22\n",
    );
    write(
        project.path(),
        "demo.go",
        "package demo\n\nfunc Area() int { return 1 }\n",
    );
    project
}

fn rust_project() -> TempDir {
    let project = TempDir::new().unwrap();
    write(
        project.path(),
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(project.path(), "src/lib.rs", "pub fn area() -> u32 { 1 }\n");
    project
}

fn python_project() -> TempDir {
    let project = TempDir::new().unwrap();
    write(project.path(), "demo.py", "def area():\n    return 1\n");
    project
}

#[test]
fn the_overview_names_every_language_and_changes_nothing() {
    let machine = BareMachine::new();
    let output = machine.run(&["doctor"]);
    assert!(output.status.success(), "{}", stderr_text(&output));
    let text = stdout_text(&output);
    for language in ["go", "rust", "python", "dart", "kotlin", "typescript"] {
        assert!(text.contains(language), "{language} missing from:\n{text}");
    }
    assert!(text.contains("MISSING"), "{text}");
    assert!(text.contains("refac doctor <language>"), "{text}");
}

#[test]
fn a_missing_server_is_explained_step_by_step_and_the_exit_code_says_so() {
    let machine = BareMachine::new();
    let output = machine.run(&["doctor", "go"]);
    assert!(!output.status.success());
    let text = stdout_text(&output);
    assert!(text.contains("Go: gopls [MISSING]"), "{text}");
    assert!(text.contains("Looked here:"), "{text}");
    assert!(text.contains("$REFAC_GOPLS: not set"), "{text}");
    assert!(text.contains("PATH: no command named `gopls`"), "{text}");
    assert!(
        text.contains("go install golang.org/x/tools/gopls@latest"),
        "{text}"
    );
    assert!(text.contains("Check again with: refac doctor go"), "{text}");
}

#[test]
fn the_doctor_answers_in_json_for_scripts() {
    let machine = BareMachine::new();
    let output = machine.run(&["doctor", "rust", "--json"]);
    assert!(!output.status.success());
    let reports: serde_json::Value = serde_json::from_str(&stdout_text(&output)).unwrap();
    let report = &reports[0];
    assert_eq!(report["language"], "rust");
    assert_eq!(report["status"], "missing");
    assert_eq!(report["env_var"], "REFAC_RUST_ANALYZER");
    assert!(
        report["install"][0]
            .as_str()
            .unwrap()
            .contains("rustup component add rust-analyzer")
    );
    assert!(!report["looked"].as_array().unwrap().is_empty());
}

#[test]
fn an_unknown_language_lists_the_ones_that_work() {
    let machine = BareMachine::new();
    let output = machine.run(&["doctor", "cobol"]);
    assert!(!output.status.success());
    let text = stderr_text(&output);
    assert!(text.contains("`cobol` is not a language"), "{text}");
    assert!(text.contains("Known: go, rust, python"), "{text}");
}

#[test]
fn a_wrong_variable_is_final_and_names_itself() {
    let machine = BareMachine::new();
    let output = machine.run_with("REFAC_GOPLS", "/no/such/gopls", &["doctor", "go"]);
    assert!(!output.status.success());
    let text = stdout_text(&output);
    assert!(text.contains("[BROKEN]"), "{text}");
    assert!(
        text.contains("$REFAC_GOPLS is set to /no/such/gopls"),
        "{text}"
    );
    // A mistake in the variable is reported, not worked around.
    assert!(!text.contains("PATH:"), "{text}");
}

/// A rename that cannot find its server says what it looked at, names the
/// variable, and points at the doctor, in all three languages.
#[test]
fn a_rename_without_its_server_points_at_the_doctor() {
    let machine = BareMachine::new();
    let go = go_project();
    let rust = rust_project();
    let python = python_project();
    let cases: [(&TempDir, &str, &str, &str, &str, &str); 3] = [
        (&go, "demo.go", "Area", "REFAC_GOPLS", "go", "gopls"),
        (
            &rust,
            "src/lib.rs",
            "area",
            "REFAC_RUST_ANALYZER",
            "rust",
            "rust-analyzer",
        ),
        (
            &python,
            "demo.py",
            "area",
            "REFAC_PYTHON_SERVER",
            "python",
            "basedpyright",
        ),
    ];
    for (project, file, symbol, variable, language, server) in cases {
        let output = machine.run(&[
            "rename",
            "--project-path",
            project.path().to_str().unwrap(),
            "--file",
            file,
            "--symbol",
            symbol,
            "--new-name",
            "surface",
        ]);
        assert!(!output.status.success(), "{language} should fail");
        let text = stderr_text(&output);
        assert!(text.contains("was not found"), "{language}: {text}");
        assert!(text.contains(server), "{language}: {text}");
        assert!(text.contains(&format!("${variable}")), "{language}: {text}");
        assert!(
            text.contains(&format!("Run `refac doctor {language}`")),
            "{language}: {text}"
        );
    }
}

#[test]
fn nothing_is_written_when_the_server_is_missing() {
    let machine = BareMachine::new();
    let project = go_project();
    let before = fs::read_to_string(project.path().join("demo.go")).unwrap();
    let output = machine.run(&[
        "rename",
        "--project-path",
        project.path().to_str().unwrap(),
        "--file",
        "demo.go",
        "--symbol",
        "Area",
        "--new-name",
        "Surface",
    ]);
    assert!(!output.status.success());
    assert_eq!(
        fs::read_to_string(project.path().join("demo.go")).unwrap(),
        before
    );
}

/// The checks that need no server run first, so the agent is not sent to
/// install a server for a request that was wrong anyway.
#[test]
fn a_bad_request_is_reported_before_the_server_is_looked_for() {
    let machine = BareMachine::new();
    let project = go_project();
    let output = machine.run(&[
        "rename",
        "--project-path",
        project.path().to_str().unwrap(),
        "--file",
        "demo.go",
        "--symbol",
        "Area",
        "--new-name",
        "func",
    ]);
    assert!(!output.status.success());
    let text = stderr_text(&output);
    assert!(text.contains("func"), "{text}");
    assert!(!text.contains("language server"), "{text}");
}

#[test]
fn the_server_variables_are_the_documented_ones() {
    // The names printed by the doctor are the ones the code reads.
    let machine = BareMachine::new();
    let output = machine.run(&["doctor", "--json"]);
    let reports: serde_json::Value = serde_json::from_str(&stdout_text(&output)).unwrap();
    let names: Vec<&str> = reports
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|report| report["env_var"].as_str())
        .filter(|name| !name.is_empty())
        .collect();
    for variable in SERVER_VARIABLES {
        assert!(names.contains(variable), "{variable} not in {names:?}");
    }
}

/// A move needs the same servers, and says the same thing when they are not
/// there.
#[test]
fn a_move_without_its_server_points_at_the_doctor_too() {
    let machine = BareMachine::new();
    let go = go_project();
    write(go.path(), "shape/shape.go", "package shape\n");
    let output = machine.run(&[
        "move",
        "--project-path",
        go.path().to_str().unwrap(),
        "--source-path",
        "shape/shape.go",
        "--target-path",
        "geometry/shape.go",
    ]);
    assert!(!output.status.success());
    let text = stderr_text(&output);
    assert!(
        text.contains("The Go language server (gopls) was not found"),
        "{text}"
    );
    assert!(text.contains("Run `refac doctor go`"), "{text}");
    assert!(go.path().join("shape/shape.go").exists());
    assert!(!go.path().join("geometry").exists());
}
