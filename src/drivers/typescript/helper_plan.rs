//! The plan the TypeScript helper prints, and the preview refac makes of it.

use crate::drivers::MovePreview;
use anyhow::{Context, Result};
use std::path::PathBuf;

/// The plan the helper prints as its last line, with `--dry-run` and after a
/// real move: `{"moves": [{from, to}], "files": [{path, edits}], "unchecked":
/// n, "notes": [..]}`. `unchecked` counts the rewritten imports the helper
/// could not resolve without the moved files; `notes` are what the move
/// changes that no edit can repair.
#[derive(serde::Deserialize)]
pub(super) struct HelperPlan {
    pub moves: Vec<HelperMove>,
    pub files: Vec<HelperFile>,
    pub unchecked: usize,
    pub notes: Vec<String>,
}

#[derive(serde::Deserialize)]
pub(super) struct HelperMove {
    pub from: PathBuf,
    pub to: PathBuf,
}

#[derive(serde::Deserialize)]
pub(super) struct HelperFile {
    pub path: PathBuf,
    pub edits: usize,
}

pub(super) fn parse_helper_plan(stdout: &str) -> Result<HelperPlan> {
    let line = stdout
        .lines()
        .rev()
        .find(|line| line.trim_start().starts_with('{'))
        .ok_or_else(|| anyhow::anyhow!("The TypeScript helper printed no plan: {stdout:?}"))?;
    serde_json::from_str(line).context("The TypeScript helper printed a plan refac cannot read")
}

pub(super) fn parse_plan(stdout: &str) -> Result<MovePreview> {
    let plan = parse_helper_plan(stdout)?;
    let mut preview = MovePreview {
        moves: plan
            .moves
            .into_iter()
            .map(|entry| (entry.from, entry.to))
            .collect(),
        ..Default::default()
    };
    for file in plan.files {
        preview.add_edits(file.path, file.edits);
    }
    preview.notes.extend(plan.notes);
    if plan.unchecked > 0 {
        preview.notes.push(format!(
            "{} rewritten import(s) of asset files through an alias could not be checked against the moved files in a dry run; the move itself checks them.",
            plan.unchecked
        ));
    }
    Ok(preview)
}
