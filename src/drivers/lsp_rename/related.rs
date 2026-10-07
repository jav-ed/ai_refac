//! The symbols a rename really changes. Usually that is the one symbol the
//! user named, and every edit lies in one of its references. Some renames
//! reach further on purpose: renaming an interface method must also rename
//! the methods that implement it, or the program no longer compiles. The
//! server then edits places that are no reference of the named symbol.
//!
//! Each such edit is accepted only if it is itself a reference of a symbol the
//! server can name at that spot, and that symbol joins the rename as one more
//! group. The verification then proves every group, not only the first: each
//! group's usages after the rename must be exactly its usages before. An edit
//! that belongs to no symbol (a stray change to code that has nothing to do
//! with the rename) still stops it.

use super::discover::{Candidate, Reference, RenamePlan, references_at};
use super::language::{EditedText, Language};
use super::server::RenameServer;
use crate::drivers::lsp_session::RpcError;
use crate::drivers::lsp_text::TextIndex;
use anyhow::{Result, bail};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// More related symbols than this is not a rename of one thing.
const MAX_GROUPS: usize = 50;

/// One symbol the rename changes, found from one position.
pub struct Group {
    pub path: PathBuf,
    /// Byte offset in the old text of `path` where the symbol was asked about.
    pub start: usize,
    pub references: Vec<Reference>,
}

/// The byte ranges, per edited file, of the references of the groups so far.
struct Coverage<'a> {
    indexes: HashMap<&'a Path, TextIndex<'a>>,
    ranges: HashMap<&'a Path, Vec<(usize, usize)>>,
}

impl<'a> Coverage<'a> {
    fn new(plan: &'a RenamePlan) -> Self {
        Self {
            indexes: plan
                .files
                .iter()
                .map(|edited| {
                    (
                        edited.file.path.as_path(),
                        TextIndex::new(&edited.file.before),
                    )
                })
                .collect(),
            ranges: HashMap::new(),
        }
    }

    fn add(&mut self, references: &[Reference]) -> Result<()> {
        for reference in references {
            let Some((path, index)) = self.indexes.get_key_value(reference.path.as_path()) else {
                continue;
            };
            let range = (
                index.offset(reference.range.start)?,
                index.offset(reference.range.end)?,
            );
            self.ranges.entry(path).or_default().push(range);
        }
        Ok(())
    }

    fn covers(&self, path: &Path, start: usize, end: usize) -> bool {
        self.ranges
            .get(path)
            .is_some_and(|ranges| ranges.iter().any(|(from, to)| *from <= start && end <= *to))
    }
}

/// The primary group, and one more for each symbol an edit outside every
/// known reference belongs to. Edits the language expects outside references
/// are skipped.
pub async fn groups(
    server: &mut dyn RenameServer,
    language: &dyn Language,
    candidate: &Candidate,
    target: &Path,
    symbol: &str,
) -> Result<Vec<Group>> {
    let plan = &candidate.plan;
    let mut groups = vec![Group {
        path: target.to_path_buf(),
        start: candidate.anchor_start,
        references: candidate.references.clone(),
    }];
    let mut coverage = Coverage::new(plan);
    coverage.add(&candidate.references)?;

    for edited in &plan.files {
        let before = &edited.file.before;
        let index = TextIndex::new(before);
        let spans: Vec<(usize, usize)> = edited
            .edits
            .iter()
            .map(|edit| {
                Ok((
                    index.offset(edit.range.start)?,
                    index.offset(edit.range.end)?,
                ))
            })
            .collect::<Result<_>>()?;
        let exempt = language.exempt_edits(&EditedText {
            before,
            spans: spans.clone(),
            new_texts: edited
                .edits
                .iter()
                .map(|edit| edit.new_text.as_str())
                .collect(),
            symbol,
        });
        for ((edit, (start, end)), expected) in edited.edits.iter().zip(spans).zip(exempt) {
            let path = edited.file.path.as_path();
            if expected || coverage.covers(path, start, end) {
                continue;
            }
            let references = match references_at(server, path, edit.range.start).await {
                Ok(references) => references,
                // A spot the server will not name a symbol at belongs to none.
                Err(error) if error.downcast_ref::<RpcError>().is_some() => Vec::new(),
                Err(error) => return Err(error),
            };
            let belongs = references.iter().any(|reference| {
                let range = index
                    .offset(reference.range.start)
                    .ok()
                    .zip(index.offset(reference.range.end).ok());
                reference.path == path && range.is_some_and(|(from, to)| from <= start && end <= to)
            });
            if !belongs || groups.len() >= MAX_GROUPS {
                bail!(
                    "The server edits {} line {} outside every place that refers to the symbol, and that spot belongs to no other symbol the rename could change; refusing to rename without understanding that edit. Nothing was changed.",
                    path.display(),
                    edit.range.start.line + 1
                );
            }
            coverage.add(&references)?;
            groups.push(Group {
                path: path.to_path_buf(),
                start,
                references,
            });
        }
    }
    Ok(groups)
}

#[cfg(test)]
mod tests;
