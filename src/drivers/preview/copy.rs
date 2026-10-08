//! A dry run for the backends that cannot plan a move without carrying it out:
//! Rope applies one move after the other and reads the moved files, and the
//! Kotlin server, Snapshot and Android layer need them in their new places.
//! The real move runs on a throw-away copy of the project, and what differs
//! between the copy and the original is the preview. The original is never
//! touched.
//!
//! The copy holds what the project's `.gitignore` does not exclude (and never
//! `.git`), plus the few ignored files a tool needs to load the project. It is
//! limited in size (`REFAC_DRY_RUN_COPY_MAX_MB`, default 500) so that a dry run
//! cannot fill the disk; a project over the limit is an error that says so.

use super::MovePreview;
use anyhow::{Context, Result, bail};
use ignore::WalkBuilder;
use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::path::{Component, Path, PathBuf};

const LIMIT_ENV: &str = "REFAC_DRY_RUN_COPY_MAX_MB";
const DEFAULT_LIMIT_MB: u64 = 500;
const MIB: u64 = 1024 * 1024;
/// Ignored by `.gitignore` in most projects, needed to load them.
const ALWAYS_COPIED: [&str; 3] = ["local.properties", "gradle/wrapper", ".ropeproject"];
/// Things a few created files are listed for, then counted.
const CREATED_LISTED: usize = 5;

/// What a backend says about its copy: the folders a tool fills with its own
/// files, which are not part of the move.
pub struct CopyPlan<'a> {
    /// Names of folders (at any depth) whose content is the tool's own state
    /// and is left out of the comparison altogether, such as `.ropeproject`.
    pub tool_state: &'a [&'a str],
    /// Names of folders the tool writes build output into: a file created
    /// there is not reported, a file that was already there is compared like
    /// any other (a package may be called `build`).
    pub scratch: &'a [&'a str],
}

/// Runs `run` (the backend's real move) on a copy of the project and returns
/// the difference. `run` gets the pairs relative to the copy and the copy's
/// root, and returns the notes the real move would print.
pub async fn preview_on_copy<F, Fut>(
    root_path: Option<&Path>,
    file_map: &[(String, String)],
    plan: CopyPlan<'_>,
    run: F,
) -> Result<MovePreview>
where
    F: FnOnce(Vec<(String, String)>, PathBuf) -> Fut,
    Fut: Future<Output = Result<Vec<String>>>,
{
    let root = match root_path {
        Some(root) => std::path::absolute(root)?,
        None => std::env::current_dir()?,
    };
    let root = root
        .canonicalize()
        .with_context(|| format!("Cannot read the project folder {}", root.display()))?;
    let pairs = relative_pairs(&root, file_map)?;

    let copy = tempfile::Builder::new()
        .prefix("refac-dry-run-")
        .tempdir()
        .context("Cannot create the folder for the dry-run copy")?;
    let copied = copy_project(&root, copy.path(), limit_bytes()?)?;

    let notes = run(pairs.clone(), copy.path().to_path_buf()).await?;

    let after = read_tree(copy.path(), plan.tool_state)?;
    let mut preview = compare(&root, &copied, &after, &pairs, &plan)?;
    preview.notes.splice(0..0, notes);
    Ok(preview)
}

/// The pairs relative to the project: the copy has the same layout.
fn relative_pairs(root: &Path, file_map: &[(String, String)]) -> Result<Vec<(String, String)>> {
    let inside = |path: &str| -> Result<String> {
        let joined = if Path::new(path).is_absolute() {
            PathBuf::from(path)
        } else {
            root.join(path)
        };
        let normalized = normalize(&joined);
        let Ok(relative) = normalized.strip_prefix(root) else {
            bail!(
                "A dry run copies the project, so every path must lie inside {}: {path} does not. Run the move itself to move files out of the project.",
                root.display()
            );
        };
        Ok(relative.to_string_lossy().into_owned())
    };
    file_map
        .iter()
        .map(|(source, target)| Ok((inside(source)?, inside(target)?)))
        .collect()
}

fn normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn limit_bytes() -> Result<u64> {
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
fn copy_project(root: &Path, copy: &Path, limit: u64) -> Result<BTreeSet<PathBuf>> {
    let mut size = 0u64;
    let mut copied = BTreeSet::new();
    let walker = WalkBuilder::new(root)
        .hidden(false)
        .require_git(false)
        .filter_entry(|entry| entry.file_name() != ".git")
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
        if copied.contains(&relative) {
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

/// Every file of the copy after the move: relative path to its bytes.
fn read_tree(copy: &Path, tool_state: &[&str]) -> Result<BTreeMap<PathBuf, Vec<u8>>> {
    let mut files = BTreeMap::new();
    for entry in walkdir::WalkDir::new(copy).follow_links(false) {
        let entry = entry?;
        if entry.file_type().is_dir() {
            continue;
        }
        let relative = entry.path().strip_prefix(copy)?.to_path_buf();
        if has_component(&relative, tool_state) {
            continue;
        }
        let bytes = if entry.file_type().is_symlink() {
            std::fs::read_link(entry.path())?
                .to_string_lossy()
                .into_owned()
                .into_bytes()
        } else {
            std::fs::read(entry.path())?
        };
        files.insert(relative, bytes);
    }
    Ok(files)
}

/// The preview: the requested moves, what moved with them, and the edits.
fn compare(
    root: &Path,
    before: &BTreeSet<PathBuf>,
    after: &BTreeMap<PathBuf, Vec<u8>>,
    pairs: &[(String, String)],
    plan: &CopyPlan<'_>,
) -> Result<MovePreview> {
    let mut preview = MovePreview {
        moves: pairs
            .iter()
            .map(|(from, to)| (root.join(from), root.join(to)))
            .collect(),
        ..Default::default()
    };
    let mut claimed: BTreeSet<&PathBuf> = BTreeSet::new();
    for relative in before {
        if has_component(relative, plan.tool_state) {
            continue;
        }
        let original = std::fs::read(root.join(relative))
            .with_context(|| format!("Cannot read {}", root.join(relative).display()))?;
        // Where the file is after the move: where a requested move put it, or
        // where it was.
        let expected = moved_to(relative, pairs);
        if let Some((key, bytes)) = after.get_key_value(&expected) {
            claimed.insert(key);
            preview.add_edits(root.join(relative), edit_count(&original, bytes));
            continue;
        }
        // Gone from the copy: moved by the tool beyond the request (a package
        // moves with its files), or deleted.
        let travelled = after.iter().find(|(path, bytes)| {
            !claimed.contains(path) && !before.contains(*path) && **bytes == original
        });
        match travelled {
            Some((path, _)) => {
                claimed.insert(path);
                preview.moves.push((root.join(relative), root.join(path)));
            }
            None => preview.notes.push(format!(
                "The move would delete {}.",
                root.join(relative).display()
            )),
        }
    }
    let created: Vec<&PathBuf> = after
        .keys()
        .filter(|path| {
            !before.contains(*path) && !claimed.contains(path) && !has_component(path, plan.scratch)
        })
        .collect();
    for path in created.iter().take(CREATED_LISTED) {
        preview.notes.push(format!(
            "The move would create {}.",
            root.join(path).display()
        ));
    }
    if created.len() > CREATED_LISTED {
        preview.notes.push(format!(
            "The move would create {} more files that are not listed.",
            created.len() - CREATED_LISTED
        ));
    }
    Ok(preview)
}

fn has_component(path: &Path, names: &[&str]) -> bool {
    path.components().any(|component| {
        names
            .iter()
            .any(|name| component.as_os_str() == std::ffi::OsStr::new(name))
    })
}

/// Where `relative` ends up when the pairs move it, directly or inside a moved
/// folder.
fn moved_to(relative: &Path, pairs: &[(String, String)]) -> PathBuf {
    for (from, to) in pairs {
        if let Ok(rest) = relative.strip_prefix(from) {
            return Path::new(to).join(rest);
        }
    }
    relative.to_path_buf()
}

/// How many separate changed passages the move made in a file: a changed text
/// file counts its hunks, a changed binary file counts one.
fn edit_count(before: &[u8], after: &[u8]) -> usize {
    if before == after {
        return 0;
    }
    match (std::str::from_utf8(before), std::str::from_utf8(after)) {
        (Ok(old), Ok(new)) => similar::TextDiff::from_lines(old, new)
            .grouped_ops(0)
            .len()
            .max(1),
        _ => 1,
    }
}

#[cfg(test)]
mod tests;
