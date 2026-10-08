use anyhow::{Context, Result, bail};
use ra_ap_ide::TextRange;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextReplacement {
    pub path: PathBuf,
    pub start: usize,
    pub end: usize,
    pub replacement: String,
}

impl TextReplacement {
    pub fn from_range(path: PathBuf, range: TextRange, replacement: String) -> Self {
        Self {
            path,
            start: usize::from(range.start()),
            end: usize::from(range.end()),
            replacement,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PlannedMove {
    pub source: PathBuf,
    pub target: PathBuf,
}

pub struct MovePlan {
    pub writes: BTreeMap<PathBuf, String>,
    pub moves: Vec<PlannedMove>,
    pub created_directories: Vec<PathBuf>,
}

pub fn render_writes(
    replacements: Vec<TextReplacement>,
    new_files: BTreeMap<PathBuf, String>,
) -> Result<BTreeMap<PathBuf, String>> {
    let mut by_file = BTreeMap::<PathBuf, Vec<TextReplacement>>::new();
    for replacement in replacements {
        by_file
            .entry(replacement.path.clone())
            .or_default()
            .push(replacement);
    }

    let mut writes = new_files;
    for (path, mut edits) in by_file {
        let mut content = writes
            .remove(&path)
            .map(Ok)
            .unwrap_or_else(|| std::fs::read_to_string(&path))
            .with_context(|| {
                format!("Could not read planned Rust edit target {}", path.display())
            })?;
        edits.sort_by(|left, right| right.start.cmp(&left.start).then(right.end.cmp(&left.end)));
        let mut previous_start = content.len() + 1;
        let mut seen = BTreeSet::new();
        for edit in edits {
            let identity = (edit.start, edit.end, edit.replacement.clone());
            if !seen.insert(identity) {
                continue;
            }
            if edit.end > content.len() || edit.start > edit.end {
                bail!("Planned edit is outside {}", path.display());
            }
            if edit.end > previous_start {
                bail!(
                    "Overlapping semantic edits were planned for {}; no files were changed",
                    path.display()
                );
            }
            content.replace_range(edit.start..edit.end, &edit.replacement);
            previous_start = edit.start;
        }
        writes.insert(path, content);
    }
    Ok(writes)
}

pub fn apply_transaction<F>(plan: MovePlan, validate: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    let snapshots = plan
        .writes
        .keys()
        .map(|path| {
            let before = if path.exists() {
                Some(std::fs::read(path)?)
            } else {
                None
            };
            Ok((path.clone(), before))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;

    let result = apply_changes(&plan).and_then(|()| validate());
    if result.is_ok() {
        return result;
    }

    let rollback_result = rollback(&plan, &snapshots);
    match rollback_result {
        Ok(()) => Err(result.unwrap_err())
            .context("Rust module move failed; every changed path was restored"),
        Err(rollback_error) => bail!(
            "Rust module move failed and rollback also failed: {rollback_error:#}. Original failure: {:#}",
            result.unwrap_err()
        ),
    }
}

fn apply_changes(plan: &MovePlan) -> Result<()> {
    for directory in &plan.created_directories {
        std::fs::create_dir_all(directory)?;
    }
    for (path, content) in &plan.writes {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
    }
    for movement in &plan.moves {
        if let Some(parent) = movement.target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::rename(&movement.source, &movement.target).with_context(|| {
            format!(
                "Could not move {} to {}",
                movement.source.display(),
                movement.target.display()
            )
        })?;
    }
    Ok(())
}

fn rollback(plan: &MovePlan, snapshots: &BTreeMap<PathBuf, Option<Vec<u8>>>) -> Result<()> {
    for movement in plan.moves.iter().rev() {
        if movement.target.exists() {
            std::fs::rename(&movement.target, &movement.source)?;
        }
    }
    for (path, before) in snapshots {
        match before {
            Some(content) => std::fs::write(path, content)?,
            None if path.exists() => std::fs::remove_file(path)?,
            None => {}
        }
    }
    let mut directories = plan.created_directories.clone();
    directories.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    for directory in directories {
        if directory.exists() {
            let _ = std::fs::remove_dir(&directory);
        }
    }
    Ok(())
}

pub fn append_replacement(path: &Path, content: &str, line: &str) -> TextReplacement {
    let separator = if content.is_empty() || content.ends_with('\n') {
        ""
    } else {
        "\n"
    };
    TextReplacement {
        path: path.to_path_buf(),
        start: content.len(),
        end: content.len(),
        replacement: format!("{separator}{line}\n"),
    }
}
