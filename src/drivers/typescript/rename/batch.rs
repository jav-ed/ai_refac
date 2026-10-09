//! Several TypeScript renames in one engine session. Starting the engine and
//! loading the project is most of the cost of a rename, so a batch starts it
//! once. Every rename is planned, proven and written in order against the files
//! the one before left, and the engine is stopped before the last write.
//!
//! The batch is all or nothing: a rename that fails takes the written renames
//! before it back. A dry run writes nothing and keeps what the earlier renames
//! would have written in a view (see `symbol::view`), so it plans and refuses
//! exactly what the real batch does. `rename_symbol` is a batch of one.

use super::super::process::{self, Limits};
use super::plan::Plan;
use super::{RenameReport, RenameRequest, apply, engine, plan_and_verify, session, validate_names};
use crate::drivers::symbol::batch as batch_errors;
use crate::drivers::symbol::view;
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use sysinfo::Pid;

/// The project of a batch and, for each request, the file it renames in.
struct Checked {
    project: PathBuf,
    files: Vec<PathBuf>,
}

/// Run every request, in order, in one engine session. The reports come back
/// in the same order.
pub async fn rename_symbols(requests: Vec<RenameRequest>) -> Result<Vec<RenameReport>> {
    if requests.first().is_some_and(|first| first.dry_run) {
        view::scoped(run(requests)).await
    } else {
        run(requests).await
    }
}

async fn run(requests: Vec<RenameRequest>) -> Result<Vec<RenameReport>> {
    let checked = check_requests(&requests)?;
    let limits = Limits::from_env()?;
    let executable = engine::locate().await?;
    engine::preflight(
        &executable,
        &checked.project,
        &checked.files,
        limits.timeout,
    )
    .await?;

    let mut session = Some(session::start(&executable, &checked.project).await?);
    let pid = session
        .as_ref()
        .and_then(|session| session.pid())
        .context("TypeScript engine has no PID")?;
    let mut written: Vec<Plan> = Vec::new();
    let mut reports = Vec::new();
    for (index, request) in requests.iter().enumerate() {
        let planned = match session.as_mut() {
            Some(live) => {
                // Planning and verification only read; a limit failure here
                // leaves this rename's files untouched.
                tokio::select! {
                    result = plan_and_verify(live, request, &checked.files[index], &checked.project) => result,
                    _ = tokio::time::sleep(limits.timeout) => Err(anyhow::anyhow!(
                        "TypeScript engine timed out after {} seconds. No files were changed", limits.timeout.as_secs_f64()
                    )),
                    exceeded = process::memory_limit(Pid::from_u32(pid), limits.rss_bytes) => Err(exceeded
                        .err()
                        .unwrap_or_else(|| anyhow::anyhow!("TypeScript RAM monitor stopped unexpectedly"))
                        .context("No files were changed")),
                }
            }
            None => Err(anyhow::anyhow!(
                "The TypeScript engine stopped before the batch was done"
            )),
        };
        // The engine is stopped before the last write, or when a rename failed.
        if (planned.is_err() || index + 1 == requests.len())
            && let Some(live) = session.take()
        {
            live.shutdown().await;
        }
        let outcome = planned.and_then(|plan| write(&plan, request).map(|()| plan));
        let plan = match outcome {
            Ok(plan) => plan,
            Err(error) => return Err(undo(written, error, index, &requests)),
        };
        reports.push(report(&checked.project, &plan, request.dry_run));
        if !request.dry_run {
            written.push(plan);
        }
    }
    Ok(reports)
}

/// A real rename is written; a dry-run one is remembered for the renames after
/// it. The engine already holds the renamed texts it was shown to prove the plan.
fn write(plan: &Plan, request: &RenameRequest) -> Result<()> {
    if !request.dry_run {
        return apply::apply(plan);
    }
    for file in &plan.files {
        view::remember(&file.path, apply::with_bom(file.bom, &file.new))?;
    }
    Ok(())
}

/// Everything that can be checked before the engine starts, for every request,
/// so that a typo in the fifth does not cost an engine start.
fn check_requests(requests: &[RenameRequest]) -> Result<Checked> {
    let Some(first) = requests.first() else {
        bail!("A batch needs at least one rename");
    };
    for (index, request) in requests.iter().enumerate() {
        let which = |error: anyhow::Error| batch_errors::in_batch(error, index, requests);
        validate_names(&request.symbol, &request.new_name).map_err(which)?;
        if request.column.is_some() && request.line.is_none() {
            return Err(which(anyhow::anyhow!("--column needs --line")));
        }
        if request.dry_run != first.dry_run {
            return Err(which(anyhow::anyhow!(
                "every rename of a batch is a dry run or none is"
            )));
        }
    }
    let project = resolve_project(&first.project_path)?;
    let mut files = Vec::new();
    for (index, request) in requests.iter().enumerate() {
        let which = |error: anyhow::Error| batch_errors::in_batch(error, index, requests);
        if index > 0 {
            let other = resolve_project(&request.project_path).map_err(which)?;
            if other != project {
                return Err(which(anyhow::anyhow!(
                    "a batch works in one project, but this one is in {} and the first is in {}",
                    other.display(),
                    project.display()
                )));
            }
        }
        files.push(resolve_file(&project, &request.file).map_err(which)?);
    }
    Ok(Checked { project, files })
}

fn resolve_project(path: &Path) -> Result<PathBuf> {
    let project = path
        .canonicalize()
        .with_context(|| format!("Project path does not exist: {}", path.display()))?;
    if !project.join("tsconfig.json").is_file() {
        bail!(
            "No tsconfig.json in {}. Symbol rename needs the package root with the authoritative tsconfig.",
            project.display()
        );
    }
    Ok(project)
}

fn resolve_file(project: &Path, file: &Path) -> Result<PathBuf> {
    let absolute = if file.is_absolute() {
        file.to_path_buf()
    } else {
        project.join(file)
    };
    let file = absolute
        .canonicalize()
        .with_context(|| format!("File does not exist: {}", absolute.display()))?;
    if !file.starts_with(project) {
        bail!(
            "{} is outside the project {}",
            file.display(),
            project.display()
        );
    }
    Ok(file)
}

/// Take back the renames that were written before the failing one, newest
/// first, and say what happened. A single rename keeps its error as it is.
fn undo(
    written: Vec<Plan>,
    error: anyhow::Error,
    failed: usize,
    requests: &[RenameRequest],
) -> anyhow::Error {
    let applied = written.len();
    let mut failures = Vec::new();
    for plan in written.iter().rev() {
        if let Err(failure) = apply::revert(plan) {
            failures.push(format!("{failure:#}"));
        }
    }
    batch_errors::step_failed(error, failed, requests, applied, failures)
}

fn report(project: &Path, plan: &Plan, dry_run: bool) -> RenameReport {
    let files = plan
        .files
        .iter()
        .map(|file| {
            (
                file.path
                    .strip_prefix(project)
                    .unwrap_or(&file.path)
                    .to_path_buf(),
                file.edits.len(),
            )
        })
        .collect();
    RenameReport {
        files,
        edits: plan.edit_count(),
        dry_run,
        notes: Vec::new(),
    }
}
