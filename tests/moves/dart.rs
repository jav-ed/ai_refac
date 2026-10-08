//! Dart moves through the analysis server.
//!
//! The Dart analysis server is sensitive to concurrent starts: multiple instances
//! running simultaneously can interfere, causing flaky failures under full-suite load.
//! Serialise all tests in this binary with a global mutex.
//!
//! Fixture: tests/fixtures/dart/project/  (20 files)
//!
//! Move under test: lib/src/formatter.dart  ->  lib/src/core/formatter.dart
//!
//! The driver uses the Dart analysis server via workspace/willRenameFiles LSP.
//! The server handles all URI rewriting — both package: and relative imports,
//! plus export directives in barrel files.
//!
//! Patterns exercised:
//!   - Barrel export directive (lib/acme_utils.dart) — export URI updated
//!   - package: import — updated when imported file moves (validator.dart, order.dart, etc.)
//!   - package: import with show combinator — URI updates, combinator survives (item.dart)
//!   - Relative import from same directory (service.dart): 'formatter.dart' → 'core/formatter.dart'
//!   - Relative import from subdirectory (models/, utils/, network/, cache/): '../formatter.dart'
//!   - Relative import + as alias (service.dart as fmt, api_client.dart as f)
//!   - dart: SDK imports must NOT be rewritten (dart:io, dart:convert)
//!   - Control files with no formatter dep: byte-identical after move

use crate::common;

fn run_move(project: &std::path::Path) -> std::process::Output {
    common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("lib/src/formatter.dart").to_str().unwrap(),
        "--target-path",
        project
            .join("lib/src/core/formatter.dart")
            .to_str()
            .unwrap(),
    ])
}

mod imports;
mod untouched;
