//! Several renames in one language-server session. Starting a server costs
//! seconds for most languages and half a minute for Kotlin, and a server kept
//! running between commands would hold its memory and, worse, a view of the
//! files that goes stale whenever anything else edits them. A batch gets the
//! saving without either: the server is started once, every rename is planned,
//! proven and written in order against the files as the previous one left
//! them, and the server is stopped before the command ends.
//!
//! The batch is all or nothing. Each step keeps its undo log, and a step that
//! fails undoes the steps before it, so the caller never has to work out which
//! half of a series was applied. `rename_symbol` is a batch of one.

use super::journal::Journal;
use super::language::Language;
use super::server::RenameServer;
use super::{Located, apply, leftovers, locate, names, plan_and_verify, put_back, report};
use crate::drivers::symbol_rename::{RenameReport, RenameRequest};
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

/// Run every request, in order, in one server session. The reports come back
/// in the same order.
pub async fn rename_all(
    language: &dyn Language,
    requests: Vec<RenameRequest>,
) -> Result<Vec<RenameReport>> {
    let root = check_requests(language, &requests)?;
    // The first rename is looked up before the server starts: a symbol that is
    // not where the request says costs no start. The later ones depend on what
    // the earlier ones write (a Kotlin class and its file), so they are looked
    // up when their turn comes.
    let first = locate(language, &root, &requests[0])
        .map_err(|error| undo(Vec::new(), error, 0, &requests))?;
    let mut server: Option<Box<dyn RenameServer>> = Some(language.start(&root, &first.file).await?);
    let mut first = Some(first);
    let mut journals: Vec<Journal> = Vec::new();
    let mut reports = Vec::new();
    for (index, request) in requests.iter().enumerate() {
        let located = first.take();
        let last = index + 1 == requests.len();
        let step = run_step(
            language,
            &root,
            request,
            located,
            &mut server,
            last,
            &mut journals,
        )
        .await;
        match step {
            Ok(report) => reports.push(report),
            Err(error) => {
                if let Some(server) = server.take() {
                    server.shutdown().await;
                }
                return Err(undo(journals, error, index, &requests));
            }
        }
    }
    Ok(reports)
}

/// Everything that can be checked before the first server starts, for every
/// request, so that a typo in the fifth does not cost a server start. Returns
/// the project root the whole batch works in.
fn check_requests(language: &dyn Language, requests: &[RenameRequest]) -> Result<PathBuf> {
    let Some(first) = requests.first() else {
        bail!("A batch needs at least one rename");
    };
    for (index, request) in requests.iter().enumerate() {
        let which = |error: anyhow::Error| in_batch(error, index, requests);
        names::validate(language, &request.symbol, &request.new_name).map_err(which)?;
        if request.column.is_some() && request.line.is_none() {
            return Err(which(anyhow::anyhow!("--column needs --line")));
        }
        if request.dry_run != first.dry_run {
            return Err(which(anyhow::anyhow!(
                "every rename of a batch is a dry run or none is"
            )));
        }
    }
    let root = language.project_root(&first.project_path)?;
    for (index, request) in requests.iter().enumerate().skip(1) {
        let other = language.project_root(&request.project_path)?;
        if other != root {
            return Err(in_batch(
                anyhow::anyhow!(
                    "a batch works in one project, but this one is in {} and the first is in {}",
                    other.display(),
                    root.display()
                ),
                index,
                requests,
            ));
        }
    }
    Ok(root)
}

/// One rename against the files as they are now. On the last step the server
/// is stopped before the follow-ups and the write, as for a single rename.
async fn run_step(
    language: &dyn Language,
    root: &Path,
    request: &RenameRequest,
    located: Option<Located>,
    server: &mut Option<Box<dyn RenameServer>>,
    last: bool,
    journals: &mut Vec<Journal>,
) -> Result<RenameReport> {
    let located = match located {
        Some(located) => located,
        None => locate(language, root, request)?,
    };
    let live = server
        .as_deref_mut()
        .context("The language server stopped before the batch was done")?;
    let candidate = plan_and_verify(
        live,
        language,
        root,
        &located.file,
        &located.text,
        &located.occurrences,
        request,
    )
    .await?;
    let plan = &candidate.plan;

    if last {
        if let Some(server) = server.take() {
            server.shutdown().await;
        }
    } else if request.dry_run {
        // Nothing is written, so the next rename starts from the files as they
        // are on disk: the server must forget what this one showed it.
        if let Some(live) = server.as_deref_mut() {
            put_back(live, plan).await?;
        }
    }

    let mut follow_ups = language.follow_ups(root, plan)?;
    follow_ups
        .notes
        .extend(leftovers::scan(root, language, &request.symbol, plan)?);
    if !request.dry_run {
        journals.push(apply::apply_undoable(plan, &follow_ups.writes)?);
        if let Some(live) = server.as_deref_mut() {
            let edited: Vec<PathBuf> = plan
                .files
                .iter()
                .map(|file| file.file.path.clone())
                .collect();
            live.after_apply(&edited, &plan.moves).await?;
        }
    }
    Ok(report(root, plan, follow_ups, request.dry_run))
}

/// Take back the steps that were written before the failing one, newest first,
/// and say what happened. A single rename keeps its error as it is.
fn undo(
    journals: Vec<Journal>,
    error: anyhow::Error,
    failed: usize,
    requests: &[RenameRequest],
) -> anyhow::Error {
    if requests.len() == 1 {
        return error;
    }
    let applied = journals.len();
    let mut failures = Vec::new();
    for journal in journals.into_iter().rev() {
        if let Err(failure) = journal.rollback() {
            failures.push(format!("{failure:#}"));
        }
    }
    let outcome = if !failures.is_empty() {
        format!(
            "undoing the {applied} earlier rename(s) failed too, so the project is half refactored; inspect it with git:\n{}",
            failures.join("\n")
        )
    } else if applied == 0 {
        "nothing was changed".to_string()
    } else {
        format!("the {applied} earlier rename(s) were undone, so nothing was changed")
    };
    // Reads outermost first: which rename failed and what became of the
    // batch, then why.
    error.context(format!("{} failed; {outcome}", label(failed, requests)))
}

/// Name the rename of a batch an error belongs to. A batch of one is a plain
/// rename and keeps its messages as they are.
fn in_batch(error: anyhow::Error, index: usize, requests: &[RenameRequest]) -> anyhow::Error {
    if requests.len() == 1 {
        return error;
    }
    error.context(label(index, requests))
}

fn label(index: usize, requests: &[RenameRequest]) -> String {
    let request = &requests[index];
    format!(
        "Rename {} of {} ({} -> {} in {})",
        index + 1,
        requests.len(),
        request.symbol,
        request.new_name,
        request.file.display()
    )
}

#[cfg(test)]
mod tests;
