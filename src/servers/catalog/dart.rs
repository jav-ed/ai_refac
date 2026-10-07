use crate::drivers::lsp_client::Server as Profile;
use crate::servers::{Candidate, Find, Launch, Server, Version};
use std::path::{Path, PathBuf};

pub static DART: Server = Server {
    languages: &["dart", "flutter"],
    language: "Dart",
    name: "the Dart analysis server (dart language-server)",
    used_for: "refac rename on .dart files, and refac move of Dart files",
    env_var: "REFAC_DART",
    find: Find::Executable {
        candidates: &[Candidate {
            executable: "dart",
            version: Version::Own(&["--version"]),
        }],
        folders,
        launch: Launch {
            profile: Profile::Dart,
            args: &["language-server"],
        },
    },
    requirements: &[],
    install: &[
        "install the Dart SDK from https://dart.dev/get-dart (it contains the analysis server; there is nothing else to install)",
        "Flutter projects: Flutter ships its own Dart; add `<flutter>/bin/cache/dart-sdk/bin` to PATH, or export REFAC_DART=/full/path/to/dart",
        "check it: dart --version",
    ],
};

/// The Dart SDK inside a Flutter checkout.
fn folders(_project: &Path) -> Vec<PathBuf> {
    std::env::var_os("FLUTTER_ROOT")
        .map(|root| PathBuf::from(root).join("bin/cache/dart-sdk/bin"))
        .into_iter()
        .collect()
}
