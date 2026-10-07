//! Working out every change a set of moves makes to Markdown files, without
//! touching the disk.
//!
//! Two situations differ in where the files are. When this backend moves the
//! files itself, nothing has moved yet: a link is measured from where its file
//! is now, and goes to where its target will be. When another language's
//! backend has already moved files, the Markdown files are where they will
//! stay; a link is fixed only if its old target is gone and the new one is
//! there, which also keeps a half-moved folder from sending links to files that
//! were never moved.

use super::moves::MoveSet;
use super::rewrite::rewrite;
use super::workspace;
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum When {
    /// This backend moves the files, so nothing has moved yet.
    BeforeMoving,
    /// Files were moved by someone else.
    AfterMoving,
}

/// A Markdown file whose text changes.
pub(crate) struct FileWrite {
    /// Where the file is once the moves are done.
    pub destination: PathBuf,
    pub before: String,
    pub after: String,
}

pub(crate) struct LinkPlan {
    pub when: When,
    pub moves: MoveSet,
    pub writes: Vec<FileWrite>,
    pub links_updated: usize,
    /// How many Markdown files were looked at.
    pub checked: usize,
    /// Markdown files that are not valid UTF-8 and could not be checked.
    pub unreadable: Vec<PathBuf>,
}

pub(crate) async fn plan(moves: MoveSet, root: &Path, when: When) -> Result<LinkPlan> {
    let workspace = workspace::read(files_to_check(&moves, root, when)?).await?;
    let checked = workspace.files.len() + workspace.unreadable.len();

    let mut writes = Vec::new();
    let mut links_updated = 0;
    for file in workspace.files {
        let (original, destination) = match when {
            When::BeforeMoving => (file.path.clone(), moves.destination(&file.path)),
            When::AfterMoving => (origin_on_disk(&moves, &file.path), file.path.clone()),
        };
        let resolve = |path: &Path| match when {
            When::BeforeMoving => moves.destination(path),
            When::AfterMoving => moved_on_disk(&moves, path),
        };
        let rewritten = rewrite(&file.content, &original, &destination, &resolve)?;
        links_updated += rewritten.links_updated;
        if rewritten.content != file.content {
            writes.push(FileWrite {
                destination,
                before: file.content,
                after: rewritten.content,
            });
        }
    }

    Ok(LinkPlan {
        when,
        moves,
        writes,
        links_updated,
        checked,
        unreadable: workspace.unreadable,
    })
}

/// The Markdown files below the root, plus those inside a moved folder or
/// moved by name, which may lie outside the root or in an ignored folder.
fn files_to_check(moves: &MoveSet, root: &Path, when: When) -> Result<Vec<PathBuf>> {
    let mut files: BTreeSet<PathBuf> = workspace::find(root)?.into_iter().collect();
    for entry in moves.moves() {
        let current = match when {
            When::BeforeMoving => &entry.from,
            When::AfterMoving => &entry.to,
        };
        if entry.is_dir {
            if current.is_dir() {
                files.extend(workspace::find(current)?);
            }
        } else if workspace::is_markdown(current) && current.is_file() {
            files.insert(current.clone());
        }
    }
    Ok(files.into_iter().collect())
}

/// Where a Markdown file that is already in place was written for: the old
/// path, unless that path still exists (then nothing moved it).
fn origin_on_disk(moves: &MoveSet, path: &Path) -> PathBuf {
    match moves.origin(path) {
        Some(origin) if !origin.exists() => origin,
        _ => path.to_path_buf(),
    }
}

/// The new place of `path` if the request moved it there: the old one is gone
/// and the new one exists.
fn moved_on_disk(moves: &MoveSet, path: &Path) -> PathBuf {
    match moves.map(path) {
        Some(new) if !path.exists() && new.exists() => new,
        _ => path.to_path_buf(),
    }
}
