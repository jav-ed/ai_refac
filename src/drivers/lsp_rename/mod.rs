//! One semantic symbol rename for every language whose server speaks the
//! Language Server Protocol. The server finds every reference; this engine
//! makes the rename safe: it validates the project and the new name, plans all
//! edits without writing, proves them faithful in memory, and only then
//! writes, with an undo journal. A language is a `Language`: its identifiers,
//! the server it starts, and the few edits it expects outside a reference.

pub mod apply;
pub mod comments;
pub mod discover;
pub mod edits;
mod family;
pub mod journal;
pub mod language;
pub mod languages;
mod leftovers;
mod names;
pub mod project_server;
mod related;
pub mod server;
#[cfg(test)]
mod test_language;
mod verify;

use crate::drivers::symbol_rename::{RenameReport, RenameRequest};
use crate::drivers::symbol_scan;
use anyhow::{Context, Result, bail};
use discover::{Candidate, RenamePlan};
use journal::FileWrite;
use language::{FollowUps, Language};
use server::RenameServer;
use std::path::{Path, PathBuf};

const BOM: char = '\u{FEFF}';

/// The whole rename: check the request, start the language's server, plan and
/// verify, stop the server, write. The server only lives while it is needed.
pub async fn rename_symbol(
    language: &dyn Language,
    request: RenameRequest,
) -> Result<RenameReport> {
    names::validate(language, &request.symbol, &request.new_name)?;
    if request.column.is_some() && request.line.is_none() {
        bail!("--column needs --line");
    }
    let root = language.project_root(&request.project_path)?;
    let file = names::resolve_file(language, &request.file, &root)?;
    let raw = std::fs::read_to_string(&file)
        .with_context(|| format!("Cannot read {}", file.display()))?;
    // The server counts positions from the first real character.
    let text = raw.strip_prefix(BOM).unwrap_or(&raw);
    let occurrences = symbol_scan::occurrences(
        text,
        &request.symbol,
        request.line,
        request.column,
        |character| language.is_identifier_char(character),
        symbol_scan::line_column(text),
    )?;

    let mut server = language.start(&root, &file).await?;
    let outcome = plan_and_verify(
        &mut *server,
        language,
        &root,
        &file,
        text,
        &occurrences,
        &request,
    )
    .await;
    server.shutdown().await;
    let candidate = outcome?;

    let mut follow_ups = language.follow_ups(&root, &candidate.plan)?;
    follow_ups.notes.extend(leftovers::scan(
        &root,
        language,
        &request.symbol,
        &candidate.plan,
    )?);
    if !request.dry_run {
        apply::apply(&candidate.plan, &follow_ups.writes)?;
    }
    Ok(report(&root, &candidate.plan, follow_ups, request.dry_run))
}

/// Plan against the server, then prove the plan before anything is written.
/// A plan that fails the proof is asked for again as often as the language
/// allows; the documents the server holds are put back first.
async fn plan_and_verify(
    server: &mut dyn RenameServer,
    language: &dyn Language,
    root: &Path,
    file: &Path,
    text: &str,
    occurrences: &[symbol_scan::Occurrence],
    request: &RenameRequest,
) -> Result<Candidate> {
    let attempts = language.rename_attempts();
    for attempt in 1..=attempts {
        server.sync_document(file, text).await?;
        server.settle().await?;
        let candidate = discover::discover(
            server,
            language,
            file,
            text,
            occurrences,
            &request.symbol,
            &request.new_name,
        )
        .await?;
        match verify::verify(server, language, root, &candidate, file, &request.symbol).await {
            Ok(()) => return Ok(candidate),
            Err(error) if attempt < attempts && verify::is_unfaithful(&error) => {
                tracing::warn!("{error}\nAsking the server again ({attempt} of {attempts})");
                for edited in &candidate.plan.files {
                    server
                        .sync_document(&edited.file.path, &edited.file.before)
                        .await?;
                }
                server.settle().await?;
            }
            Err(error) => return Err(error),
        }
    }
    unreachable!("the last attempt always returns")
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
            .map(|write: &FileWrite| (relative(&write.path), write.changes)),
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
