//! The paths a request moves, and where any path ends up.
//!
//! A moved directory carries everything below it, so a link to a file inside it
//! (or to the directory itself) is answered by the same entry. All paths are
//! absolute and normalized; nothing here touches the disk.

use anyhow::{Result, bail};
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Move {
    pub from: PathBuf,
    pub to: PathBuf,
    pub is_dir: bool,
}

#[derive(Debug, Default)]
pub(crate) struct MoveSet {
    moves: Vec<Move>,
}

impl MoveSet {
    /// A set in which no two moves compete for a path: no source inside
    /// another moved directory, no two targets that contain one another, and no
    /// directory moved into itself.
    pub(crate) fn new(moves: Vec<Move>) -> Result<Self> {
        for (index, first) in moves.iter().enumerate() {
            if first.is_dir && first.to.starts_with(&first.from) {
                bail!(
                    "Cannot move the directory {} into itself ({})",
                    first.from.display(),
                    first.to.display()
                );
            }
            for second in &moves[index + 1..] {
                if contains(first, &first.from, &second.from)
                    || contains(second, &second.from, &first.from)
                {
                    bail!(
                        "{} and {} overlap: one is inside the other and both are moved",
                        first.from.display(),
                        second.from.display()
                    );
                }
                if contains(first, &first.to, &second.to) || contains(second, &second.to, &first.to)
                {
                    bail!(
                        "The targets {} and {} overlap: one is inside the other",
                        first.to.display(),
                        second.to.display()
                    );
                }
            }
        }
        Ok(Self { moves })
    }

    pub(crate) fn moves(&self) -> &[Move] {
        &self.moves
    }

    /// Where `path` ends up when it is moved, directly or as part of a moved
    /// directory. `None` when the request leaves it where it is.
    pub(crate) fn map(&self, path: &Path) -> Option<PathBuf> {
        self.moves.iter().find_map(|entry| {
            if path == entry.from {
                return Some(entry.to.clone());
            }
            if !entry.is_dir {
                return None;
            }
            let rest = path.strip_prefix(&entry.from).ok()?;
            Some(entry.to.join(rest))
        })
    }

    /// `map`, or the path itself.
    pub(crate) fn destination(&self, path: &Path) -> PathBuf {
        self.map(path).unwrap_or_else(|| path.to_path_buf())
    }

    /// Where `path` was before the moves: the reverse of `map`. `None` when the
    /// request did not move anything to it.
    pub(crate) fn origin(&self, path: &Path) -> Option<PathBuf> {
        self.moves.iter().find_map(|entry| {
            if path == entry.to {
                return Some(entry.from.clone());
            }
            if !entry.is_dir {
                return None;
            }
            let rest = path.strip_prefix(&entry.to).ok()?;
            Some(entry.from.join(rest))
        })
    }
}

/// Whether `inner` is `outer` itself or, when `entry` is a directory, below it.
fn contains(entry: &Move, outer: &Path, inner: &Path) -> bool {
    inner == outer || (entry.is_dir && inner.starts_with(outer))
}

/// Resolve `.` and `..` without the file system (symlinks stay as written).
pub(crate) fn normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(segment) => normalized.push(segment),
        }
    }
    normalized
}

/// The deepest directory that contains every one of the paths.
pub(crate) fn common_ancestor<'a>(paths: impl IntoIterator<Item = &'a Path>) -> PathBuf {
    let mut paths = paths.into_iter();
    let Some(first) = paths.next() else {
        return PathBuf::new();
    };
    let mut shared: Vec<Component> = first.components().collect();
    for path in paths {
        let keep = shared
            .iter()
            .zip(path.components())
            .take_while(|(left, right)| *left == right)
            .count();
        shared.truncate(keep);
    }
    shared
        .iter()
        .map(|component| component.as_os_str())
        .collect()
}

/// An absolute, normalized path: relative paths are taken from `root`, which
/// itself may be relative to the working directory.
pub(crate) fn absolute(path: &Path, root: &Path) -> PathBuf {
    if path.is_absolute() {
        normalize(path)
    } else {
        normalize(&root.join(path))
    }
}

#[cfg(test)]
mod tests;
