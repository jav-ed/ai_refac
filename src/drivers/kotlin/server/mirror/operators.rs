//! Operators that the server turned into calls.
//!
//! When the server adds the import of an operator function a moved file needs,
//! it also rewrites some uses of that operator as an infix call: `a < b`
//! became `a compareTo b`, `3 in a` became `3 contains a` and `a[1] = 2`
//! became `a[1] set 2` (found on the real server, 2026-10-12). None of these
//! compiles: the function is not `infix`, and `contains` has the wrong
//! receiver. A move does not rewrite operators, so every change below the
//! package and import lines that replaces an operator symbol (or `in`) with
//! the name of an operator function is undone. The import the server added
//! for that function stays, and the operator is what it was.

use super::edits::{offsets, tokens};
use super::untouched::{header_end, words};
use similar::{Algorithm, DiffTag, capture_diff_slices};

/// The functions an operator stands for.
const OPERATOR_FUNCTIONS: &[&str] = &[
    "plus",
    "minus",
    "times",
    "div",
    "rem",
    "mod",
    "unaryPlus",
    "unaryMinus",
    "not",
    "inc",
    "dec",
    "rangeTo",
    "rangeUntil",
    "contains",
    "get",
    "set",
    "invoke",
    "compareTo",
    "plusAssign",
    "minusAssign",
    "timesAssign",
    "divAssign",
    "remAssign",
    "modAssign",
];

/// `after`, with every operator the server replaced by the name of its
/// function written as it was in `before`.
pub fn undo_operator_calls(before: &str, after: &str) -> String {
    if before == after {
        return after.to_string();
    }
    let (old, new) = (tokens(before), tokens(after));
    let (old_starts, new_starts) = (offsets(&old), offsets(&new));
    let header = header_end(before);
    let mut result = String::with_capacity(after.len());
    for op in capture_diff_slices(Algorithm::Myers, &old, &new) {
        let (tag, old_range, new_range) = op.as_tag_tuple();
        let old_text = &before[old_starts[old_range.start]..old_starts[old_range.end]];
        let new_text = &after[new_starts[new_range.start]..new_starts[new_range.end]];
        let called = tag == DiffTag::Replace
            && old_starts[old_range.start] > header
            && is_operator(&words(&old[old_range]))
            && matches!(words(&new[new_range]).as_slice(), [name] if is_function(name));
        result.push_str(if called { old_text } else { new_text });
    }
    result
}

/// Only symbols, or `in`: the words an operator is written with.
fn is_operator(words: &[&str]) -> bool {
    !words.is_empty()
        && words.iter().all(|word| {
            *word == "in"
                || word
                    .chars()
                    .all(|ch| !ch.is_alphanumeric() && ch != '_' && !ch.is_whitespace())
        })
}

fn is_function(name: &str) -> bool {
    OPERATOR_FUNCTIONS.contains(&name)
        || name
            .strip_prefix("component")
            .is_some_and(|n| !n.is_empty() && n.chars().all(|ch| ch.is_ascii_digit()))
}

#[cfg(test)]
mod tests;
