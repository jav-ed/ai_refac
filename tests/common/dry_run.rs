//! The check every `move --dry-run` test makes: the plan changes nothing, and
//! the real move on the same project then does what the plan said.

use super::project::{Project, Tree, assert_same_tree};
use super::{cli_binary, stderr_text, stdout_text};
use serde_json::Value;
use std::collections::BTreeSet;
use std::process::Command;

impl Project {
    /// `refac move --dry-run --json` for the pairs.
    pub fn dry_run_json(&self, moves: &[(&str, &str)]) -> std::process::Output {
        let mut command = Command::new(cli_binary());
        command
            .args(["move", "--dry-run", "--json", "--project-path"])
            .arg(self.path());
        command.arg("--source-path");
        command.args(moves.iter().map(|(source, _)| source));
        command.arg("--target-path");
        command.args(moves.iter().map(|(_, target)| target));
        command.output().expect("failed to execute CLI binary")
    }
}

/// Where `path` ends up when the planned moves are carried out.
fn destination(path: &str, moves: &[(String, String)]) -> String {
    for (from, to) in moves {
        if path == from {
            return to.clone();
        }
        if let Some(rest) = path.strip_prefix(&format!("{from}/")) {
            return format!("{to}/{rest}");
        }
    }
    path.to_string()
}

fn strings(value: &Value, key: &str, inner: &str) -> Vec<String> {
    value[key]
        .as_array()
        .unwrap_or_else(|| panic!("`{key}` is missing in {value}"))
        .iter()
        .map(|entry| entry[inner].as_str().unwrap().to_string())
        .collect()
}

/// Plans the move, checks the tree is byte for byte what it was, carries the
/// move out, and checks that the plan named exactly the files the move edited
/// and the paths it moved. Returns the plan.
pub fn assert_plan_matches_move(project: &Project, moves: &[(&str, &str)]) -> Value {
    assert_plan_matches_move_ignoring(project, moves, &[])
}

/// Like `assert_plan_matches_move`, but folders a tool fills while it loads the
/// project (Cargo's `target`) are not part of the comparison, as they are not
/// part of the move.
pub fn assert_plan_matches_move_ignoring(
    project: &Project,
    moves: &[(&str, &str)],
    ignored_folders: &[&str],
) -> Value {
    let tree = || -> Tree {
        project
            .tree()
            .into_iter()
            .filter(|(path, _)| {
                !ignored_folders
                    .iter()
                    .any(|folder| path.starts_with(&format!("{folder}/")))
            })
            .collect()
    };
    let before: Tree = tree();
    let output = project.dry_run_json(moves);
    assert!(
        output.status.success(),
        "the dry run should succeed:\nstdout:\n{}\nstderr:\n{}",
        stdout_text(&output),
        stderr_text(&output)
    );
    assert_same_tree(&before, &tree());
    let plan: Value = serde_json::from_slice(&output.stdout).expect("the dry run prints JSON");
    assert_eq!(plan["dry_run"], true);

    project.move_ok(moves);
    let after = tree();

    let planned_moves: Vec<(String, String)> = strings(&plan, "moves", "from")
        .into_iter()
        .zip(strings(&plan, "moves", "to"))
        .collect();
    let planned_files: BTreeSet<String> = strings(&plan, "files", "path").into_iter().collect();

    // Every planned move happened.
    for (from, to) in &planned_moves {
        let at = |tree: &Tree, path: &str| {
            tree.keys()
                .any(|key| key == path || key.starts_with(&format!("{path}/")))
        };
        assert!(at(&before, from), "{from} was not in the project: {plan}");
        assert!(!at(&after, from), "{from} is still there after the move");
        assert!(at(&after, to), "{to} does not exist after the move");
    }

    // The files the move edited, by the path they had before it, and the files
    // that disappeared without a planned move are the plan's blind spots.
    let mut edited = BTreeSet::new();
    let mut unplanned = Vec::new();
    for (path, bytes) in &before {
        let now = destination(path, &planned_moves);
        match after.get(&now) {
            Some(new_bytes) if new_bytes != bytes => {
                edited.insert(path.clone());
            }
            Some(_) => {}
            None => unplanned.push(path.clone()),
        }
    }
    assert!(
        unplanned.is_empty(),
        "files vanished without a planned move: {unplanned:?}\nplan: {plan}"
    );
    assert_eq!(
        edited, planned_files,
        "the plan and the move edited different files\nplan: {plan}"
    );
    plan
}
