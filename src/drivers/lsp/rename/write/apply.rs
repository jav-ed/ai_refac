//! Writing a verified rename. The language server takes long to answer, so a
//! file edited meanwhile would be overwritten by the plan made from its old
//! text; every planned file must still read as it did when the plan was made.
//! Everything goes through the journal, and a failure restores the project.

use crate::drivers::lsp::rename::plan::discover::RenamePlan;
use crate::drivers::lsp::rename::write::journal::{FileWrite, Journal};
use anyhow::{Context, Result, bail};

const BOM: char = '\u{FEFF}';

pub fn ensure_unchanged(plan: &RenamePlan) -> Result<()> {
    for edited in &plan.files {
        let path = &edited.file.path;
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("Cannot read {} before writing", path.display()))?;
        if raw.strip_prefix(BOM).unwrap_or(&raw) != edited.file.before {
            bail!(
                "{} changed while the rename was being planned. Nothing was written; run the rename again.",
                path.display()
            );
        }
    }
    Ok(())
}

/// Write the server's edits, then `writes` (files refac edits itself), then
/// move the files the server wants moved.
pub fn apply(plan: &RenamePlan, writes: &[FileWrite]) -> Result<()> {
    apply_undoable(plan, writes).map(drop)
}

/// The same write, keeping the undo log so that a later step of a batch that
/// fails can take this one back too.
pub fn apply_undoable(plan: &RenamePlan, writes: &[FileWrite]) -> Result<Journal> {
    ensure_unchanged(plan)?;
    let mut journal = Journal::default();
    let outcome: Result<()> = (|| {
        for edited in &plan.files {
            journal.write_file(&edited.file.path, &edited.file.bytes)?;
        }
        journal.write_all(writes)?;
        for (from, to) in &plan.moves {
            journal.move_path(from, to)?;
        }
        Ok(())
    })();
    match outcome {
        Ok(()) => Ok(journal),
        Err(error) => match journal.rollback() {
            Ok(()) => Err(error.context("The rename failed and every change was undone")),
            Err(failure) => Err(error.context(format!("{failure:#}"))),
        },
    }
}

#[cfg(test)]
mod tests;
