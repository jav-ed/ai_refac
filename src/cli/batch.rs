//! A list given in a file (or on stdin) instead of on the command line:
//! `rename --batch` and `move --batch`. The reading is shared so both fail the
//! same way and name the file they could not read.

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::io::{self, Read};
use std::path::Path;

/// One move of a `move --batch` file. The keys are the names of the flags.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MoveEntry {
    source_path: String,
    target_path: String,
}

/// The text of a batch file, or of stdin for `-`.
pub(super) fn read_text(source: &Path) -> Result<String> {
    if source == Path::new("-") {
        let mut text = String::new();
        io::stdin()
            .read_to_string(&mut text)
            .context("Cannot read the batch from stdin")?;
        return Ok(text);
    }
    std::fs::read_to_string(source)
        .with_context(|| format!("Cannot read the batch file {}", source.display()))
}

/// The sources and the targets of a `move --batch` file, in the order the
/// file lists them (the n-th source goes to the n-th target).
pub(super) fn read_moves(source: &Path) -> Result<(Vec<String>, Vec<String>)> {
    parse_moves(&read_text(source)?, source)
}

fn parse_moves(text: &str, source: &Path) -> Result<(Vec<String>, Vec<String>)> {
    let entries: Vec<MoveEntry> = serde_json::from_str(text).with_context(|| {
        format!(
            "{} is not a list of moves: expected a JSON list like [{{\"source_path\": \"src/a.kt\", \"target_path\": \"src/pkg/a.kt\"}}, ...]; every entry needs exactly those two keys",
            source.display()
        )
    })?;
    if entries.is_empty() {
        bail!(
            "{} lists no moves: the JSON list is empty, so there is nothing to move",
            source.display()
        );
    }
    Ok(entries
        .into_iter()
        .map(|entry| (entry.source_path, entry.target_path))
        .unzip())
}

#[cfg(test)]
mod tests;
