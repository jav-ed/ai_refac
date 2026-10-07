//! Running a [`MovePlan`] against the Kotlin server: one
//! `workspace/willRenameFiles` request per group, edits applied, files moved,
//! and the server told what changed on disk before the next group. Every
//! write and move is journaled, so a failure restores the project.

use super::checks::{Check, check_moved_file};
use super::edits::{PlannedFile, parse_workspace_edit, plan_files};
use super::journal::Journal;
use super::plan::{self, Group, MovePlan, Step};
use super::project::gradle_root;
use super::server::{self, KotlinServer};
use crate::drivers::lsp_session::RpcError;
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use url::Url;
use walkdir::WalkDir;

#[derive(Debug, Default)]
pub struct MoveReport {
    /// Files whose content changed, at their final location.
    pub edited: Vec<PathBuf>,
    pub notes: Vec<String>,
}

/// Move Kotlin files and directories, updating packages and references.
pub async fn move_files(files: &[(String, String)], root: Option<&Path>) -> Result<MoveReport> {
    let gradle = gradle_root(root)?;
    let plan = plan::build(files, &gradle)?;
    let install = server::locate()?;
    let mut server = KotlinServer::start(&install, &gradle).await?;
    let mut journal = Journal::default();
    let outcome = run(&mut server, &plan, &mut journal).await;
    server.shutdown().await;
    match outcome {
        Ok(report) => Ok(report),
        Err(error) => match journal.rollback() {
            Ok(()) => Err(error.context("The move failed and every change was undone")),
            Err(failure) => Err(error.context(format!("{failure:#}"))),
        },
    }
}

async fn run(
    server: &mut KotlinServer,
    plan: &MovePlan,
    journal: &mut Journal,
) -> Result<MoveReport> {
    let mut report = MoveReport {
        notes: plan.notes.clone(),
        ..MoveReport::default()
    };
    for group in &plan.groups {
        run_group(server, group, journal, &mut report)
            .await
            .with_context(|| describe(group))?;
    }
    Ok(report)
}

async fn run_group(
    server: &mut KotlinServer,
    group: &Group,
    journal: &mut Journal,
    report: &mut MoveReport,
) -> Result<()> {
    let files: Vec<Value> = group
        .steps
        .iter()
        .map(|step| Ok(json!({ "oldUri": uri(&step.from)?, "newUri": uri(&step.to)? })))
        .collect::<Result<_>>()?;
    let answer = server
        .request("workspace/willRenameFiles", json!({ "files": files }))
        .await
        .map_err(refusal)?;
    // `null` means the server has nothing to change. That is right for a
    // rename that no code refers to, and caught by the package checks below
    // when a move needed edits it did not send.
    let edits = if answer.is_null() {
        Vec::new()
    } else {
        parse_workspace_edit(&answer)?
    };
    let planned = plan_files(edits, |path| locate(path, &group.steps))?;
    report.notes.extend(verify_packages(group, &planned)?);

    for file in &planned {
        journal.write_file(&file.path, &file.bytes)?;
    }
    for step in &group.steps {
        journal.move_path(&step.from, &step.to)?;
    }
    let edited: Vec<PathBuf> = planned
        .iter()
        .map(|file| relocate(&file.path, &group.steps))
        .collect();
    sync(server, group, &edited).await?;
    report.edited.extend(edited);
    Ok(())
}

/// The path the server named, as it exists on disk now. An edit may address a
/// file under its destination before the file has been moved there.
fn locate(path: &Path, steps: &[Step]) -> PathBuf {
    if path.exists() {
        return path.to_path_buf();
    }
    for step in steps {
        if let Ok(rest) = path.strip_prefix(&step.to) {
            return join_relative(&step.from, rest);
        }
    }
    path.to_path_buf()
}

/// `Path::join` with an empty path appends a trailing separator, which turns
/// a file into a bogus directory path.
fn join_relative(base: &Path, rest: &Path) -> PathBuf {
    if rest.as_os_str().is_empty() {
        base.to_path_buf()
    } else {
        base.join(rest)
    }
}

/// The path of a file after the steps have been carried out.
fn relocate(path: &Path, steps: &[Step]) -> PathBuf {
    for step in steps {
        if let Ok(rest) = path.strip_prefix(&step.from) {
            return join_relative(&step.to, rest);
        }
    }
    path.to_path_buf()
}

/// Every moved source file must end up in the package its directory stands for.
fn verify_packages(group: &Group, planned: &[PlannedFile]) -> Result<Vec<String>> {
    let edited: HashMap<&Path, &str> = planned
        .iter()
        .map(|file| (file.path.as_path(), file.text.as_str()))
        .collect();
    let mut notes = Vec::new();
    for step in &group.steps {
        for (from, to) in source_files(step)? {
            let before = std::fs::read_to_string(&from)
                .with_context(|| format!("Cannot read {}", from.display()))?;
            let after = edited.get(from.as_path()).copied().unwrap_or(&before);
            if let Check::Unverified(note) = check_moved_file(&from, &to, &before, after)? {
                notes.push(note);
            }
        }
    }
    Ok(notes)
}

/// The Kotlin and Java files of a step with the location each one moves to.
fn source_files(step: &Step) -> Result<Vec<(PathBuf, PathBuf)>> {
    if !step.is_dir {
        return Ok(vec![(step.from.clone(), step.to.clone())]);
    }
    let mut files = Vec::new();
    for entry in WalkDir::new(&step.from) {
        let path = entry?.into_path();
        if matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("kt" | "java")
        ) {
            let target = step.to.join(path.strip_prefix(&step.from)?);
            files.push((path, target));
        }
    }
    Ok(files)
}

/// Tell the server what happened on disk. The file watcher event alone is
/// handled asynchronously by the server, so a request sent right after it can
/// still see the old project. Documents sent with their text are known the
/// moment the notification is read, in order, before the next request.
async fn sync(server: &mut KotlinServer, group: &Group, edited: &[PathBuf]) -> Result<()> {
    const CREATED: u8 = 1;
    const CHANGED: u8 = 2;
    const DELETED: u8 = 3;
    let mut changes = Vec::new();
    for step in &group.steps {
        server.close_under(&step.from).await?;
        changes.push(json!({ "uri": uri(&step.from)?, "type": DELETED }));
        changes.push(json!({ "uri": uri(&step.to)?, "type": CREATED }));
        if !step.is_dir {
            continue;
        }
        for entry in WalkDir::new(&step.to).min_depth(1) {
            let created = entry?.into_path();
            let old = step.from.join(created.strip_prefix(&step.to)?);
            changes.push(json!({ "uri": uri(&old)?, "type": DELETED }));
            changes.push(json!({ "uri": uri(&created)?, "type": CREATED }));
        }
    }
    for path in edited {
        changes.push(json!({ "uri": uri(path)?, "type": CHANGED }));
    }
    server
        .notify(
            "workspace/didChangeWatchedFiles",
            json!({ "changes": changes }),
        )
        .await?;

    let mut sources: Vec<PathBuf> = edited.to_vec();
    for step in &group.steps {
        for entry in WalkDir::new(&step.to) {
            sources.push(entry?.into_path());
        }
    }
    sources.retain(|path| {
        matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("kt" | "java")
        )
    });
    sources.sort();
    sources.dedup();
    for path in sources {
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("Cannot read {}", path.display()))?;
        server.sync_document(&path, &text).await?;
    }
    Ok(())
}

fn uri(path: &Path) -> Result<String> {
    Ok(Url::from_file_path(path)
        .map_err(|_| anyhow::anyhow!("Cannot make a URI of {}", path.display()))?
        .to_string())
}

/// A refusal from the server is an expected outcome, so it is explained
/// rather than reported as a crash.
fn refusal(error: anyhow::Error) -> anyhow::Error {
    match error.downcast_ref::<RpcError>() {
        Some(rpc) => anyhow::anyhow!("The Kotlin language server refused the move: {rpc}"),
        None => error,
    }
}

fn describe(group: &Group) -> String {
    let steps: Vec<String> = group
        .steps
        .iter()
        .map(|step| format!("{} -> {}", step.from.display(), step.to.display()))
        .collect();
    format!("Moving {}", steps.join(", "))
}

#[cfg(test)]
mod tests;
