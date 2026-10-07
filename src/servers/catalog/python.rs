use super::in_home;
use crate::drivers::lsp_client::Server as Profile;
use crate::servers::{Candidate, Find, Launch, Requirement, Server, Version};
use std::path::{Path, PathBuf};

pub static PYTHON: Server = Server {
    languages: &["python", "py"],
    language: "Python",
    name: "basedpyright",
    used_for: "refac rename on .py files",
    env_var: "REFAC_PYTHON_SERVER",
    find: Find::Executable {
        // Plain pyright cannot list the overrides of a method, so it would
        // rename half of a family of overrides; only basedpyright is used.
        candidates: &[Candidate {
            executable: "basedpyright-langserver",
            version: Version::Companion("basedpyright", &["--version"]),
        }],
        folders,
        launch: Launch {
            profile: Profile::Pyright,
            args: &["--stdio"],
        },
    },
    requirements: &[Requirement {
        name: "Python 3 with pip",
        executable: "python3",
        why: "basedpyright is a Python package and brings its own Node.js",
        hint: "install Python 3 from https://www.python.org/downloads/ or with the system package manager",
    }],
    install: &[
        "pip install basedpyright   (or: pipx install basedpyright, or: uv tool install basedpyright)",
        "plain pyright is not enough: it cannot list the overrides of a method, and refac renames a method together with its overrides",
        "refac looks in PATH, ~/.local/bin, and the project's .venv/bin and venv/bin; for any other place: export REFAC_PYTHON_SERVER=/full/path/to/basedpyright-langserver",
        "check it: basedpyright --version",
    ],
};

/// Where pip, pipx and uv put commands: the user's bin folder and the
/// virtual environment of the project.
fn folders(project: &Path) -> Vec<PathBuf> {
    let mut folders = vec![project.join(".venv/bin"), project.join("venv/bin")];
    folders.extend(in_home(".local/bin"));
    folders
}
