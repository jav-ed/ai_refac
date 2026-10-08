//! Python: basedpyright (or pyright). Python is typed only where the code
//! says so, so the server renames what it can prove and leaves the rest: a
//! method called on an untyped parameter keeps its old name. The engine's
//! leftover note lists those places; here the language only says what a name
//! may be and where the project starts.

use super::start_server;
use crate::drivers::lsp::rename::language::Language;
use crate::drivers::lsp::rename::server::RenameServer;
use anyhow::{Result, bail};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// The hard keywords. `match`, `case`, `type` and `_` are soft keywords and
/// still name variables.
const KEYWORDS: &[&str] = &[
    "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class", "continue",
    "def", "del", "elif", "else", "except", "finally", "for", "from", "global", "if", "import",
    "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try", "while",
    "with", "yield",
];

pub struct Python;

/// `__init__`, `__str__`: names the interpreter calls by spelling.
fn is_dunder(name: &str) -> bool {
    name.len() > 4 && name.starts_with("__") && name.ends_with("__")
}

#[async_trait]
impl Language for Python {
    fn name(&self) -> &'static str {
        "Python"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["py"]
    }

    fn reserved_words(&self) -> &'static [&'static str] {
        KEYWORDS
    }

    /// Python has no file that marks a project, so the path given is the
    /// root; it is where pyright reads `pyrightconfig.json` or
    /// `[tool.pyright]` and where imports are resolved from.
    fn project_root(&self, project_path: &Path) -> Result<PathBuf> {
        let root = project_path.canonicalize().map_err(|error| {
            anyhow::anyhow!(
                "Cannot read the project path {}: {error}",
                project_path.display()
            )
        })?;
        if !root.is_dir() {
            bail!("{} is not a directory", root.display());
        }
        Ok(root)
    }

    async fn start(&self, root: &Path, file: &Path) -> Result<Box<dyn RenameServer>> {
        start_server("python", root, file).await
    }

    /// basedpyright lists the overrides of a method (`textDocument/implementation`)
    /// but renames only the method it is asked about.
    fn renames_overrides(&self) -> bool {
        true
    }

    /// A module name in an import is the name of a file or folder.
    fn not_a_symbol_hint(&self) -> Option<&'static str> {
        Some(
            "If it is the name of a module or package (a name inside an import path), it is a file or folder: rename it with `refac move`, which also updates the imports",
        )
    }

    /// The interpreter calls `__init__`, `__enter__` and the rest by their
    /// spelling and nothing in the code mentions them, so a rename would
    /// silently change what the program does.
    fn refuse_names(&self, symbol: &str, new_name: &str) -> Option<String> {
        if is_dunder(symbol) {
            return Some(format!(
                "`{symbol}` is a special method the Python interpreter calls by its name; renaming it silently changes what the program does"
            ));
        }
        if is_dunder(new_name) {
            return Some(format!(
                "`{new_name}` is a spelling the Python interpreter gives a meaning to; pick a name without double underscores on both sides"
            ));
        }
        None
    }
}
