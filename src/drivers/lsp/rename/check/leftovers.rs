//! After a rename: where the old name is still written. A language server
//! links a name to its symbol only where it can prove the link, so a method
//! called on an untyped value, an override in a dynamic language, or a name
//! inside a string is left alone. The scan cannot tell those from another,
//! unrelated symbol that merely shares the name, so it reports instead of
//! deciding: the caller sees how many matches remain and where the first are.

use crate::drivers::lsp::rename::language::Language;
use crate::drivers::lsp::rename::plan::discover::RenamePlan;
use anyhow::{Context, Result};
use ignore::WalkBuilder;
use std::path::Path;

/// Folders that hold generated or third-party code in every language refac renames.
const SKIPPED_FOLDERS: [&str; 7] = [
    "node_modules",
    "target",
    "build",
    "dist",
    "__pycache__",
    "vendor",
    "site-packages",
];
const MAX_FILE_BYTES: u64 = 2_000_000;
/// How many places the note names; the rest is a count.
const LISTED: usize = 6;

/// A whole-word match of the old name that the rename leaves in place.
struct Match {
    path: String,
    line: usize,
    /// Set when the match lies where the server is known not to rename.
    unrenamed: Option<&'static str>,
}

/// Notes about the old name that is still written in `root`'s sources: first
/// an attention note for places the server never renames and the build
/// depends on, then one general note. Edited files are read as they will be
/// after the rename, so the scan can run before anything is written.
pub fn scan(
    root: &Path,
    language: &dyn Language,
    symbol: &str,
    plan: &RenamePlan,
) -> Result<Vec<String>> {
    let mut matches = Vec::new();
    for path in sources(root, language)? {
        let text = match plan.files.iter().find(|edited| edited.file.path == path) {
            Some(edited) => edited.file.text.clone(),
            None => match std::fs::metadata(&path) {
                Ok(meta) if meta.len() <= MAX_FILE_BYTES => {
                    match crate::drivers::symbol::view::read_to_string(&path) {
                        Ok(text) => text,
                        // Not valid UTF-8: not source this scan can read.
                        Err(_) => continue,
                    }
                }
                _ => continue,
            },
        };
        let shown = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string();
        matches.extend(matches_in(&text, &shown, symbol, language));
    }
    if matches.is_empty() {
        return Ok(Vec::new());
    }
    let mut notes = Vec::new();
    let unrenamed: Vec<&Match> = matches
        .iter()
        .filter(|found| found.unrenamed.is_some())
        .collect();
    if let Some(first) = unrenamed.first() {
        let places: Vec<String> = unrenamed
            .iter()
            .take(LISTED)
            .map(|found| format!("{}:{}", found.path, found.line))
            .collect();
        notes.push(format!(
            "ATTENTION: `{symbol}` is still written inside {} ({}), which the language server does not rename. The program does not build until you change {} yourself.",
            first.unrenamed.unwrap_or("code the server does not rename"),
            places.join(", "),
            if unrenamed.len() == 1 { "it" } else { "them" }
        ));
    }
    let files = matches
        .iter()
        .map(|found| found.path.as_str())
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let first: Vec<String> = matches
        .iter()
        .take(LISTED)
        .map(|found| format!("{}:{}", found.path, found.line))
        .collect();
    notes.push(format!(
        "`{symbol}` is still written on {} line{} in {} file{} that the rename did not change (first: {}). They can be other symbols with the same name, text in strings or comments, or uses the language server could not link to this symbol (untyped values, overrides, dynamic access). Check them with `rg -w {symbol}`.",
        matches.len(),
        plural(matches.len()),
        files,
        plural(files),
        first.join(", ")
    ));
    Ok(notes)
}

/// One match per line that holds the symbol as a whole word.
fn matches_in(text: &str, path: &str, symbol: &str, language: &dyn Language) -> Vec<Match> {
    let places = language.unrenamed_places(text);
    let mut found = Vec::new();
    let mut line_start = 0;
    for (index, line) in text.split_inclusive('\n').enumerate() {
        if let Some(column) = word_in(line, symbol, language) {
            let offset = line_start + column;
            found.push(Match {
                path: path.to_string(),
                line: index + 1,
                unrenamed: places
                    .iter()
                    .find(|place| place.start <= offset && offset < place.end)
                    .map(|place| place.what),
            });
        }
        line_start += line.len();
    }
    found
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

fn sources(root: &Path, language: &dyn Language) -> Result<Vec<std::path::PathBuf>> {
    let walker = WalkBuilder::new(root)
        .parents(false)
        .git_global(false)
        .require_git(false)
        .filter_entry(|entry| {
            entry
                .file_name()
                .to_str()
                .is_none_or(|name| !SKIPPED_FOLDERS.contains(&name))
        })
        .build();
    let mut files = Vec::new();
    for entry in walker {
        let entry = entry.with_context(|| format!("Cannot scan {}", root.display()))?;
        let is_source = entry.file_type().is_some_and(|kind| kind.is_file())
            && entry
                .path()
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| language.extensions().contains(&extension));
        if is_source {
            files.push(entry.into_path());
        }
    }
    files.sort();
    Ok(files)
}

/// Where `line` holds `symbol` as a whole word (the first place).
fn word_in(line: &str, symbol: &str, language: &dyn Language) -> Option<usize> {
    line.match_indices(symbol)
        .map(|(offset, _)| offset)
        .find(|offset| {
            let before = line[..*offset].chars().next_back();
            let after = line[offset + symbol.len()..].chars().next();
            !before.is_some_and(|c| language.is_identifier_char(c))
                && !after.is_some_and(|c| language.is_identifier_char(c))
        })
}

#[cfg(test)]
mod tests;
