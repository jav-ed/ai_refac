//! Methods that override each other form a family: renaming the base method
//! and leaving the overrides makes the overrides stop overriding, which a
//! dynamic language only notices when the program runs. Some servers rename a
//! family by themselves (gopls, rust-analyzer); others (basedpyright) treat
//! each override as a symbol of its own but can list the overrides of a
//! method. For those, the engine renames every listed override with the
//! same new name and merges the edits into one plan.
//!
//! A server lists the implementations of a class too (its subclasses); those
//! have another name and are not part of a rename, so only the places that
//! spell the symbol being renamed join it.

use super::discover::{Reference, file_uri, parse_references, refuse_or};
use super::edits::{Change, FileEdits, parse_changes};
use super::server::RenameServer;
use crate::drivers::lsp::text::TextIndex;
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::PathBuf;

/// `changes` of the rename at `at`, plus the changes of renaming every
/// override of the symbol there. `known` are the references of the symbol
/// itself; an implementation that is one of them is the symbol, not an
/// override.
pub async fn with_overrides(
    server: &mut dyn RenameServer,
    at: &Value,
    known: &[Reference],
    symbol: &str,
    new_name: &str,
    mut changes: Vec<Change>,
) -> Result<Vec<Change>> {
    let answer = match refuse_or(
        server
            .request("textDocument/implementation", at.clone())
            .await,
    )? {
        Ok(answer) => answer,
        Err(message) => bail!(
            "Cannot rename: the server cannot list the overrides of this symbol ({message}), and a rename without them would change what the program does"
        ),
    };
    if answer.is_null() {
        return Ok(changes);
    }
    let mut texts: HashMap<PathBuf, String> = HashMap::new();
    for implementation in parse_references(&answer)? {
        let is_known = known.iter().any(|reference| {
            reference.path == implementation.path
                && reference.range.start == implementation.range.start
        });
        if is_known || !spells(&mut texts, &implementation, symbol)? {
            continue;
        }
        let request = json!({
            "textDocument": { "uri": file_uri(&implementation.path)? },
            "position": implementation.range.start,
            "newName": new_name,
        });
        let member = match refuse_or(server.request("textDocument/rename", request).await)? {
            Ok(member) => member,
            Err(message) => bail!(
                "Cannot rename: the server refuses to rename the override at {}:{} ({message}). Renaming only part of a family of overrides would change what the program does.",
                implementation.path.display(),
                implementation.range.start.line + 1
            ),
        };
        merge(&mut changes, parse_changes(&member)?);
    }
    Ok(changes)
}

/// Whether the text at the reference is exactly `symbol`.
fn spells(
    texts: &mut HashMap<PathBuf, String>,
    reference: &Reference,
    symbol: &str,
) -> Result<bool> {
    if !texts.contains_key(&reference.path) {
        let raw = std::fs::read_to_string(&reference.path).with_context(|| {
            format!(
                "Cannot read {}, which the server lists as an implementation",
                reference.path.display()
            )
        })?;
        texts.insert(
            reference.path.clone(),
            raw.strip_prefix('\u{FEFF}').unwrap_or(&raw).to_string(),
        );
    }
    let text = &texts[&reference.path];
    let index = TextIndex::new(text);
    let (start, end) = (
        index.offset(reference.range.start)?,
        index.offset(reference.range.end)?,
    );
    Ok(text.get(start..end) == Some(symbol))
}

/// Add the edits of `more` to `changes`. Two renames that both edit a spot
/// make the same edit there, which counts once.
fn merge(changes: &mut Vec<Change>, more: Vec<Change>) {
    for change in more {
        let Change::Edit(file) = change else {
            changes.push(change);
            continue;
        };
        let known = changes.iter_mut().find_map(|existing| match existing {
            Change::Edit(known) if known.path == file.path => Some(known),
            _ => None,
        });
        match known {
            Some(known) => {
                for edit in file.edits {
                    if !known.edits.contains(&edit) {
                        known.edits.push(edit);
                    }
                }
            }
            None => changes.push(Change::Edit(FileEdits {
                path: file.path,
                edits: file.edits,
            })),
        }
    }
}

#[cfg(test)]
mod tests;
