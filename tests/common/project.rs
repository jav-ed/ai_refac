//! A scratch project for the Markdown tests: write files, run `refac move`, and
//! compare the whole tree before and after.

use super::{assert_move_succeeded, cli_binary, setup_fixture, stderr_text, stdout_text};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

/// Every file below the project as `path -> bytes`.
pub type Tree = BTreeMap<String, Vec<u8>>;

pub struct Project {
    path: PathBuf,
    /// The directory, when the project owns it and removes it at the end.
    _dir: Option<TempDir>,
}

impl Project {
    pub fn empty() -> Self {
        let dir = tempfile::Builder::new()
            .prefix("refac-test-")
            .tempdir()
            .expect("failed to create temp dir");
        Self::owning(dir)
    }

    pub fn from_fixture(name: &str) -> Self {
        Self::owning(setup_fixture(name))
    }

    fn owning(dir: TempDir) -> Self {
        Self {
            path: dir.path().to_path_buf(),
            _dir: Some(dir),
        }
    }

    /// A project in a directory somebody else owns (the leased project of a
    /// shared Kotlin server, see `pool`): the commands run on it, it is not
    /// removed.
    pub fn over(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            _dir: None,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn write(&self, rel: &str, content: &str) {
        self.write_bytes(rel, content.as_bytes());
    }

    pub fn write_bytes(&self, rel: &str, content: &[u8]) {
        let path = self.path().join(rel);
        fs::create_dir_all(path.parent().unwrap()).expect("failed to create parent directories");
        fs::write(&path, content).expect("failed to write test file");
    }

    pub fn read(&self, rel: &str) -> String {
        let path = self.path().join(rel);
        fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
    }

    pub fn exists(&self, rel: &str) -> bool {
        self.path().join(rel).exists()
    }

    /// `refac move --project-path <project>` for the pairs, from any directory.
    pub fn run(&self, moves: &[(&str, &str)]) -> Output {
        let mut command = Command::new(cli_binary());
        // These helpers move one Kotlin file on purpose; the refusal of a
        // single Kotlin change has its own tests in tests/cli/kotlin_cost.rs.
        command.env("REFAC_KOTLIN_BATCH_ONLY", "0");
        command.args(["move", "--project-path"]).arg(self.path());
        command.arg("--source-path");
        command.args(moves.iter().map(|(source, _)| source));
        command.arg("--target-path");
        command.args(moves.iter().map(|(_, target)| target));
        command.output().expect("failed to execute CLI binary")
    }

    /// `refac move` for the pairs with `--project-path` written the way people
    /// and scripts write it: relative to the working directory, which here is
    /// the folder that holds the project.
    pub fn run_from_parent(&self, moves: &[(&str, &str)], extra: &[&str]) -> Output {
        let name = self.path().file_name().expect("the project has a name");
        let mut command = Command::new(cli_binary());
        command.env("REFAC_KOTLIN_BATCH_ONLY", "0");
        command
            .current_dir(self.path().parent().expect("the project has a parent"))
            .arg("move")
            .args(extra)
            .args(["--project-path"])
            .arg(name);
        command.arg("--source-path");
        command.args(moves.iter().map(|(source, _)| source));
        command.arg("--target-path");
        command.args(moves.iter().map(|(_, target)| target));
        command.output().expect("failed to execute CLI binary")
    }

    /// Run the move, require success, and return what the tool printed.
    pub fn move_ok(&self, moves: &[(&str, &str)]) -> String {
        let output = self.run(moves);
        assert_move_succeeded(&output);
        stdout_text(&output)
    }

    /// Run the move, require failure, and return the error text.
    pub fn move_err(&self, moves: &[(&str, &str)]) -> String {
        let output = self.run(moves);
        assert!(
            !output.status.success(),
            "the move should fail:\n{}",
            stdout_text(&output)
        );
        stderr_text(&output)
    }

    pub fn tree(&self) -> Tree {
        let mut tree = Tree::new();
        collect(self.path(), self.path(), &mut tree);
        tree
    }
}

fn collect(root: &Path, dir: &Path, tree: &mut Tree) {
    for entry in fs::read_dir(dir).expect("failed to read the project") {
        let path: PathBuf = entry.unwrap().path();
        if path.is_dir() {
            collect(root, &path, tree);
        } else {
            let rel = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .into_owned();
            tree.insert(rel, fs::read(&path).unwrap());
        }
    }
}

/// The paths that exist in only one of the trees or differ between them.
pub fn changed(before: &Tree, after: &Tree) -> Vec<String> {
    let mut paths: Vec<String> = before
        .keys()
        .chain(after.keys())
        .filter(|path| before.get(*path) != after.get(*path))
        .cloned()
        .collect();
    paths.sort();
    paths.dedup();
    paths
}

/// Text of a file of `tree`.
pub fn text(tree: &Tree, rel: &str) -> String {
    String::from_utf8(
        tree.get(rel)
            .unwrap_or_else(|| panic!("{rel} is not in the tree"))
            .clone(),
    )
    .unwrap_or_else(|_| panic!("{rel} is not UTF-8"))
}

/// Like `assert_eq!` on two trees, but says which files differ and how.
pub fn assert_same_tree(expected: &Tree, actual: &Tree) {
    let mut report = String::new();
    for path in changed(expected, actual) {
        let show = |tree: &Tree| match tree.get(&path) {
            Some(bytes) => String::from_utf8_lossy(bytes).into_owned(),
            None => "<missing>".to_string(),
        };
        report.push_str(&format!(
            "--- {path}\nexpected:\n{}\nactual:\n{}\n",
            show(expected),
            show(actual)
        ));
    }
    assert!(report.is_empty(), "the trees differ:\n{report}");
}
