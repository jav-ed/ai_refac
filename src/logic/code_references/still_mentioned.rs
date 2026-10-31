//! Files that still name a path that moved.
//!
//! The TypeScript engine rewrites the paths it can read in code, stylesheets
//! and pages. Configuration and data files (`package.json` scripts, `tsconfig`
//! entries, CI files, JSON and YAML data, a `justfile`) name paths too, and so
//! does a URL like `/brand/logo.svg` for a file of `public/`. refac does not
//! edit those, so after a move it lists the ones that still contain an old
//! path, instead of leaving the break for the build to find.
//!
//! Only text that spells the whole path from the project root is a mention:
//! `src/scripts/menu.ts` or `./src/scripts/menu.ts`, not `./menu` (which
//! could be any file of that name).

use super::imports::Groups;
use crate::drivers::typescript::files;
use aho_corasick::AhoCorasick;
use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Configuration and data formats; code, stylesheets and pages are the
/// engine's business, and lock files only list packages.
const TEXT_EXTENSIONS: &[&str] = &[
    "json", "jsonc", "json5", "yaml", "yml", "toml", "txt", "html", "htm", "xml", "cfg", "ini",
    "conf", "sh", "bash", "just", "mk",
];

const LARGEST_FILE: u64 = 1_000_000;
const MOST_FILES_SHOWN: usize = 10;

/// The old paths of a request, when the TypeScript engine takes part in it:
/// it is the one that rewrites paths in code, so its limits are what this
/// note makes visible. Taken before anything moves.
pub(in crate::logic) fn collect_for(
    groups: &Groups,
    companion: &[(String, String)],
    root: Option<&Path>,
) -> Option<Mentions> {
    let involved = !companion.is_empty() || groups.iter().any(|(lang, ..)| lang == "typescript");
    if !involved {
        return None;
    }
    let root = root
        .map(Path::to_path_buf)
        .or_else(|| std::env::current_dir().ok())?;
    let sources = groups
        .iter()
        .flat_map(|(_, files, _)| files.iter().map(|(source, _)| source));
    Some(Mentions::collect(sources, &root))
}

/// The old paths of a move.
pub(in crate::logic) struct Mentions {
    root: PathBuf,
    /// The path as written from the project root, and whether it is a folder.
    paths: Vec<(String, bool)>,
}

impl Mentions {
    /// `sources` are the paths the request moves, relative to `root` or absolute.
    pub(in crate::logic) fn collect<'a>(
        sources: impl Iterator<Item = &'a String>,
        root: &Path,
    ) -> Self {
        let mut paths = Vec::new();
        for source in sources {
            let absolute = root.join(source);
            let Ok(relative) = absolute.strip_prefix(root) else {
                continue;
            };
            let written = relative.to_string_lossy().replace('\\', "/");
            let folder = absolute.is_dir();
            // A file of `public/` is a URL of the site too.
            if let Some((first, rest)) = written.split_once('/')
                && first.eq_ignore_ascii_case("public")
            {
                paths.push((format!("/{rest}"), folder));
            }
            paths.push((written, folder));
        }
        Self {
            root: root.to_path_buf(),
            paths,
        }
    }

    /// A note naming the files that still hold an old path, none when no file does.
    /// `edited` are files the move changes anyway (a dry run does not change them yet).
    pub(in crate::logic) fn scan(&self, edited: &HashSet<PathBuf>) -> Vec<String> {
        match self.find(edited) {
            Ok(found) => note(&found),
            Err(error) => vec![format!(
                "Could not look for files that still name a moved path: {error:#}"
            )],
        }
    }

    fn find(&self, edited: &HashSet<PathBuf>) -> Result<Vec<(String, String)>> {
        if self.paths.is_empty() {
            return Ok(Vec::new());
        }
        let automaton = AhoCorasick::new(self.paths.iter().map(|(path, _)| path))?;
        let mut found = Vec::new();
        for file in files::find(&self.root, TEXT_EXTENSIONS)? {
            let name = file.file_name().map(|name| name.to_string_lossy());
            let lock = name.is_some_and(|name| name.contains("lock"));
            let large = file.metadata().is_ok_and(|meta| meta.len() > LARGEST_FILE);
            if lock || large || edited.contains(&file) {
                continue;
            }
            // Not text, or not readable: nothing to read a path from.
            let Ok(bytes) = std::fs::read(&file) else {
                continue;
            };
            let hit = automaton.find_iter(&bytes).find(|hit| {
                let (_, folder) = &self.paths[hit.pattern().as_usize()];
                names_the_path(&bytes, hit.start(), hit.end(), *folder)
            });
            if let Some(hit) = hit {
                let shown = file.strip_prefix(&self.root).unwrap_or(&file);
                found.push((
                    shown.to_string_lossy().replace('\\', "/"),
                    self.paths[hit.pattern().as_usize()].0.clone(),
                ));
            }
        }
        found.sort();
        Ok(found)
    }
}

fn note(found: &[(String, String)]) -> Vec<String> {
    if found.is_empty() {
        return Vec::new();
    }
    let shown: Vec<String> = found
        .iter()
        .take(MOST_FILES_SHOWN)
        .map(|(file, path)| format!("{file} ({path})"))
        .collect();
    let more = found.len().saturating_sub(MOST_FILES_SHOWN);
    let tail = if more > 0 {
        format!(", and {more} more")
    } else {
        String::new()
    };
    vec![format!(
        "These files still name a path that moved and were not changed: {}{tail}.",
        shown.join(", ")
    )]
}

/// Whether the text at `start..end` is the whole path: it does not continue a
/// longer name on either side. A folder may continue with `/`.
fn names_the_path(text: &[u8], start: usize, end: usize, folder: bool) -> bool {
    let before = start.checked_sub(1).and_then(|index| text.get(index));
    let before_that = start.checked_sub(2).and_then(|index| text.get(index));
    let starts = match before {
        None => true,
        Some(b'/') => before_that == Some(&b'.') && text[start] != b'/',
        Some(byte) => !(byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.')),
    };
    let ends = match text.get(end) {
        None => true,
        Some(b'/') => folder,
        Some(byte) => !(byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.')),
    };
    starts && ends
}

#[cfg(test)]
mod tests;
