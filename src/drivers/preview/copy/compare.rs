//! Reading the copy after the backend's move and comparing it with the
//! project: the difference is the preview.

use super::CopyPlan;
use crate::drivers::MovePreview;
use anyhow::{Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Things a few created files are listed for, then counted.
const CREATED_LISTED: usize = 5;

/// Every file of the copy after the move: relative path to its bytes.
pub(super) fn read_tree(copy: &Path, tool_state: &[&str]) -> Result<BTreeMap<PathBuf, Vec<u8>>> {
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
pub(super) fn compare(
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
pub(super) fn moved_to(relative: &Path, pairs: &[(String, String)]) -> PathBuf {
    for (from, to) in pairs {
        if let Ok(rest) = relative.strip_prefix(from) {
            return Path::new(to).join(rest);
        }
    }
    relative.to_path_buf()
}

/// How many separate changed passages the move made in a file: a changed text
/// file counts its hunks, a changed binary file counts one.
pub(super) fn edit_count(before: &[u8], after: &[u8]) -> usize {
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
