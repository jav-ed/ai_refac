//! Semantic symbol rename for TypeScript/JavaScript, driven by the TypeScript 7
//! native language server. The server finds every reference; this module makes
//! the rename safe: it validates the project and the new name, plans all edits
//! without writing, proves them faithful in memory, and only then writes.

mod apply;
mod batch;
mod edits;
mod engine;
mod locate;
mod plan;
mod session;
mod verify;

pub use crate::drivers::symbol::rename::{RenameReport, RenameRequest};
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

const RESERVED_WORDS: &[&str] = &[
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "interface",
    "let",
    "new",
    "null",
    "package",
    "private",
    "protected",
    "public",
    "return",
    "static",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

fn validate_names(symbol: &str, new_name: &str) -> Result<()> {
    let mut chars = new_name.chars();
    let starts_well = chars
        .next()
        .is_some_and(|first| first == '$' || first == '_' || first.is_alphabetic());
    if !starts_well || !chars.all(locate::is_identifier_char) {
        bail!("`{new_name}` is not a valid identifier");
    }
    if RESERVED_WORDS.contains(&new_name) {
        bail!("`{new_name}` is a reserved word and cannot name a symbol");
    }
    if symbol == new_name {
        bail!("The new name equals the current name `{symbol}`");
    }
    Ok(())
}

/// Path of the TypeScript 7 native executable, installing it when missing.
/// Tests use it to prove a renamed project still typechecks.
pub async fn native_executable() -> Result<PathBuf> {
    engine::locate().await
}

/// One rename: a batch of one, with its messages as they are.
pub async fn rename_symbol(request: RenameRequest) -> Result<RenameReport> {
    let mut reports = rename_symbols(vec![request]).await?;
    reports.pop().context("The rename produced no report")
}

/// Several renames of one project in one engine session, all or nothing; see
/// `batch`. The reports come back in the order of the requests.
pub use batch::rename_symbols;

async fn plan_and_verify(
    session: &mut session::Session,
    request: &RenameRequest,
    file: &Path,
    project: &Path,
) -> Result<plan::Plan> {
    let text = crate::drivers::symbol::view::read_to_string(file)
        .with_context(|| format!("Cannot read {}", file.display()))?;
    // The engine reads files without their BOM, so positions must exclude it.
    let text = text.strip_prefix('\u{FEFF}').unwrap_or(&text);
    let matches = locate::occurrences(text, &request.symbol, request.line, request.column)?;
    session.sync_document(file, text).await?;
    let plan = plan::plan_rename(session, file, text, &matches, &request.new_name, project).await?;
    verify::verify(session, &plan, &request.new_name).await?;
    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::validate_names;

    #[test]
    fn accepts_identifiers_and_rejects_everything_else() {
        assert!(validate_names("total", "grandTotal").is_ok());
        assert!(validate_names("total", "$élan_2").is_ok());
        assert!(validate_names("total", "2fast").is_err());
        assert!(validate_names("total", "has space").is_err());
        assert!(validate_names("total", "").is_err());
        assert!(validate_names("total", "class").is_err());
        assert!(validate_names("total", "total").is_err());
    }
}
