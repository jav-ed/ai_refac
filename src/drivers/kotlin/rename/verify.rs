//! Proving a rename is faithful before any file changes. The server does not
//! notice when the new name collides with another symbol in scope: the rename
//! then compiles into a different program (a call that used to reach one
//! declaration reaches another). The check shows the server the renamed text
//! in memory and asks for the references of the renamed declaration again:
//! they must be exactly the places that referred to the symbol before, carried
//! through the edits. A usage that was lost or gained changed meaning.

use super::plan::{Candidate, Reference, RenamePlan, file_uri};
use crate::drivers::kotlin::server::KotlinServer;
use crate::drivers::lsp_text::TextIndex;
use anyhow::{Context, Result, bail};
use serde_json::json;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use url::Url;

/// (file, zero-based line, UTF-16 column) of where a reference starts.
type Site = (PathBuf, u32, u32);

pub async fn verify(
    server: &mut KotlinServer,
    candidate: &Candidate,
    plan: &RenamePlan,
    target: &Path,
    symbol: &str,
) -> Result<()> {
    check_edits_stay_inside_references(candidate, plan, symbol)?;
    for edited in &plan.files {
        server
            .sync_document(&edited.file.path, &edited.file.text)
            .await?;
    }
    let expected = expected_sites(&candidate.references, plan)?;
    let anchor = new_text_position(plan, target, candidate.anchor_start)?;
    let answer = server
        .request(
            "textDocument/references",
            json!({
                "textDocument": { "uri": file_uri(target)? },
                "position": anchor,
                "context": { "includeDeclaration": true },
            }),
        )
        .await?;
    let actual = sites_of(&answer)?;
    if actual == expected {
        return Ok(());
    }
    bail!(
        "The rename is not faithful: after it, the symbol's usages differ from before, so something would change meaning (a clash with, or shadowing of, another declaration). Nothing was changed.\n{}",
        describe(&expected, &actual)
    )
}

/// An edit outside every reference would change code that has nothing to do
/// with the symbol. The exception is the import of the symbol: the server does
/// not list it as a reference (extension functions), yet rewrites or drops it,
/// and tidies the blank lines around a dropped import.
fn check_edits_stay_inside_references(
    candidate: &Candidate,
    plan: &RenamePlan,
    symbol: &str,
) -> Result<()> {
    for edited in &plan.files {
        let before = &edited.file.before;
        let index = TextIndex::new(before);
        let ranges: Vec<(usize, usize)> = candidate
            .references
            .iter()
            .filter(|reference| reference.path == edited.file.path)
            .map(|reference| {
                Ok((
                    index.offset(reference.range.start)?,
                    index.offset(reference.range.end)?,
                ))
            })
            .collect::<Result<_>>()?;
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
        let import_edited = spans
            .iter()
            .any(|(start, end)| edits_import_of_symbol(before, *start, *end, symbol));
        for (edit, (start, end)) in edited.edits.iter().zip(spans) {
            let inside_reference = ranges.iter().any(|(from, to)| *from <= start && end <= *to);
            let import_cleanup = edits_import_of_symbol(before, start, end, symbol)
                || (import_edited && removes_blank_lines(before, start, end, &edit.new_text));
            if !inside_reference && !import_cleanup {
                bail!(
                    "The server edits {} line {} outside every place that refers to the symbol; refusing to rename without understanding that edit. Nothing was changed.",
                    edited.file.path.display(),
                    edit.range.start.line + 1
                );
            }
        }
    }
    Ok(())
}

/// A deletion of whole blank lines, nothing else.
fn removes_blank_lines(before: &str, start: usize, end: usize, new_text: &str) -> bool {
    let removed = &before[start..end];
    new_text.is_empty()
        && !removed.is_empty()
        && removed.ends_with('\n')
        && removed.chars().all(char::is_whitespace)
        && (start == 0 || before[..start].ends_with('\n'))
}

/// An edit on the `import ...<symbol>` line of the symbol being renamed. The
/// server lists no reference for that import, yet it edits it: it rewrites the
/// name, or drops the whole line when the new name no longer needs it (a member
/// of the same name now wins over the imported extension). Dropping the line
/// shows up in the usage comparison as lost usages when it changes meaning.
fn edits_import_of_symbol(before: &str, start: usize, end: usize, symbol: &str) -> bool {
    let line_start = before[..start].rfind('\n').map_or(0, |at| at + 1);
    let line_end = before[start..]
        .find('\n')
        .map_or(before.len(), |at| start + at);
    // The line break itself may go with the line.
    let reach = (line_end + 1).min(before.len());
    if end > reach {
        return false;
    }
    let line = before[line_start..line_end].trim_end();
    if !line.trim_start().starts_with("import ") {
        return false;
    }
    // `import a.b.symbol` or `import a.b.symbol as alias`: the symbol is the
    // last path segment.
    let path = line.split(" as ").next().unwrap_or(line).trim_end();
    path.strip_suffix(symbol)
        .is_some_and(|prefix| prefix.ends_with('.'))
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
