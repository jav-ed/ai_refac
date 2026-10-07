//! Moving Markdown files, asset files (images, PDFs, ...), and folders of
//! them, and fixing every Markdown link that follows them.

use super::apply::apply;
use super::moves::{self, Move, MoveSet, absolute, normalize};
use super::plan::{LinkPlan, When, plan};
use super::workspace::is_markdown;
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

/// Move what the request names and update the links. The returned notes say
/// what happened besides the move.
pub(super) async fn move_documents(
    file_map: Vec<(String, String)>,
    root_path: Option<&Path>,
) -> Result<Vec<String>> {
    let base = base_directory(root_path)?;
    let moves = requested(&file_map, &base)?;
    let root = scan_root(root_path, &base, &moves);

    let plan = plan(moves, &root, When::BeforeMoving).await?;
    apply(&plan).await?;
    Ok(notes(&plan, &root))
}

/// Relative paths are taken from the project path, or from where refac runs.
pub(super) fn base_directory(root_path: Option<&Path>) -> Result<PathBuf> {
    match root_path {
        Some(root) => Ok(normalize(
            &std::path::absolute(root).context("Cannot resolve the project path")?,
        )),
        None => std::env::current_dir().context("Failed to read the current directory"),
    }
}

/// The folder whose Markdown files are checked: the project path, or without
/// one the folder all moves share.
pub(super) fn scan_root(root_path: Option<&Path>, base: &Path, moves: &MoveSet) -> PathBuf {
    match root_path {
        Some(_) => base.to_path_buf(),
        None => common_directory(moves),
    }
}

fn requested(file_map: &[(String, String)], base: &Path) -> Result<MoveSet> {
    let mut moves = Vec::with_capacity(file_map.len());
    for (source, target) in file_map {
        let from = absolute(Path::new(source), base);
        let to = absolute(Path::new(target), base);
        if !from.exists() {
            bail!("Source path does not exist: {}", from.display());
        }
        if from != to && to.exists() {
            bail!("Target already exists: {}", to.display());
        }
        let is_dir = from.is_dir();
        if !is_dir && is_markdown(&from) != is_markdown(&to) {
            bail!(
                "A Markdown file must stay a Markdown file (.md, .markdown, .mdx) and another file must not become one: {} -> {}",
                from.display(),
                to.display()
            );
        }
        moves.push(Move { from, to, is_dir });
    }
    MoveSet::new(moves)
}

/// Without a project path the links are checked below the folder all moves
/// share.
fn common_directory(moves: &MoveSet) -> PathBuf {
    moves::common_ancestor(
        moves
            .moves()
            .iter()
            .flat_map(|entry| [&entry.from, &entry.to])
            .filter_map(|path| path.parent()),
    )
}

pub(super) fn notes(plan: &LinkPlan, root: &Path) -> Vec<String> {
    let mut notes = vec![summary(plan)];
    if !plan.unreadable.is_empty() {
        let names: Vec<String> = plan
            .unreadable
            .iter()
            .map(|path| {
                path.strip_prefix(root)
                    .unwrap_or(path)
                    .display()
                    .to_string()
            })
            .collect();
        notes.push(format!(
            "These Markdown files are not valid UTF-8, so their links were not checked: {}.",
            names.join(", ")
        ));
    }
    notes
}

fn plural(count: usize, word: &str) -> String {
    format!("{count} {word}{}", if count == 1 { "" } else { "s" })
}

pub(super) fn summary(plan: &LinkPlan) -> String {
    format!(
        "Checked {}; updated {} in {}.",
        plural(plan.checked, "Markdown file"),
        plural(plan.links_updated, "link"),
        plural(plan.writes.len(), "file")
    )
}
