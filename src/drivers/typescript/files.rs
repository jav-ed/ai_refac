//! Finding the files of a project that hold code the TypeScript engine reads.
//!
//! The walk honours the project's ignore files, so build output stays out, and
//! always skips `.git` and `node_modules`. Hidden folders are searched: tools
//! keep code in `.github/` and `.astro/`.

use anyhow::{Context, Result};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

const ALWAYS_SKIPPED: [&str; 2] = [".git", "node_modules"];

/// The files below `root` with one of `extensions` (compared without case).
pub fn find(root: &Path, extensions: &[&str]) -> Result<Vec<PathBuf>> {
    let walker = WalkBuilder::new(root)
        .hidden(false)
        .parents(false)
        .git_global(false)
        .require_git(false)
        .filter_entry(|entry| {
            entry
                .file_name()
                .to_str()
                .is_none_or(|name| !ALWAYS_SKIPPED.contains(&name))
        })
        .build();
    let mut files = Vec::new();
    for entry in walker {
        let entry = entry.with_context(|| format!("Cannot scan {} for code", root.display()))?;
        let wanted = entry
            .path()
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extensions.contains(&extension.to_ascii_lowercase().as_str()));
        if wanted && entry.file_type().is_some_and(|kind| kind.is_file()) {
            files.push(entry.into_path());
        }
    }
    Ok(files)
}
