//! Finding and reading the Markdown files a move can affect.
//!
//! The walk starts at the project root and reads ignore files (`.gitignore`,
//! `.ignore`) found inside it, so build output and vendored folders are left
//! alone; hidden folders are not skipped, because documentation lives in
//! `.github/` and `.agents/`. `.git` and `node_modules` are always skipped.

use anyhow::{Context, Result};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

const MARKDOWN_EXTENSIONS: [&str; 3] = ["md", "markdown", "mdx"];
const ALWAYS_SKIPPED: [&str; 2] = [".git", "node_modules"];

pub(crate) fn is_markdown(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            MARKDOWN_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
        })
}

pub(super) struct MarkdownFile {
    pub path: PathBuf,
    pub content: String,
}

pub(super) struct Workspace {
    pub files: Vec<MarkdownFile>,
    /// Markdown files that are not valid UTF-8 and so were not read.
    pub unreadable: Vec<PathBuf>,
}

/// Every Markdown file below `root`, in path order.
pub(super) fn find(root: &Path) -> Result<Vec<PathBuf>> {
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
        let entry =
            entry.with_context(|| format!("Cannot scan {} for Markdown files", root.display()))?;
        if entry.file_type().is_some_and(|kind| kind.is_file()) && is_markdown(entry.path()) {
            files.push(entry.into_path());
        }
    }
    files.sort();
    Ok(files)
}

pub(super) async fn read(paths: Vec<PathBuf>) -> Result<Workspace> {
    let mut workspace = Workspace {
        files: Vec::with_capacity(paths.len()),
        unreadable: Vec::new(),
    };
    for path in paths {
        match tokio::fs::read(&path).await {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(content) => workspace.files.push(MarkdownFile { path, content }),
                Err(_) => workspace.unreadable.push(path),
            },
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("Failed to read Markdown file {}", path.display()));
            }
        }
    }
    Ok(workspace)
}

#[cfg(test)]
mod tests;
