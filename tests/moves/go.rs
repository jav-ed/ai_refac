//! Go moves through gopls.
//!
//! Fixture: tests/fixtures/go/project/  (21 files)
//!
//! Move under test: pkg/utils/format.go  ->  pkg/helpers/format.go
//!
//! Go-specific: moving a file changes its package (directory = package).
//! The driver uses gopls via textDocument/rename on the package name symbol.
//! gopls cascades the rename to:
//!   1. Package declaration in the moved file (package utils → package helpers)
//!   2. All import path strings referencing pkg/utils for symbols in format.go
//!   3. All unaliased call-site qualifiers (utils.X → helpers.X)
//!
//! Aliased imports keep the alias — only the path string changes.
//!   u "pkg/utils" → u "pkg/helpers"   (qualifier u.X is unchanged)
//!
//! Blank imports of unrelated packages (_ "pkg/setup") are untouched.
//!
//! Whole-package rename: gopls treats all files in a directory as one package.
//! Moving format.go to pkg/helpers/ triggers a full package rename — validate.go
//! also moves to pkg/helpers/ and ALL callers (including validate-only ones) get
//! their import paths rewritten to pkg/helpers.

use crate::common;

fn run_move(project: &std::path::Path) -> std::process::Output {
    common::run_cli(&[
        "move",
        "--project-path",
        project.to_str().unwrap(),
        "--source-path",
        project.join("pkg/utils/format.go").to_str().unwrap(),
        "--target-path",
        project.join("pkg/helpers/format.go").to_str().unwrap(),
    ])
}

mod imports;
mod package;
