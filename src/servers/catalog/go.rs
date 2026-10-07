use super::in_home;
use crate::drivers::lsp_client::Server as Profile;
use crate::servers::{Candidate, Find, Launch, Requirement, Server, Version};
use std::path::{Path, PathBuf};

pub static GO: Server = Server {
    languages: &["go", "golang"],
    language: "Go",
    name: "gopls",
    used_for: "refac rename on .go files, and refac move of Go files and packages",
    env_var: "REFAC_GOPLS",
    find: Find::Executable {
        candidates: &[Candidate {
            executable: "gopls",
            version: Version::Own(&["version"]),
        }],
        folders,
        launch: Launch {
            profile: Profile::Go,
            args: &[],
        },
    },
    requirements: &[Requirement {
        name: "Go toolchain",
        executable: "go",
        why: "gopls builds the packages of the project with it",
        hint: "install Go from https://go.dev/dl/ or with the system package manager",
    }],
    install: &[
        "go install golang.org/x/tools/gopls@latest",
        "gopls lands in `$(go env GOPATH)/bin`, usually ~/go/bin; refac looks there by itself. If the file is somewhere else: export REFAC_GOPLS=/full/path/to/gopls",
        "check it: gopls version",
    ],
};

/// Where `go install` puts binaries.
fn folders(_project: &Path) -> Vec<PathBuf> {
    let mut folders = Vec::new();
    folders.extend(std::env::var_os("GOBIN").map(PathBuf::from));
    folders.extend(std::env::var_os("GOPATH").map(|gopath| PathBuf::from(gopath).join("bin")));
    folders.extend(in_home("go/bin"));
    folders
}
