//! `refac doctor --help`.

pub(in crate::cli) const ABOUT: &str =
    "Check the language servers refac starts, and print how to install a missing one.";

pub(in crate::cli) const LONG_ABOUT: &str = r#"Check the language servers refac starts, and print how to install a missing one.

Moves and renames in Go, Rust, Python, Dart, Kotlin and TypeScript are done with a language server
that refac starts for the command and stops afterwards. When one is not installed, the failing
command names the places it looked at and points here. `doctor` is how to fix that without help.

  refac doctor              One line per language: ready, missing or broken, the version found and
                            where. No server is started, so it returns at once.
  refac doctor <language>   Looks for that language's server exactly as a command would, then
                            STARTS it, makes it answer the handshake and stops it again. Prints the
                            version, the path, the capabilities it must offer, and, when something
                            is wrong, the numbered steps that fix it (download, install, the
                            environment variable that points refac at it).

<LANGUAGE> is one of: go, rust, python, dart, kotlin, typescript.

HOW A SERVER IS FOUND
  1. The environment variable of the language (REFAC_GOPLS, REFAC_RUST_ANALYZER, REFAC_PYTHON_SERVER,
     REFAC_DART, REFAC_KOTLIN_SERVER). If it is set it is the only place looked at, so a wrong value
     is reported as wrong instead of silently replaced.
  2. Otherwise the usual places for that tool: the PATH, the toolchain of the project (rustup picks
     rust-analyzer from --project-path), and the language's own install folders.

EXIT CODE
  `refac doctor` alone: 0. `refac doctor <language>`: 0 when the server is ready, 1 when it is missing
  or broken (the steps are printed first). A script or agent can run it, branch on the code and
  follow the printed steps.

WHAT YOU GET BACK
  Readable text, or with --json a list with one object per language: language, server, status (ready,
  missing or broken), used_for, env_var, executable, version, looked (every place and what was there),
  requirements, start_check (only for `doctor <language>`) and install (the steps)."#;

pub(in crate::cli) const AFTER_LONG_HELP: &str = r#"EXAMPLES
  # Which servers are ready on this machine?
  refac doctor

  # Go's gopls is missing: the steps to install it, and a check that it starts
  refac doctor go

  # Rust: rustup chooses rust-analyzer by the project's toolchain
  refac doctor rust --project-path /my/cargo-workspace

  # For a script
  refac doctor kotlin --json

The Kotlin server is a large download (about 1.2 GB unpacked) and holds 1.3 to 1.8 GB of memory while a
command runs; the others are small."#;
