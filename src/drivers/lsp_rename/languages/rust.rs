//! Rust: rust-analyzer. It renames through macros, shorthand fields and
//! `use` trees. Renaming a module renames its files, which is a move.

mod macros;

use super::start_server;
use crate::drivers::lsp_rename::comments;
use crate::drivers::lsp_rename::language::{EditedText, Language, UnrenamedPlace};
use crate::drivers::lsp_rename::server::RenameServer;
use anyhow::{Result, bail};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// Strict and reserved keywords of the 2024 edition.
const KEYWORDS: &[&str] = &[
    "Self", "abstract", "as", "async", "await", "become", "box", "break", "const", "continue",
    "crate", "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "gen", "if",
    "impl", "in", "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub",
    "ref", "return", "self", "static", "struct", "super", "trait", "true", "try", "type", "typeof",
    "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
];

pub struct Rust;

#[async_trait]
impl Language for Rust {
    fn name(&self) -> &'static str {
        "Rust"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["rs"]
    }

    fn reserved_words(&self) -> &'static [&'static str] {
        KEYWORDS
    }

    /// rust-analyzer loads the Cargo package or workspace it is started in.
    fn project_root(&self, project_path: &Path) -> Result<PathBuf> {
        let root = project_path.canonicalize().map_err(|error| {
            anyhow::anyhow!(
                "Cannot read the project path {}: {error}",
                project_path.display()
            )
        })?;
        if !root.join("Cargo.toml").is_file() {
            bail!(
                "{} has no Cargo.toml. Pass the Cargo package or workspace root as --project-path.",
                root.display()
            );
        }
        Ok(root)
    }

    async fn start(&self, root: &Path, file: &Path) -> Result<Box<dyn RenameServer>> {
        start_server("rust", root, file).await
    }

    /// Doc comments that mention the symbol outside a link.
    fn exempt_edits(&self, edited: &EditedText) -> Vec<bool> {
        comments::on_comment_lines(edited, &["//"])
    }

    /// rust-analyzer renames through macro calls, never inside `macro_rules!`.
    fn unrenamed_places(&self, text: &str) -> Vec<UnrenamedPlace> {
        macros::bodies(text)
    }

    fn refuse_file_operations(&self) -> Option<&'static str> {
        Some(
            "A Rust module is renamed by moving it: use `refac move-module crate::old crate::new` (semantic) or `refac move` for the file",
        )
    }
}
