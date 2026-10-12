//! The server's changes to a file as the smallest edits: one per changed run
//! of tokens, so that a rename engine can still match each edit to a place.

use crate::drivers::lsp::text::TextIndex;
use lsp_types::{Range, TextEdit};
use similar::{Algorithm, DiffTag, capture_diff_slices};

/// The edits that turn `before` into `after`, one per changed run of tokens
/// (names, punctuation, blanks): as fine as the server's own, so a rename
/// that changes a name changes just the name.
pub(super) fn edits_between(before: &str, after: &str) -> Vec<TextEdit> {
    let (old, new) = (tokens(before), tokens(after));
    let (old_starts, new_starts) = (offsets(&old), offsets(&new));
    let index = TextIndex::new(before);
    capture_diff_slices(Algorithm::Myers, &old, &new)
        .iter()
        .filter_map(|op| {
            let (tag, old_range, new_range) = op.as_tag_tuple();
            (tag != DiffTag::Equal).then(|| TextEdit {
                range: Range::new(
                    index.position(old_starts[old_range.start]),
                    index.position(old_starts[old_range.end]),
                ),
                new_text: after[new_starts[new_range.start]..new_starts[new_range.end]].to_string(),
            })
        })
        .collect()
}

/// Names (and numbers) and runs of blanks are one token; every other
/// character, newlines included, is one by itself.
pub(super) fn tokens(text: &str) -> Vec<&str> {
    let class = |ch: char| match ch {
        _ if ch.is_alphanumeric() || ch == '_' => 1,
        ' ' | '\t' => 2,
        _ => 0,
    };
    let mut tokens = Vec::new();
    let (mut start, mut current) = (0, None);
    for (at, ch) in text.char_indices() {
        let kind = class(ch);
        if let Some(previous) = current {
            if kind != 0 && kind == previous {
                continue;
            }
            tokens.push(&text[start..at]);
            start = at;
        }
        current = Some(kind);
    }
    if start < text.len() {
        tokens.push(&text[start..]);
    }
    tokens
}

/// Where each token starts, and the end of the last one.
pub(super) fn offsets(tokens: &[&str]) -> Vec<usize> {
    let mut offsets = vec![0];
    for token in tokens {
        offsets.push(offsets.last().copied().unwrap_or(0) + token.len());
    }
    offsets
}
