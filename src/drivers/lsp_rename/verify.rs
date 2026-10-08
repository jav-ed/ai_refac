//! Proving a rename is faithful before any file changes. The server does not
//! notice when the new name collides with another symbol in scope: the rename
//! then compiles into a different program (a call that used to reach one
//! declaration reaches another). The check shows the server the renamed text
//! in memory and asks for the references of the renamed declaration again:
//! they must be exactly the places that referred to the symbol before, carried
//! through the edits. A usage that was lost or gained changed meaning.

use super::discover::{Candidate, Reference, RenamePlan, file_uri};
use super::language::Language;
use super::related::{self, Group};
use super::server::RenameServer;
use crate::drivers::lsp::text::TextIndex;
use anyhow::{Context, Result, bail};
use serde_json::json;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use url::Url;

/// (file, zero-based line, UTF-16 column) of where a reference starts.
type Site = (PathBuf, u32, u32);

/// The server's answer cannot be trusted as a faithful rename. A server under
/// load can answer a rename with fewer edits than the references it lists
/// (gopls does), so callers may ask again before giving up.
#[derive(Debug)]
pub struct Unfaithful(pub String);

impl std::fmt::Display for Unfaithful {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Unfaithful {}

pub fn is_unfaithful(error: &anyhow::Error) -> bool {
    error.downcast_ref::<Unfaithful>().is_some()
}

pub async fn verify(
    server: &mut dyn RenameServer,
    language: &dyn Language,
    root: &Path,
    candidate: &Candidate,
    target: &Path,
    symbol: &str,
) -> Result<()> {
    let plan = &candidate.plan;
    let groups = related::groups(server, language, candidate, target, symbol).await?;
    for edited in &plan.files {
        server
            .sync_document(&edited.file.path, &edited.file.text)
            .await?;
    }
    server.settle().await?;
    for group in &groups {
        check_every_reference_is_edited(plan, group, symbol, root)?;
        check_group(server, plan, group).await?;
    }
    Ok(())
}

/// Whether an edit of `from..to` changes the reference at `start..end`. Some
/// servers (the Kotlin one) answer with the smallest edits, a few letters
/// inside the name or an insertion at its end, so an overlap counts.
fn touches(from: usize, to: usize, start: usize, end: usize) -> bool {
    let overlaps = from < end && to > start;
    let inserts_inside = from == to && start <= from && from <= end;
    overlaps || inserts_inside
}

/// A rename must change every place that writes the symbol's name. When the
/// server lists a reference that spells the old name and none of its edits
/// touches it, its answer is incomplete. A reference that spells something
/// else (`Self` for a type, a Java accessor for a Kotlin property) is
/// rightly left alone.
///
/// A reference the server did not edit in a file outside the project (a
/// dependency in the package cache that uses the symbol) is a different
/// failure: no server edits those files, so asking again cannot help, and the
/// rename could never be complete. It is reported as such, not as `Unfaithful`.
fn check_every_reference_is_edited(
    plan: &RenamePlan,
    group: &Group,
    symbol: &str,
    root: &Path,
) -> Result<()> {
    let mut texts: HashMap<&Path, String> = HashMap::new();
    let mut missed = Vec::new();
    for reference in &group.references {
        let edited = plan
            .files
            .iter()
            .find(|edited| edited.file.path == reference.path);
        let before = match edited {
            Some(edited) => edited.file.before.as_str(),
            None => {
                if !texts.contains_key(reference.path.as_path()) {
                    let raw = std::fs::read_to_string(&reference.path).with_context(|| {
                        format!(
                            "Cannot read {}, a reference of the symbol",
                            reference.path.display()
                        )
                    })?;
                    texts.insert(
                        reference.path.as_path(),
                        raw.strip_prefix('\u{FEFF}').unwrap_or(&raw).to_string(),
                    );
                }
                texts[reference.path.as_path()].as_str()
            }
        };
        let index = TextIndex::new(before);
        let (start, end) = (
            index.offset(reference.range.start)?,
            index.offset(reference.range.end)?,
        );
        if before.get(start..end) != Some(symbol) {
            continue;
        }
        let touched = edited.is_some_and(|edited| {
            edited.edits.iter().any(|edit| {
                matches!(
                    (index.offset(edit.range.start), index.offset(edit.range.end)),
                    (Ok(from), Ok(to)) if touches(from, to, start, end)
                )
            })
        });
        if !touched {
            missed.push(reference);
        }
    }
    if missed.is_empty() {
        return Ok(());
    }
    let places = |references: &[&Reference]| -> String {
        references
            .iter()
            .take(6)
            .map(|reference| {
                format!(
                    "{}:{}",
                    reference.path.display(),
                    reference.range.start.line + 1
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    let outside: Vec<&Reference> = missed
        .iter()
        .filter(|reference| !reference.path.starts_with(root))
        .copied()
        .collect();
    if !outside.is_empty() {
        bail!(
            "The symbol is also used in {} places in files outside the project folder {} (first: {}). Those files belong to other packages, such as dependencies in the package cache, and the language server never edits them, so after this rename they would still use the old name and stop building. Nothing was changed. A symbol that other packages use is part of this project's public interface: rename it in those packages together with this one, or keep the name.",
            outside.len(),
            root.display(),
            places(&outside)
        );
    }
    Err(Unfaithful(format!(
        "The server's rename leaves {} of the {} places that refer to the symbol unchanged (first: {}), so the renamed program would not mean the same. Nothing was changed.",
        missed.len(),
        group.references.len(),
        places(&missed)
    ))
    .into())
}

/// After the rename the symbol's usages are exactly the old ones, carried
/// through the edits.
async fn check_group(
    server: &mut dyn RenameServer,
    plan: &RenamePlan,
    group: &Group,
) -> Result<()> {
    let expected = expected_sites(&group.references, plan)?;
    let anchor = new_text_position(plan, &group.path, group.start)?;
    let answer = server
        .request(
            "textDocument/references",
            json!({
                "textDocument": { "uri": file_uri(&group.path)? },
                "position": anchor,
                "context": { "includeDeclaration": true },
            }),
        )
        .await?;
    let actual = sites_of(&answer)?;
    if actual == expected {
        return Ok(());
    }
    Err(Unfaithful(format!(
        "The rename is not faithful: after it, the usages of the symbol at {}:{} differ from before ({} before, {} after), so something would change meaning (a clash with, or shadowing of, another declaration). Nothing was changed.\n{}",
        group.path.display(),
        anchor.line + 1,
        expected.len(),
        actual.len(),
        describe(&expected, &actual)
    ))
    .into())
}

/// Where each reference starts once the edits are applied. A file the
/// rename does not edit keeps its positions.
fn expected_sites(references: &[Reference], plan: &RenamePlan) -> Result<BTreeSet<Site>> {
    let mut sites = BTreeSet::new();
    for reference in references {
        let position = match plan
            .files
            .iter()
            .find(|edited| edited.file.path == reference.path)
        {
            Some(edited) => {
                let old = TextIndex::new(&edited.file.before);
                let start = old.offset(reference.range.start)?;
                let moved = shifted(&old, &edited.edits, start)?;
                TextIndex::new(&edited.file.text).position(moved)
            }
            None => reference.range.start,
        };
        sites.insert((reference.path.clone(), position.line, position.character));
    }
    Ok(sites)
}

/// The offset of a reference start after the edits. Edits that end at or
/// before it move it; an edit that starts exactly there belongs to the
/// identifier itself and does not.
fn shifted(old: &TextIndex, edits: &[lsp_types::TextEdit], start: usize) -> Result<usize> {
    let mut moved = start as isize;
    for edit in edits {
        let (from, to) = (old.offset(edit.range.start)?, old.offset(edit.range.end)?);
        let insertion_at_start = from == to && from == start;
        if to <= start && !insertion_at_start {
            moved += edit.new_text.len() as isize - (to - from) as isize;
        }
    }
    Ok(moved as usize)
}

fn new_text_position(
    plan: &RenamePlan,
    target: &Path,
    anchor_start: usize,
) -> Result<lsp_types::Position> {
    let edited = plan
        .files
        .iter()
        .find(|edited| edited.file.path == target)
        .context("The rename does not edit the file it was asked about")?;
    let old = TextIndex::new(&edited.file.before);
    let moved = shifted(&old, &edited.edits, anchor_start)?;
    Ok(TextIndex::new(&edited.file.text).position(moved))
}

fn sites_of(answer: &serde_json::Value) -> Result<BTreeSet<Site>> {
    let locations = answer
        .as_array()
        .context("The references answer is not a list")?;
    locations
        .iter()
        .map(|location| {
            let path = Url::parse(
                location["uri"]
                    .as_str()
                    .context("A reference without a uri")?,
            )
            .ok()
            .and_then(|url| url.to_file_path().ok())
            .context("A reference outside the local files")?;
            let start = &location["range"]["start"];
            let number = |value: &serde_json::Value| {
                value
                    .as_u64()
                    .context("A reference position is not a number")
                    .map(|n| n as u32)
            };
            Ok((path, number(&start["line"])?, number(&start["character"])?))
        })
        .collect()
}

fn describe(expected: &BTreeSet<Site>, actual: &BTreeSet<Site>) -> String {
    let show = |sites: Vec<&Site>| {
        sites
            .iter()
            .map(|(path, line, column)| {
                format!(
                    "  {}:{}:{}",
                    path.file_name()
                        .map_or_else(String::new, |name| name.to_string_lossy().into_owned()),
                    line + 1,
                    column + 1
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let lost: Vec<&Site> = expected.difference(actual).collect();
    let gained: Vec<&Site> = actual.difference(expected).collect();
    let mut message = String::new();
    if !lost.is_empty() {
        message.push_str(&format!(
            "Usages that no longer refer to the symbol:\n{}\n",
            show(lost)
        ));
    }
    if !gained.is_empty() {
        message.push_str(&format!(
            "Usages that newly refer to it:\n{}\n",
            show(gained)
        ));
    }
    message
}

#[cfg(test)]
mod tests;
