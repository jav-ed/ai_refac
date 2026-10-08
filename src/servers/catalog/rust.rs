use super::in_home;
use crate::drivers::lsp::client::Server as Profile;
use crate::servers::{Candidate, Find, Launch, Requirement, Server, Version};
use std::path::{Path, PathBuf};

pub static RUST: Server = Server {
    languages: &["rust", "rs"],
    language: "Rust",
    name: "rust-analyzer",
    used_for: "refac rename on .rs files",
    env_var: "REFAC_RUST_ANALYZER",
    find: Find::Executable {
        candidates: &[Candidate {
            executable: "rust-analyzer",
            version: Version::Own(&["--version"]),
        }],
        folders,
        launch: Launch {
            profile: Profile::Rust,
            args: &[],
        },
    },
    requirements: &[Requirement {
        name: "Rust toolchain (rustup and cargo)",
        executable: "cargo",
        why: "rust-analyzer loads the project through cargo and the toolchain's standard library sources",
        hint: "install rustup from https://rustup.rs",
    }],
    install: &[
        "rustup component add rust-analyzer   (run it inside the project when the project pins its own toolchain in rust-toolchain.toml, because rustup installs components per toolchain)",
        "rustup also puts a `rust-analyzer` stand-in in ~/.cargo/bin even when the component is missing; the stand-in then fails with \"Unknown binary 'rust-analyzer'\", which is what refac reports. The command above fixes it.",
        "without rustup: download the binary from https://github.com/rust-lang/rust-analyzer/releases, make it executable, and export REFAC_RUST_ANALYZER=/full/path/to/rust-analyzer",
        "check it: rust-analyzer --version   (run from the project folder)",
    ],
};

/// Where rustup and `cargo install` put binaries.
fn folders(_project: &Path) -> Vec<PathBuf> {
    let mut folders = Vec::new();
    folders.extend(std::env::var_os("CARGO_HOME").map(|home| PathBuf::from(home).join("bin")));
    folders.extend(in_home(".cargo/bin"));
    folders
}
