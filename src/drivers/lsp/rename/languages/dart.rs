//! Dart: the analysis server that ships in the Dart SDK
//! (`dart language-server`). It resolves `package:` imports through
//! `.dart_tool/package_config.json`; without that file it cannot see the
//! files that import the symbol, so the rename would be incomplete and the
//! project is refused until `dart pub get` has run.

use super::start_server;
use crate::drivers::lsp::rename::language::{EditedText, Language};
use crate::drivers::lsp::rename::plan::comments;
use crate::drivers::lsp::rename::server::RenameServer;
use anyhow::{Result, bail};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// The reserved words, which cannot name anything.
const KEYWORDS: &[&str] = &[
    "assert", "break", "case", "catch", "class", "const", "continue", "default", "do", "else",
    "enum", "extends", "false", "final", "finally", "for", "if", "in", "is", "new", "null",
    "rethrow", "return", "super", "switch", "this", "throw", "true", "try", "var", "void", "while",
    "with",
];

pub struct Dart;

#[async_trait]
impl Language for Dart {
    fn name(&self) -> &'static str {
        "Dart"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["dart"]
    }

    fn is_identifier_char(&self, character: char) -> bool {
        character == '_' || character == '$' || character.is_alphanumeric()
    }

    fn reserved_words(&self) -> &'static [&'static str] {
        KEYWORDS
    }

    fn project_root(&self, project_path: &Path) -> Result<PathBuf> {
        let root = project_path.canonicalize().map_err(|error| {
            anyhow::anyhow!(
                "Cannot read the project path {}: {error}",
                project_path.display()
            )
        })?;
        if !root.join("pubspec.yaml").is_file() {
            bail!(
                "{} has no pubspec.yaml. Pass the Dart package root as --project-path.",
                root.display()
            );
        }
        if !root.join(".dart_tool/package_config.json").is_file() {
            bail!(
                "{} has no .dart_tool/package_config.json, so the analysis server cannot resolve `package:` imports and would miss the files that import the symbol. Run `dart pub get` (or `flutter pub get`) in it and repeat the command.",
                root.display()
            );
        }
        Ok(root)
    }

    async fn start(&self, root: &Path, file: &Path) -> Result<Box<dyn RenameServer>> {
        start_server("dart", root, file).await
    }

    /// The server renames `[Name]` references in doc comments, and may edit
    /// comment lines that quote the name.
    fn exempt_edits(&self, edited: &EditedText) -> Vec<bool> {
        comments::on_comment_lines(edited, &["//"])
    }

    /// A package or file name in an import is a path, not a symbol.
    fn not_a_symbol_hint(&self) -> Option<&'static str> {
        Some(
            "If it is part of an import path, it names a file or package: rename the file with `refac move`, which also updates the imports",
        )
    }

    fn refuse_file_operations(&self) -> Option<&'static str> {
        Some(
            "refac rename edits symbols inside files; to rename or move a Dart file use `refac move`",
        )
    }
}
