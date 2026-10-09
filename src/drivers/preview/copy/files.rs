//! Making the copy: which files of the project go into it, and the size limit.

use super::CopyPlan;
use anyhow::{Context, Result, bail};
use ignore::WalkBuilder;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(super) const LIMIT_ENV: &str = "REFAC_DRY_RUN_COPY_MAX_MB";
const DEFAULT_LIMIT_MB: u64 = 500;
const MIB: u64 = 1024 * 1024;
/// Ignored by `.gitignore` in most projects, needed to load them.
const ALWAYS_COPIED: [&str; 3] = ["local.properties", "gradle/wrapper", ".ropeproject"];

pub(super) fn limit_bytes() -> Result<u64> {
    match std::env::var(LIMIT_ENV) {
        Err(_) => Ok(DEFAULT_LIMIT_MB * MIB),
        Ok(value) => value
            .trim()
            .parse::<u64>()
            .ok()
            .filter(|megabytes| *megabytes > 0)
            .map(|megabytes| megabytes * MIB)
            .with_context(|| {
                format!("{LIMIT_ENV} must be a positive number of MiB, got `{value}`")
            }),
    }
}

/// Copies the project and returns the relative paths of the files copied.
/// `plan.only` limits the copy to files with one of these extensions (a backend
/// that reads nothing else does not need a project's data files); empty copies
/// every file the project's `.gitignore` does not exclude. Folders named in
/// `plan.skip` are not entered.
pub(super) fn copy_project(
    root: &Path,
    copy: &Path,
    limit: u64,
    plan: &CopyPlan,
) -> Result<BTreeSet<PathBuf>> {
    let only = plan.only;
    // The walker keeps its filter beyond this function's borrow.
    let skip: Vec<String> = plan.skip.iter().map(|name| name.to_string()).collect();
    let mut size = 0u64;
    let mut copied = BTreeSet::new();
    let walker = WalkBuilder::new(root)
        .hidden(false)
        .require_git(false)
        .filter_entry(move |entry| {
            let name = entry.file_name();
            let is_dir = entry.file_type().is_some_and(|kind| kind.is_dir());
            name != ".git" && !(is_dir && skip.iter().any(|skipped| name == skipped.as_str()))
        })
        .build();
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in walker {
        let entry = entry.context("Cannot read the project folder")?;
        if entry.file_type().is_some_and(|kind| !kind.is_dir()) {
            paths.push(entry.into_path());
        }
    }
    for extra in ALWAYS_COPIED {
        let path = root.join(extra);
        if path.is_file() {
            paths.push(path);
        } else if path.is_dir() {
            paths.extend(files_below(&path)?);
        }
    }
    for path in paths {
        let relative = path.strip_prefix(root)?.to_path_buf();
        if copied.contains(&relative) || !wanted(&path, only) {
            continue;
        }
        let metadata = std::fs::symlink_metadata(&path)?;
        size += metadata.len();
        if size > limit {
            bail!(
                "The project is larger than the {} MiB a dry run may copy ({LIMIT_ENV} raises the limit). The dry run copies the project because this language's backend cannot plan a move without carrying it out.",
                limit / MIB
            );
        }
        let target = copy.join(&relative);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if metadata.file_type().is_symlink() {
            link(&std::fs::read_link(&path)?, &target)?;
        } else {
            std::fs::copy(&path, &target)
                .with_context(|| format!("Cannot copy {}", path.display()))?;
        }
        copied.insert(relative);
    }
    Ok(copied)
}

#[cfg(unix)]
fn link(to: &Path, at: &Path) -> Result<()> {
    Ok(std::os::unix::fs::symlink(to, at)?)
}

#[cfg(not(unix))]
fn link(_: &Path, at: &Path) -> Result<()> {
    bail!("The dry run cannot copy the symbolic link {}", at.display())
}

fn files_below(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry?;
        if !entry.file_type().is_dir() {
            files.push(entry.into_path());
        }
    }
    Ok(files)
}

/// Whether `only` (extensions without the dot; empty means all) names the file.
fn wanted(path: &Path, only: &[&str]) -> bool {
    only.is_empty()
        || path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| only.contains(&extension))
}
