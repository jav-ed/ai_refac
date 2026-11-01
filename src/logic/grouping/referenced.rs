//! Markdown-side paths that code refers to.
//!
//! A page, an image or a folder of them is moved by the Markdown driver, which
//! fixes the Markdown links. Code that imports such a file (`import art from
//! "./hero.svg"` in a component, `import Card from "../Card.astro"` in an
//! `.mdx` page) is the business of the TypeScript engine, which needs Bun and
//! reads the whole project. The engine is asked for those paths only when
//! something uses them, so moving pictures that no code touches never starts it,
//! and an unrelated file it cannot read never blocks such a move.
//!
//! "Uses" is read from the text: a code file that has the file name written as
//! the end of a quoted path, the name of a moved folder written in a path
//! (`base: "./src/Content/blog"`), or a moved `.mdx` page with `import`/`export`
//! lines or relative frontmatter paths of its own (they change with the
//! folder). A page's frontmatter names pictures too (`image: ../a/hero.png`),
//! which no Markdown link covers. A Markdown link to the same file
//! (`](img/hero.png)`) is not a use; the Markdown driver handles it. A project
//! with an `import.meta.glob` or `Astro.glob` call uses every file: a glob names
//! a folder, and only the engine tells whether a moved file is one of the files
//! it selects. Only a call counts (`glob_calls`): prose, a fenced example and a
//! bare mention do not. The reading over-approximates (a quoted attribute
//! counts), and the engine then finds nothing to change, which costs time and
//! nothing else.

mod glob_calls;

use super::route::resolve;
use crate::drivers::typescript::files;
use aho_corasick::AhoCorasick;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;

type Pairs = Vec<(String, String)>;

/// Calls that select files by pattern. The engine tells whether a moved file
/// starts or stops being selected, so a project that has one asks it for every
/// move.
const GLOB_CALLS: &[&str] = &["import.meta.glob", "Astro.glob"];

/// The files the TypeScript engine reads for imports.
const CODE_EXTENSIONS: &[&str] = &[
    "ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs", "astro", "mdx", "md", "css",
];

/// The pairs of the `markdown` batch that code uses.
pub(super) fn used_pairs(batch_map: &HashMap<String, Pairs>, root: Option<&Path>) -> Result<Pairs> {
    let Some(pairs) = batch_map.get("markdown") else {
        return Ok(Vec::new());
    };
    let used = used_by_code(pairs, root)?;
    Ok(pairs
        .iter()
        .zip(used)
        .filter(|(_, used)| *used)
        .map(|(pair, _)| pair.clone())
        .collect())
}

/// For each pair, whether code uses the file or one of the files of the
/// folder.
fn used_by_code(pairs: &[(String, String)], root: Option<&Path>) -> Result<Vec<bool>> {
    let mut used = vec![false; pairs.len()];
    // Which pairs each name belongs to: every file name, and the name of a
    // moved folder itself (code names a folder, not its files, in a path).
    let mut owners: HashMap<String, Owner> = HashMap::new();
    for (index, (source, _)) in pairs.iter().enumerate() {
        let path = resolve(Path::new(source), root);
        for entry in WalkDir::new(&path) {
            let entry = entry.with_context(|| format!("Cannot read inside {}", path.display()))?;
            let folder = entry.depth() == 0 && entry.file_type().is_dir();
            if !folder && !entry.file_type().is_file() {
                continue;
            }
            if has_own_imports(entry.path())? {
                used[index] = true;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            let owner = owners.entry(name).or_default();
            owner.pairs.push(index);
            owner.folder |= folder;
        }
    }
    if used.iter().all(|used| *used) {
        return Ok(used);
    }

    // The file names, then the calls that select files by pattern: a glob names a
    // folder, not the files it selects, so any moved file may be one of them.
    let names: Vec<&String> = owners.keys().collect();
    let patterns: Vec<&str> = names
        .iter()
        .map(|name| name.as_str())
        .chain(GLOB_CALLS.iter().copied())
        .collect();
    let automaton = AhoCorasick::new(&patterns)
        .context("Cannot search the code for the requested file names")?;
    let scan_root = match root {
        Some(root) => root.to_path_buf(),
        None => std::env::current_dir().context("Cannot read the current directory")?,
    };
    for file in files::find(&scan_root, CODE_EXTENSIONS)? {
        // Bytes, not text: a file that is not UTF-8 may still use a name, and
        // the engine reports it precisely if it cannot read it.
        let bytes =
            std::fs::read(&file).with_context(|| format!("Cannot read {}", file.display()))?;
        for found in automaton.find_iter(&bytes) {
            let Some(name) = names.get(found.pattern().as_usize()) else {
                if glob_calls::is_call(&bytes, found.start(), found.end(), &extension(&file)) {
                    used.fill(true);
                    return Ok(used);
                }
                continue;
            };
            let owner = &owners[*name];
            if !ends_a_path(
                &bytes,
                found.start(),
                found.end(),
                Text::of(&file),
                owner.folder,
            ) {
                continue;
            }
            for index in &owner.pairs {
                used[*index] = true;
            }
        }
        if used.iter().all(|used| *used) {
            break;
        }
    }
    Ok(used)
}

/// The pairs a name belongs to, and whether one of them is a moved folder of
/// that name.
#[derive(Default)]
struct Owner {
    pairs: Vec<usize>,
    folder: bool,
}

/// What a scanned file is, which decides how it writes a path without quotes.
#[derive(Clone, Copy, PartialEq)]
enum Text {
    Code,
    Stylesheet,
    /// `.md` and `.mdx`: a frontmatter value ends at the end of its line.
    Page,
}

impl Text {
    fn of(file: &Path) -> Self {
        match extension(file).as_str() {
            "css" => Self::Stylesheet,
            "md" | "mdx" => Self::Page,
            _ => Self::Code,
        }
    }
}

fn extension(file: &Path) -> String {
    file.extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
}

/// Whether the name at `start..end` is the last part of a path written in
/// code: quoted (`"./hero.svg"`, `'../a/hero.svg?url'`, `` `hero.svg#x` ``),
/// in a stylesheet inside `url(...)` (`url(../hero.svg)`), or in a page at the
/// end of a frontmatter line (`image: ../hero.svg`). A folder name may also be
/// followed by `/` (`"./src/Content/blog/ar"`). A longer name that ends in the
/// same letters, and a Markdown link, are not.
fn ends_a_path(text: &[u8], start: usize, end: usize, kind: Text, folder: bool) -> bool {
    let before = start.checked_sub(1).and_then(|index| text.get(index));
    let after = text.get(end);
    let quoted = matches!(before, Some(b'/' | b'"' | b'\'' | b'`'))
        && (matches!(after, Some(b'"' | b'\'' | b'`' | b'?' | b'#'))
            || (folder && after == Some(&b'/')));
    let unquoted = kind == Text::Stylesheet
        && matches!(before, Some(b'/' | b'(' | b' '))
        && matches!(after, Some(b')' | b' ' | b'?' | b'#'));
    let line_end = kind == Text::Page
        && before == Some(&b'/')
        && matches!(after, None | Some(b'\n' | b'\r' | b' '));
    quoted || unquoted || line_end
}

/// A moved page with relative paths in its frontmatter or, for `.mdx`, with
/// `import` or `export` lines, or a moved stylesheet with `url(...)` or
/// `@import`: their relative paths change when the file changes folder.
fn has_own_imports(file: &Path) -> Result<bool> {
    let kind = Text::of(file);
    if kind == Text::Code {
        return Ok(false);
    }
    let bytes = std::fs::read(file).with_context(|| format!("Cannot read {}", file.display()))?;
    let text = String::from_utf8_lossy(&bytes);
    if kind == Text::Stylesheet {
        return Ok(text.to_ascii_lowercase().contains("url(") || text.contains("@import"));
    }
    if frontmatter_has_relative_path(&text) {
        return Ok(true);
    }
    Ok(extension(file) == "mdx"
        && text.lines().any(|line| {
            ["import", "export"].iter().any(|keyword| {
                line.strip_prefix(keyword)
                    .and_then(|rest| rest.chars().next())
                    .is_some_and(|next| matches!(next, ' ' | '\t' | '{' | '*'))
            })
        }))
}

/// Whether a value of the frontmatter starts like a relative path (`./` or
/// `../`, quoted or not, as a list item or after a key). A text that does not
/// start with `---` has no frontmatter. This over-approximates; the engine
/// checks that the value names a file.
fn frontmatter_has_relative_path(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    if lines
        .first()
        .map(|line| line.trim_start_matches('\u{feff}').trim_end())
        != Some("---")
    {
        return false;
    }
    let Some(close) = lines[1..].iter().position(|line| line.trim_end() == "---") else {
        return false;
    };
    lines[1..=close].iter().any(|line| {
        let mut value = line.trim_start();
        while let Some(rest) = value.strip_prefix("- ") {
            value = rest.trim_start();
        }
        if let Some((_, rest)) = value.split_once(": ") {
            value = rest.trim_start();
        }
        let value = value.trim_start_matches(['"', '\'']);
        value.starts_with("./") || value.starts_with("../")
    })
}

#[cfg(test)]
mod tests;
