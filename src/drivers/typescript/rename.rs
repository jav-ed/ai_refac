//! Semantic symbol rename for TypeScript/JavaScript, driven by the TypeScript 7
//! native language server. The server finds every reference; this module makes
//! the rename safe: it validates the project and the new name, plans all edits
//! without writing, proves them faithful in memory, and only then writes.

mod apply;
mod edits;
mod engine;
mod locate;
mod plan;
mod session;
mod verify;

use super::process::{self, Limits};
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use sysinfo::Pid;

pub struct RenameRequest {
    pub project_path: PathBuf,
    pub file: PathBuf,
    pub symbol: String,
    pub new_name: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub dry_run: bool,
}

pub struct RenameReport {
    /// Edited files relative to the project, with their edit counts.
    pub files: Vec<(PathBuf, usize)>,
    pub edits: usize,
    pub dry_run: bool,
}

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

pub async fn rename_symbol(request: RenameRequest) -> Result<RenameReport> {
    validate_names(&request.symbol, &request.new_name)?;
    if request.column.is_some() && request.line.is_none() {
        bail!("--column needs --line");
    }
    let project = request.project_path.canonicalize().with_context(|| {
        format!(
            "Project path does not exist: {}",
            request.project_path.display()
        )
    })?;
    if !project.join("tsconfig.json").is_file() {
        bail!(
            "No tsconfig.json in {}. Symbol rename needs the package root with the authoritative tsconfig.",
            project.display()
        );
    }
    let absolute = if request.file.is_absolute() {
        request.file.clone()
    } else {
        project.join(&request.file)
    };
    let file = absolute
        .canonicalize()
        .with_context(|| format!("File does not exist: {}", absolute.display()))?;
    if !file.starts_with(&project) {
        bail!(
            "{} is outside the project {}",
            file.display(),
            project.display()
        );
    }

    let limits = Limits::from_env()?;
    let executable = engine::locate().await?;
    engine::preflight(&executable, &project, &file, limits.timeout).await?;

    let mut session = session::Session::start(&executable, &project).await?;
    let pid = session.pid().context("TypeScript engine has no PID")?;
    // Planning and verification only read; a limit failure here leaves the
    // working tree untouched. Writing happens after this block, uninterrupted.
    let outcome = tokio::select! {
        result = plan_and_verify(&mut session, &request, &file, &project) => result,
        _ = tokio::time::sleep(limits.timeout) => Err(anyhow::anyhow!(
            "TypeScript engine timed out after {} seconds. No files were changed", limits.timeout.as_secs_f64()
        )),
        exceeded = process::memory_limit(Pid::from_u32(pid), limits.rss_bytes) => Err(exceeded
            .err()
            .unwrap_or_else(|| anyhow::anyhow!("TypeScript RAM monitor stopped unexpectedly"))
            .context("No files were changed")),
    };
    session.shutdown().await;
    let plan = outcome?;

    if !request.dry_run {
        apply::apply(&plan)?;
    }
    let files = plan
        .files
        .iter()
        .map(|file| {
            (
                file.path
                    .strip_prefix(&project)
                    .unwrap_or(&file.path)
                    .to_path_buf(),
                file.edits.len(),
            )
        })
        .collect();
    Ok(RenameReport {
        files,
        edits: plan.edit_count(),
        dry_run: request.dry_run,
    })
}

async fn plan_and_verify(
    session: &mut session::Session,
    request: &RenameRequest,
    file: &Path,
    project: &Path,
) -> Result<plan::Plan> {
    let text =
        std::fs::read_to_string(file).with_context(|| format!("Cannot read {}", file.display()))?;
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
