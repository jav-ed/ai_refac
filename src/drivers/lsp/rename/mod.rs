//! One semantic symbol rename for every language whose server speaks the
//! Language Server Protocol. The server finds every reference; this engine
//! makes the rename safe: it validates the project and the new name, plans all
//! edits without writing, proves them faithful in memory, and only then
//! writes, with an undo journal. A language is a `Language`: its identifiers,
//! the server it starts, and the few edits it expects outside a reference.

mod batch;
mod check;
pub mod language;
pub mod languages;
pub mod plan;
pub mod project_server;
pub mod server;
#[cfg(test)]
mod test_language;
pub mod write;

use crate::drivers::lsp::rename::plan::discover::{Candidate, RenamePlan};
use crate::drivers::lsp::rename::write::journal::FileWrite;
use crate::drivers::symbol::rename::{RenameReport, RenameRequest};
use crate::drivers::symbol::scan;
use anyhow::{Context, Result};
use language::{FollowUps, Language};
use server::RenameServer;
use std::path::{Path, PathBuf};

const BOM: char = '\u{FEFF}';

/// The whole rename: check the request, start the language's server, plan and
/// verify, stop the server, write. The server only lives while it is needed.
/// It is a batch of one.
pub async fn rename_symbol(
    language: &dyn Language,
    request: RenameRequest,
) -> Result<RenameReport> {
    let mut reports = rename_symbols(language, vec![request]).await?;
    reports.pop().context("The rename produced no report")
}

/// Several renames of one project in one server session, all or nothing; see
/// `batch`. The reports come back in the order of the requests.
pub async fn rename_symbols(
    language: &dyn Language,
    requests: Vec<RenameRequest>,
) -> Result<Vec<RenameReport>> {
    batch::rename_all(language, requests).await
}

/// Where a rename starts: the file, its text as the server counts positions
/// (without a BOM) and every place the symbol is written at the requested
/// position. Read at the time the rename runs, so that it sees what an earlier
/// rename of a batch wrote.
struct Located {
    file: PathBuf,
    text: String,
    occurrences: Vec<scan::Occurrence>,
}

fn locate(language: &dyn Language, root: &Path, request: &RenameRequest) -> Result<Located> {
    let file =
        crate::drivers::lsp::rename::plan::names::resolve_file(language, &request.file, root)?;
    let raw = crate::drivers::symbol::view::read_to_string(&file)
        .with_context(|| format!("Cannot read {}", file.display()))?;
    // The server counts positions from the first real character.
    let text = raw.strip_prefix(BOM).unwrap_or(&raw).to_string();
    let occurrences = scan::occurrences(
        &text,
        &request.symbol,
        request.line,
        request.column,
        |character| language.is_identifier_char(character),
        scan::line_column(&text),
    )?;
    Ok(Located {
        file,
        text,
        occurrences,
    })
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
    occurrences: &[scan::Occurrence],
    request: &RenameRequest,
) -> Result<Candidate> {
    let attempts = language.rename_attempts();
    for attempt in 1..=attempts {
        server.sync_document(file, text).await?;
        server.settle().await?;
        let candidate = crate::drivers::lsp::rename::plan::discover::discover(
            server,
            language,
            file,
            text,
            occurrences,
            &request.symbol,
            &request.new_name,
        )
        .await?;
        match crate::drivers::lsp::rename::check::verify::verify(
            server,
            language,
            root,
            &candidate,
            file,
            &request.symbol,
        )
        .await
        {
            Ok(()) => return Ok(candidate),
            Err(error)
                if attempt < attempts
                    && crate::drivers::lsp::rename::check::verify::is_unfaithful(&error) =>
            {
                tracing::warn!("{error}\nAsking the server again ({attempt} of {attempts})");
                put_back(server, &candidate.plan).await?;
            }
            Err(error) => return Err(error),
        }
    }
    unreachable!("the last attempt always returns")
}

/// Show the server the files a plan edited as they are on disk again, so that
/// its next answer is not about the renamed text it was shown to prove the plan.
async fn put_back(server: &mut dyn RenameServer, plan: &RenamePlan) -> Result<()> {
    for edited in &plan.files {
        server
            .sync_document(&edited.file.path, &edited.file.before)
            .await?;
    }
    server.settle().await
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
