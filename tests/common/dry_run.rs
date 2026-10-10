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
        self.dry_run_json_with_env(moves, &[])
    }

    /// The same with environment variables set for the command.
    pub fn dry_run_json_with_env(
        &self,
        moves: &[(&str, &str)],
        env: &[(&str, &str)],
    ) -> std::process::Output {
        let mut command = Command::new(cli_binary());
        // A dry run of one Kotlin file is refused by default; these tests plan it.
        command.env("REFAC_KOTLIN_BATCH_ONLY", "0");
        command.envs(env.iter().copied());
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
    let (plan, before) = plan_without_changes(project, moves, ignored_folders);
    project.move_ok(moves);
    assert_plan_was_carried_out(project, &plan, &before, ignored_folders);
    plan
}

/// Every file of the project by path, without the folders a tool fills.
pub fn tree_without(project: &Project, ignored_folders: &[&str]) -> Tree {
    project
        .tree()
        .into_iter()
        .filter(|(path, _)| {
            !ignored_folders
                .iter()
                .any(|folder| path.starts_with(&format!("{folder}/")))
        })
        .collect()
}

/// The first half of `assert_plan_matches_move_ignoring`, for a test that
/// carries the move out itself (on a server it keeps): plans the move, checks
/// that the tree is byte for byte what it was, and returns the plan with the
/// tree it was made from.
pub fn plan_without_changes(
    project: &Project,
    moves: &[(&str, &str)],
    ignored_folders: &[&str],
) -> (Value, Tree) {
    let before = tree_without(project, ignored_folders);
    let output = project.dry_run_json(moves);
    assert!(
        output.status.success(),
        "the dry run should succeed:\nstdout:\n{}\nstderr:\n{}",
        stdout_text(&output),
        stderr_text(&output)
    );
    assert_same_tree(&before, &tree_without(project, ignored_folders));
    let plan: Value = serde_json::from_slice(&output.stdout).expect("the dry run prints JSON");
    assert_eq!(plan["dry_run"], true);
    (plan, before)
}

/// The plan of a Kotlin move, made on a fresh copy of the fixture. The leased
/// project of a shared server is no place for it: the compile checks of the
/// tests before left build output in it, which carries the paths of the folder
/// it was built in, so a copy of it that Gradle imports again differs in
/// files that no move touches. A fresh copy is what a project is when nobody
/// built it.
pub fn plan_on_fresh_copy(
    fixture: &str,
    moves: &[(&str, &str)],
    ignored_folders: &[&str],
) -> Value {
    let project = Project::from_fixture(fixture);
    let (plan, _) = plan_without_changes(&project, moves, ignored_folders);
    // The original was never opened by Gradle: nothing of the tools' own state.
    for state in [".gradle", ".kotlin", ".idea", "build", "app/build"] {
        assert!(!project.exists(state), "the dry run left {state} behind");
    }
    plan
}

/// The second half: the move was carried out, and the plan named exactly the
/// files it edited and the paths it moved.
pub fn assert_plan_was_carried_out(
    project: &Project,
    plan: &Value,
    before: &Tree,
    ignored_folders: &[&str],
) {
    let after = tree_without(project, ignored_folders);

    let planned_moves: Vec<(String, String)> = strings(plan, "moves", "from")
        .into_iter()
        .zip(strings(plan, "moves", "to"))
        .collect();
    let planned_files: BTreeSet<String> = strings(plan, "files", "path").into_iter().collect();

    // Every planned move happened.
    for (from, to) in &planned_moves {
        let at = |tree: &Tree, path: &str| {
            tree.keys()
                .any(|key| key == path || key.starts_with(&format!("{path}/")))
        };
        assert!(at(before, from), "{from} was not in the project: {plan}");
        assert!(!at(&after, from), "{from} is still there after the move");
        assert!(at(&after, to), "{to} does not exist after the move");
    }

    // The files the move edited, by the path they had before it, and the files
    // that disappeared without a planned move are the plan's blind spots.
    let mut edited = BTreeSet::new();
    let mut unplanned = Vec::new();
    for (path, bytes) in before {
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
}
