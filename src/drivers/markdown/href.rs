//! Reading a link destination as a file path, and writing one back the way the
//! author wrote it.
//!
//! Only relative references to files are paths: `docs/guide.md#intro`,
//! `./img/logo.png?raw=1`, `../My%20Notes.md`. Web addresses, `#anchor`, site
//! rooted `/paths`, `mailto:` and the like are not, and neither is anything that
//! looks like a template (`{{ x }}`, `${x}`) or a path with an HTML entity in it.

use super::moves::normalize;
use std::path::{Path, PathBuf};

/// A destination split into the path and what follows it (`?query` and
/// `#fragment`), both exactly as written.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct PathReference<'a> {
    pub path: &'a str,
    pub suffix: &'a str,
}

/// `None` for everything that is not a relative file path.
pub(super) fn parse(href: &str) -> Option<PathReference<'_>> {
    let trimmed = href.trim();
    if trimmed.is_empty()
        || trimmed.starts_with('#')
        || trimmed.starts_with('/')
        || trimmed.starts_with('?')
        || trimmed.contains(['{', '}', '$', '\\'])
        || has_uri_scheme(trimmed)
    {
        return None;
    }
    let end = path_end(trimmed);
    let path = &trimmed[..end];
    // `&amp;` in a path would have to be decoded to find the file; the query
    // is kept as written, so entities there do no harm.
    if has_html_entity(path) {
        return None;
    }
    Some(PathReference {
        path,
        suffix: &trimmed[end..],
    })
}

/// Where the path ends and the `?query` or `#fragment` begins. The `#` of a
/// numeric entity (`&#38;`) is part of the path.
fn path_end(href: &str) -> usize {
    let mut previous = ' ';
    for (at, character) in href.char_indices() {
        if character == '?' || (character == '#' && previous != '&') {
            return at;
        }
        previous = character;
    }
    href.len()
}

/// Where a path written in `file` points, with percent-encoding resolved.
pub(super) fn resolve(file: &Path, written: &str) -> Option<PathBuf> {
    Some(normalize(&file.parent()?.join(decode(written))))
}

/// The destination to write for a link that now points at `target`, from the
/// file in `from_dir`. It keeps the author's habits: a `./` prefix only if the
/// old path had one (and the new one does not climb), a trailing `/` on
/// directory links, and `%20` style escaping where the old path escaped or the
/// destination is not wrapped in `<...>`.
pub(super) fn write(
    written: &str,
    from_dir: &Path,
    target: &Path,
    wrapped_in_angle_brackets: bool,
) -> Option<String> {
    let mut path = relative(target, from_dir)?;
    let directory_link = written.ends_with('/');
    if path.is_empty() {
        path = ".".to_string();
    }
    if directory_link && !path.ends_with('/') {
        path.push('/');
    }

    let climbs = path == ".." || path.starts_with("../");
    // `a:b.md` would read as a link with the scheme `a`.
    let first_segment_has_colon = path
        .split('/')
        .next()
        .is_some_and(|segment| segment.contains(':'));
    if !climbs
        && path != "."
        && !path.starts_with("./")
        && (written.starts_with("./") || first_segment_has_colon)
    {
        path = format!("./{path}");
    }

    let escape = written.contains('%') || !wrapped_in_angle_brackets;
    Some(if escape { encode(&path) } else { path })
}

/// `target` as a `/` separated path from `from_dir`.
pub(super) fn relative(target: &Path, from_dir: &Path) -> Option<String> {
    let relative = pathdiff::diff_paths(target, from_dir)?;
    Some(relative.to_string_lossy().replace('\\', "/"))
}

fn decode(written: &str) -> String {
    percent_encoding::percent_decode_str(written)
        .decode_utf8()
        .map_or_else(|_| written.to_string(), |decoded| decoded.into_owned())
}

/// Percent-encode what cannot stand in a link destination. Non-ASCII letters
/// stay as they are, which every Markdown renderer reads, and so do
/// parentheses that match each other (`a_(b).md`), which CommonMark allows.
fn encode(path: &str) -> String {
    let parentheses_match = parentheses_match(path);
    let mut encoded = String::with_capacity(path.len());
    for character in path.chars() {
        let parenthesis = matches!(character, '(' | ')') && !parentheses_match;
        let unsafe_character = " \"<>`[]%\\^#?".contains(character) || parenthesis;
        if character.is_ascii() && (character.is_ascii_control() || unsafe_character) {
            encoded.push_str(&format!("%{:02X}", character as u8));
        } else {
            encoded.push(character);
        }
    }
    encoded
}

fn parentheses_match(path: &str) -> bool {
    let mut depth = 0usize;
    for character in path.chars() {
        match character {
            '(' => depth += 1,
            ')' => {
                let Some(shallower) = depth.checked_sub(1) else {
                    return false;
                };
                depth = shallower;
            }
            _ => {}
        }
    }
    depth == 0
}

/// `&name;` or `&#123;`.
fn has_html_entity(text: &str) -> bool {
    text.match_indices('&').any(|(at, _)| {
        let rest = &text[at + 1..];
        let end = rest
            .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '#'))
            .unwrap_or(rest.len());
        end > 0 && rest[end..].starts_with(';')
    })
}

fn has_uri_scheme(href: &str) -> bool {
    let Some(colon) = href.find(':') else {
        return false;
    };
    let scheme = &href[..colon];
    // A single letter is a drive (`C:`), and a colon after `/` or `?` is no scheme.
    scheme.len() > 1
        && scheme.starts_with(|first: char| first.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'))
}

#[cfg(test)]
mod tests;
