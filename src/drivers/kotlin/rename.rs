//! Semantic symbol rename for Kotlin, driven by the JetBrains Kotlin language
//! server. The server finds every reference; this module makes the rename
//! safe: it validates the project and the new name, plans all edits without
//! writing, proves them faithful in memory, repairs what the server never
//! touches (Android XML), and only then writes, with an undo journal.

mod plan;
mod verify;

use super::android;
use super::journal::{FileWrite, Journal};
use super::moved::MovedFile;
use super::project::gradle_root;
use super::renames;
use super::server::{self, KotlinServer};
use super::stale;
use super::survey::survey;
pub use crate::drivers::symbol_rename::{RenameReport, RenameRequest};
use crate::drivers::symbol_scan::{self, Occurrence};
use anyhow::{Context, Result, bail};
use plan::{Candidate, RenamePlan};
use std::path::{Path, PathBuf};

const BOM: char = '\u{FEFF}';

/// Words that cannot name a symbol (hard keywords).
const RESERVED_WORDS: &[&str] = &[
    "as",
    "break",
    "class",
    "continue",
    "do",
    "else",
    "false",
    "for",
    "fun",
    "if",
    "in",
    "interface",
    "is",
    "null",
    "object",
    "package",
    "return",
    "super",
    "this",
    "throw",
    "true",
    "try",
    "typealias",
    "typeof",
    "val",
    "var",
    "when",
    "while",
];

fn is_identifier_char(character: char) -> bool {
    character == '_' || character.is_alphanumeric()
}

fn validate_names(symbol: &str, new_name: &str) -> Result<()> {
    let mut chars = new_name.chars();
    let starts_well = chars
        .next()
        .is_some_and(|first| first == '_' || first.is_alphabetic());
    if !starts_well || !chars.all(is_identifier_char) {
        bail!(
            "`{new_name}` is not a plain identifier (letters, digits and underscores; backtick names are not supported)"
        );
    }
    if RESERVED_WORDS.contains(&new_name) {
        bail!("`{new_name}` is a Kotlin keyword and cannot name a symbol");
    }
    if symbol == new_name {
        bail!("The new name equals the current name `{symbol}`");
    }
    Ok(())
}

pub async fn rename_symbol(request: RenameRequest) -> Result<RenameReport> {
    validate_names(&request.symbol, &request.new_name)?;
    if request.column.is_some() && request.line.is_none() {
        bail!("--column needs --line");
    }
    let root = gradle_root(Some(&request.project_path))?;
    let file = resolve_file(&request.file, &root)?;
    let raw = std::fs::read_to_string(&file)
        .with_context(|| format!("Cannot read {}", file.display()))?;
    // The server counts positions from the first real character.
    let text = raw.strip_prefix(BOM).unwrap_or(&raw);
    let occurrences = symbol_scan::occurrences(
        text,
        &request.symbol,
        request.line,
        request.column,
        is_identifier_char,
        symbol_scan::line_column(text),
    )?;

    let install = server::locate()?;
    let mut server = KotlinServer::start(&install, &root).await?;
    let outcome = plan_and_verify(&mut server, &file, text, &occurrences, &request).await;
    server.shutdown().await;
    let candidate = outcome?;

    let follow_ups = follow_ups(&root, &candidate.plan)?;
    if !request.dry_run {
        apply(&candidate.plan, &follow_ups.writes)?;
    }
    Ok(report(&root, &candidate.plan, follow_ups, request.dry_run))
}

fn resolve_file(file: &Path, root: &Path) -> Result<PathBuf> {
    let absolute = if file.is_absolute() {
        file.to_path_buf()
    } else {
        root.join(file)
    };
    let file = absolute
        .canonicalize()
        .with_context(|| format!("File does not exist: {}", absolute.display()))?;
    if !file.starts_with(root) {
        bail!(
            "{} is outside the project {}",
            file.display(),
            root.display()
        );
    }
    if file.extension().and_then(|extension| extension.to_str()) != Some("kt") {
        bail!(
            "Kotlin symbol rename needs a .kt file; got {}",
            file.display()
        );
    }
    Ok(file)
}

/// Plan against the server, then prove the plan before anything is written.
async fn plan_and_verify(
    server: &mut KotlinServer,
    file: &Path,
    text: &str,
    occurrences: &[Occurrence],
    request: &RenameRequest,
) -> Result<Candidate> {
    server.sync_document(file, text).await?;
    let candidate = plan::discover(server, file, text, occurrences, &request.new_name).await?;
    verify::verify(server, &candidate, &candidate.plan, file, &request.symbol).await?;
    Ok(candidate)
}

/// What the rename implies beyond the server's edits: Android XML that names
/// a renamed class, and old names left where refac does not edit.
struct FollowUps {
    writes: Vec<FileWrite>,
    notes: Vec<String>,
}

fn follow_ups(root: &Path, plan: &RenamePlan) -> Result<FollowUps> {
    let moved: Vec<MovedFile> = plan
        .files
        .iter()
        .filter(|edited| edited.file.path.extension().and_then(|e| e.to_str()) == Some("kt"))
        .map(|edited| MovedFile {
            from: edited.file.path.clone(),
            to: new_path(&edited.file.path, plan),
            before: edited.file.before.clone(),
            after: edited.file.text.clone(),
        })
        .collect();
    let renames = renames::collect(&moved)?;
    let files = survey(root)?;
    let writes = android::plan(&files, &moved, &renames.classes)?;
    // Files already decided, so the scan reads them as they will be.
    let decided: Vec<FileWrite> = writes
        .iter()
        .cloned()
        .chain(plan.files.iter().map(|edited| FileWrite {
            path: edited.file.path.clone(),
            bytes: edited.file.bytes.clone(),
            changes: edited.edits.len(),
        }))
        .collect();
    let mut notes: Vec<String> = plan
        .moves
        .iter()
        .map(|(from, to)| {
            format!(
                "{} is renamed to {} together with its class",
                from.strip_prefix(root).unwrap_or(from).display(),
                to.strip_prefix(root).unwrap_or(to).display()
            )
        })
        .collect();
    notes.extend(stale::scan(root, &files, &renames, &decided)?);
    Ok(FollowUps { writes, notes })
}

fn new_path(path: &Path, plan: &RenamePlan) -> PathBuf {
    plan.moves
        .iter()
        .find(|(from, _)| from == path)
        .map_or_else(|| path.to_path_buf(), |(_, to)| to.clone())
}

/// The server needs a long time to answer. A file edited meanwhile would be
/// overwritten by the plan made from its old text, so every planned file must
/// still read as it did when the plan was made.
fn ensure_unchanged(plan: &RenamePlan) -> Result<()> {
    for edited in &plan.files {
        let path = &edited.file.path;
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("Cannot read {} before writing", path.display()))?;
        if raw.strip_prefix(BOM).unwrap_or(&raw) != edited.file.before {
            bail!(
                "{} changed while the rename was being planned. Nothing was written; run the rename again.",
                path.display()
            );
        }
    }
    Ok(())
}

/// Write everything through the journal; a failure restores the project.
fn apply(plan: &RenamePlan, writes: &[FileWrite]) -> Result<()> {
    ensure_unchanged(plan)?;
    let mut journal = Journal::default();
    let outcome: Result<()> = (|| {
        for edited in &plan.files {
            journal.write_file(&edited.file.path, &edited.file.bytes)?;
        }
        journal.write_all(writes)?;
        for (from, to) in &plan.moves {
            journal.move_path(from, to)?;
        }
        Ok(())
    })();
    match outcome {
        Ok(()) => Ok(()),
        Err(error) => match journal.rollback() {
            Ok(()) => Err(error.context("The rename failed and every change was undone")),
            Err(failure) => Err(error.context(format!("{failure:#}"))),
        },
    }
}

fn report(root: &Path, plan: &RenamePlan, follow_ups: FollowUps, dry_run: bool) -> RenameReport {
    let relative = |path: &Path| path.strip_prefix(root).unwrap_or(path).to_path_buf();
    let mut files: Vec<(PathBuf, usize)> = plan
        .files
        .iter()
        .map(|edited| (relative(&edited.file.path), edited.edits.len()))
        .collect();
    files.extend(
        follow_ups
            .writes
            .iter()
            .map(|write| (relative(&write.path), write.changes)),
    );
    files.sort();
    RenameReport {
        edits: files.iter().map(|(_, edits)| edits).sum(),
        files,
        dry_run,
        notes: follow_ups.notes,
    }
}

#[cfg(test)]
mod tests;
