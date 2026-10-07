//! Checking a Dart move plan before anything is written. The Dart server
//! answers "no edits", or only some of them, when it has not finished
//! analysing, and the move would then look like a success while imports point
//! at files that no longer exist. Every `import`, `export` and `part`
//! directive that names a file of this project is resolved against the
//! project as it will be after the move; one that points at nothing, and did
//! not point at nothing before, is reported.

use crate::drivers::lsp_text::apply_text_edits;
use anyhow::{Context, Result};
use lsp_types::TextEdit;
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};

/// Directives that would dangle after the move, as one message each.
/// `files` are the project's Dart files, `moves` are absolute (from, to)
/// pairs and `edits` the planned text edits by the file's current path.
pub fn dangling_after(
    root: &Path,
    files: &[PathBuf],
    moves: &[(PathBuf, PathBuf)],
    edits: &HashMap<PathBuf, Vec<TextEdit>>,
) -> Result<Vec<String>> {
    let package = package_name(root)?;
    let sources: HashSet<&PathBuf> = moves.iter().map(|(from, _)| from).collect();
    let targets: HashSet<&PathBuf> = moves.iter().map(|(_, to)| to).collect();
    let new_place: HashMap<&PathBuf, &PathBuf> =
        moves.iter().map(|(from, to)| (from, to)).collect();
    let exists_before = |path: &Path| path.is_file();
    let exists_after = |path: &Path| {
        targets.contains(&path.to_path_buf())
            || (path.is_file() && !sources.contains(&path.to_path_buf()))
    };

    let mut dangling = Vec::new();
    for file in files {
        let old_text = std::fs::read_to_string(file)
            .with_context(|| format!("Cannot read {}", file.display()))?;
        let new_text = match edits.get(file) {
            Some(file_edits) => apply_text_edits(&old_text, file_edits.clone())
                .with_context(|| format!("Cannot apply the planned edits to {}", file.display()))?,
            None => old_text.clone(),
        };
        let old_uris = directives(&old_text);
        let after = new_place
            .get(file)
            .map_or(file.as_path(), |to| to.as_path());
        for uri in directives(&new_text) {
            let Some(target) = resolve(&uri, after, root, package.as_deref()) else {
                continue;
            };
            if exists_after(&target) {
                continue;
            }
            // A directive that was already broken is not this move's doing.
            let broken_before = old_uris.contains(&uri)
                && resolve(&uri, file, root, package.as_deref())
                    .is_some_and(|before| !exists_before(&before));
            if broken_before {
                continue;
            }
            dangling.push(format!(
                "{} would import '{uri}', which does not exist after the move",
                after.strip_prefix(root).unwrap_or(after).display()
            ));
        }
    }
    Ok(dangling)
}

/// `name:` of the project's `pubspec.yaml`, which the `package:` URIs of its
/// own files start with. A project without one has no local package URIs.
fn package_name(root: &Path) -> Result<Option<String>> {
    let pubspec = root.join("pubspec.yaml");
    if !pubspec.is_file() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&pubspec)
        .with_context(|| format!("Cannot read {}", pubspec.display()))?;
    Ok(text.lines().find_map(|line| {
        let name = line.strip_prefix("name:")?.trim().trim_matches(['"', '\'']);
        (!name.is_empty()).then(|| name.to_string())
    }))
}

/// The file a directive names, `None` when it names something outside this
/// project (the SDK, another package, a URL).
fn resolve(uri: &str, from: &Path, root: &Path, package: Option<&str>) -> Option<PathBuf> {
    if let Some(rest) = uri.strip_prefix("package:") {
        let (name, path) = rest.split_once('/')?;
        return (Some(name) == package).then(|| normalize(&root.join("lib").join(path)));
    }
    if uri.contains(':') {
        return None;
    }
    Some(normalize(&from.parent()?.join(uri)))
}

/// `.` and `..` folded away without touching the file system.
fn normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other),
        }
    }
    normalized
}

/// The quoted URIs of the leading `import`, `export` and `part` directives of
/// a Dart file. Directives come before any declaration, so scanning stops at
/// the first declaration; comments and annotations are skipped.
pub fn directives(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut at = 0;
    let mut uris = Vec::new();
    // A script tag line (`#!/usr/bin/env dart`) comes first.
    if text.starts_with("#!") {
        skip_line(&chars, &mut at);
    }
    loop {
        skip_trivia(&chars, &mut at);
        if at >= chars.len() {
            return uris;
        }
        if chars[at] == '@' {
            skip_annotation(&chars, &mut at);
            continue;
        }
        match read_word(&chars, &mut at).as_str() {
            "library" => skip_statement(&chars, &mut at),
            "import" | "export" | "part" => read_statement(&chars, &mut at, &mut uris),
            _ => return uris,
        }
    }
}

fn skip_line(chars: &[char], at: &mut usize) {
    while *at < chars.len() && chars[*at] != '\n' {
        *at += 1;
    }
}

/// Whitespace, `//` comments and (nested) `/* */` comments.
fn skip_trivia(chars: &[char], at: &mut usize) {
    loop {
        match (chars.get(*at), chars.get(*at + 1)) {
            (Some(ch), _) if ch.is_whitespace() => *at += 1,
            (Some('/'), Some('/')) => skip_line(chars, at),
            (Some('/'), Some('*')) => {
                let mut depth = 0;
                while *at < chars.len() {
                    match (chars[*at], chars.get(*at + 1)) {
                        ('/', Some('*')) => {
                            depth += 1;
                            *at += 2;
                        }
                        ('*', Some('/')) => {
                            depth -= 1;
                            *at += 2;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => *at += 1,
                    }
                }
            }
            _ => return,
        }
    }
}

/// `@name`, `@a.b` and `@name(...)`.
fn skip_annotation(chars: &[char], at: &mut usize) {
    *at += 1;
    while *at < chars.len()
        && (chars[*at].is_alphanumeric() || matches!(chars[*at], '_' | '$' | '.'))
    {
        *at += 1;
    }
    if chars.get(*at) == Some(&'(') {
        let mut depth = 0;
        while *at < chars.len() {
            match chars[*at] {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
            *at += 1;
            if depth == 0 {
                break;
            }
        }
    }
}

fn read_word(chars: &[char], at: &mut usize) -> String {
    let start = *at;
    while *at < chars.len() && (chars[*at].is_alphanumeric() || matches!(chars[*at], '_' | '$')) {
        *at += 1;
    }
    if *at == start {
        // A character that starts no directive: end of the leading directives.
        *at = chars.len();
    }
    chars[start..*at].iter().collect()
}

fn skip_statement(chars: &[char], at: &mut usize) {
    while *at < chars.len() && chars[*at] != ';' {
        *at += 1;
    }
    *at += 1;
}

/// Every quoted string up to the closing `;`.
fn read_statement(chars: &[char], at: &mut usize, uris: &mut Vec<String>) {
    while *at < chars.len() && chars[*at] != ';' {
        match chars[*at] {
            quote @ ('\'' | '"') => {
                *at += 1;
                let start = *at;
                while *at < chars.len() && chars[*at] != quote {
                    *at += 1;
                }
                uris.push(chars[start..(*at).min(chars.len())].iter().collect());
                *at += 1;
            }
            '/' if matches!(chars.get(*at + 1), Some('/' | '*')) => skip_trivia(chars, at),
            _ => *at += 1,
        }
    }
    *at += 1;
}

#[cfg(test)]
mod tests;
