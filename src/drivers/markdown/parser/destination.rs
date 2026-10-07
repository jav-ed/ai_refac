//! Narrows a link the CommonMark parser already accepted down to the byte range
//! of its destination. Because the parser validated the structure, what is left
//! to read is small: skip the whitespace, then take either `<...>` or the run of
//! characters up to the next whitespace or unbalanced `)`. The text found is
//! compared with the destination the parser itself reported, so a range that
//! landed anywhere else is an error, never a guess.

use anyhow::{Result, bail};
use std::ops::Range;

/// The destination of an inline link `[text](destination "title")`, given the
/// byte where the text ended. `None` when the link has no destination (`[x]()`).
pub(super) fn of_inline_link(
    content: &str,
    text_end: usize,
    expected: &str,
) -> Result<Option<Range<usize>>> {
    if content.get(text_end..text_end + 2) != Some("](") {
        bail!("Unexpected Markdown link structure at byte {text_end}: expected `](`");
    }
    let destination = scan(content.as_bytes(), text_end + 2);
    check(content, &destination, expected)?;
    Ok(non_empty(destination))
}

/// The destination of a reference definition `[label]: destination "title"`,
/// given the byte range of the whole definition. `None` when it is empty.
pub(super) fn of_definition(
    content: &str,
    span: Range<usize>,
    expected: &str,
) -> Result<Option<Range<usize>>> {
    let bytes = content.as_bytes();
    if bytes.get(span.start) != Some(&b'[') {
        bail!(
            "Unexpected Markdown definition structure at byte {}: expected `[`",
            span.start
        );
    }
    let mut at = span.start + 1;
    while at < span.end && bytes[at] != b']' {
        at += if is_escape(bytes, at) { 2 } else { 1 };
    }
    if bytes.get(at + 1) != Some(&b':') {
        bail!(
            "Unexpected Markdown definition structure at byte {}: expected `]:`",
            span.start
        );
    }
    let destination = scan(bytes, at + 2);
    if destination.end > span.end {
        bail!(
            "Unexpected Markdown definition structure at byte {}: the destination leaves the definition",
            span.start
        );
    }
    check(content, &destination, expected)?;
    Ok(non_empty(destination))
}

/// The text in the range, with backslash escapes resolved, must be the
/// destination the parser reported.
fn check(content: &str, destination: &Range<usize>, expected: &str) -> Result<()> {
    let found = unescaped(&content[destination.clone()]);
    if found != expected {
        bail!(
            "Unexpected Markdown link structure at byte {}: read the destination `{found}` where the parser read `{expected}`",
            destination.start
        );
    }
    Ok(())
}

fn unescaped(raw: &str) -> String {
    let mut text = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\\' && chars.peek().is_some_and(char::is_ascii_punctuation) {
            text.extend(chars.next());
        } else {
            text.push(character);
        }
    }
    text
}

fn non_empty(range: Range<usize>) -> Option<Range<usize>> {
    (!range.is_empty()).then_some(range)
}

/// A backslash escapes the next byte only when that is ASCII punctuation.
fn is_escape(bytes: &[u8], at: usize) -> bool {
    bytes[at] == b'\\' && bytes.get(at + 1).is_some_and(u8::is_ascii_punctuation)
}

/// Skips the whitespace before a destination. When the gap crosses into a new
/// line inside a block quote, the `>` markers that start that line are skipped
/// too; they belong to the quote, not to the destination.
fn skip_gap(bytes: &[u8], mut at: usize) -> usize {
    let mut line_start = false;
    while let Some(&byte) = bytes.get(at) {
        match byte {
            b'\n' => line_start = true,
            b'>' if line_start => {}
            byte if byte.is_ascii_whitespace() => {}
            _ => break,
        }
        at += 1;
    }
    at
}

fn scan(bytes: &[u8], at: usize) -> Range<usize> {
    let mut at = skip_gap(bytes, at);

    if bytes.get(at) == Some(&b'<') {
        let start = at + 1;
        let mut end = start;
        while end < bytes.len() && bytes[end] != b'>' {
            end += if is_escape(bytes, end) { 2 } else { 1 };
        }
        return start..end.min(bytes.len());
    }

    let start = at;
    let mut depth = 0usize;
    while at < bytes.len() {
        match bytes[at] {
            b'(' => depth += 1,
            b')' if depth == 0 => break,
            b')' => depth -= 1,
            byte if byte.is_ascii_whitespace() || byte.is_ascii_control() => break,
            _ if is_escape(bytes, at) => at += 1,
            _ => {}
        }
        at += 1;
    }
    start..at.min(bytes.len())
}
