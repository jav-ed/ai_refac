//! Edits inside comments, which some servers make when they rename a symbol:
//! gopls rewrites the name where a doc comment mentions it. A comment is not
//! code, so such an edit cannot change meaning, but it lies outside every
//! reference the server lists, which the engine otherwise refuses.

use crate::drivers::lsp::rename::language::EditedText;

/// Which edits replace the symbol on a line that is a comment from its first
/// non-blank character on (`//` for Go, Rust, Dart; `#` for Python). An edit
/// that changes anything but exactly the symbol is never exempt.
pub fn on_comment_lines(edited: &EditedText, markers: &[&str]) -> Vec<bool> {
    edited
        .spans
        .iter()
        .map(|(start, end)| {
            edited.before.get(*start..*end) == Some(edited.symbol)
                && is_comment(line_before(edited.before, *start), markers)
        })
        .collect()
}

/// The text of the line up to `offset`.
fn line_before(text: &str, offset: usize) -> &str {
    let line_start = text[..offset].rfind('\n').map_or(0, |at| at + 1);
    &text[line_start..offset]
}

fn is_comment(line_prefix: &str, markers: &[&str]) -> bool {
    let trimmed = line_prefix.trim_start();
    markers.iter().any(|marker| trimmed.starts_with(marker))
}

#[cfg(test)]
mod tests;
