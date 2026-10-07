//! Go: gopls. It checks name conflicts itself and edits doc comments that
//! mention the symbol. Renaming a package changes directories, which is a
//! move.

use super::start_server;
use crate::drivers::lsp_rename::comments;
use crate::drivers::lsp_rename::language::{EditedText, Language};
use crate::drivers::lsp_rename::server::RenameServer;
use anyhow::{Result, bail};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// The 25 keywords of the language.
const KEYWORDS: &[&str] = &[
    "break",
    "case",
    "chan",
    "const",
    "continue",
    "default",
    "defer",
    "else",
    "fallthrough",
    "for",
    "func",
    "go",
    "goto",
    "if",
    "import",
    "interface",
    "map",
    "package",
    "range",
    "return",
    "select",
    "struct",
    "switch",
    "type",
    "var",
];

pub struct Go;

#[async_trait]
impl Language for Go {
    fn name(&self) -> &'static str {
        "Go"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["go"]
    }

    fn reserved_words(&self) -> &'static [&'static str] {
        KEYWORDS
    }

    /// gopls loads the module (or workspace) it is started in.
    fn project_root(&self, project_path: &Path) -> Result<PathBuf> {
        let root = project_path.canonicalize().map_err(|error| {
            anyhow::anyhow!(
                "Cannot read the project path {}: {error}",
                project_path.display()
            )
        })?;
        if !root.join("go.mod").is_file() && !root.join("go.work").is_file() {
            bail!(
                "{} has no go.mod or go.work. Pass the module root as --project-path.",
                root.display()
            );
        }
        Ok(root)
    }

    async fn start(&self, root: &Path, file: &Path) -> Result<Box<dyn RenameServer>> {
        start_server("go", root, file).await
    }

    /// gopls also rewrites the name in doc comments that start with it.
    fn exempt_edits(&self, edited: &EditedText) -> Vec<bool> {
        comments::on_comment_lines(edited, &["//"])
    }

    /// Measured: under CPU load gopls answered about one rename in four with
    /// a part of the edits (the implementing methods and the test variant of
    /// the package missing), while its references stayed complete. Asking
    /// again gets the whole answer.
    fn rename_attempts(&self) -> usize {
        4
    }

    fn refuse_file_operations(&self) -> Option<&'static str> {
        Some(
            "A Go package is renamed by moving its folder: use `refac move` with the package directory",
        )
    }
}
