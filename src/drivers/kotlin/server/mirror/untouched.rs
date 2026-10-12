//! Layout that the server changed and nobody asked for.
//!
//! A move edits the package line and the imports. The server also tidies the
//! code of the files it edits: a call that stood on one line came back on four
//! (found on a real project, 2026-10-12). Such an edit changes no name and no
//! punctuation, only blanks and line breaks, and a move must not do it: it
//! makes a diff of a refactoring unreadable. Every change outside the package
//! and import lines whose words are the same before and after is undone.

use super::edits::{offsets, tokens};
use similar::{Algorithm, DiffTag, capture_diff_slices};

/// `after`, with the layout of `before` wherever the server changed nothing
/// but layout below the package and import lines.
pub fn undo_reformatting(before: &str, after: &str) -> String {
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
        let only_layout = tag != DiffTag::Equal
            && old_starts[old_range.start] > header
            && words(&old[old_range]) == words(&new[new_range]);
        result.push_str(if only_layout { old_text } else { new_text });
    }
    result
}

/// The tokens that are not blanks.
fn words<'a>(tokens: &[&'a str]) -> Vec<&'a str> {
    tokens
        .iter()
        .copied()
        .filter(|token| !token.trim().is_empty())
        .collect()
}

/// Where the package and import lines end: after the last of them and the
/// blank lines that follow. What the server adds or removes there is the move.
fn header_end(text: &str) -> usize {
    let (mut end, mut offset) = (0, 0);
    let mut in_header = false;
    for line in text.split_inclusive('\n') {
        offset += line.len();
        let trimmed = line.trim_start();
        if trimmed.starts_with("package ") || trimmed.starts_with("import ") {
            (end, in_header) = (offset, true);
        } else if in_header && line.trim().is_empty() {
            end = offset;
        } else {
            in_header = false;
        }
    }
    end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_call_the_server_wrapped_comes_back_on_one_line() {
        let before = "package a\n\nimport x.Y\n\nfun f() {\n    Chip(language, selected == language, { pick() })\n}\n";
        let after = "package b\n\nimport x.Y\nimport a.Chip\n\nfun f() {\n    Chip(\n        language,\n        selected == language,\n        { pick() }\n    )\n}\n";

        assert_eq!(
            undo_reformatting(before, after),
            "package b\n\nimport x.Y\nimport a.Chip\n\nfun f() {\n    Chip(language, selected == language, { pick() })\n}\n"
        );
    }

    #[test]
    fn changes_to_the_package_and_the_imports_are_the_move_and_stay() {
        let before = "package a\n\nimport x.Y\n\nclass C\n";
        let after = "package b\n\nimport x.Y\nimport a.Z\n\nclass C\n";
        assert_eq!(undo_reformatting(before, after), after);

        // A file with no import yet: the block goes under the package line.
        let before = "package a\n\nclass C\n";
        let after = "package b\n\nimport a.Z\n\nclass C\n";
        assert_eq!(undo_reformatting(before, after), after);
    }

    #[test]
    fn a_change_of_a_name_is_kept_even_with_a_reformat_next_to_it() {
        let before = "package a\n\nfun f() = Old(1, 2)\n";
        let after = "package a\n\nfun f() = New(\n    1,\n    2\n)\n";

        assert_eq!(
            undo_reformatting(before, after),
            "package a\n\nfun f() = New(1, 2)\n"
        );
    }

    #[test]
    fn text_that_is_the_same_is_returned_as_it_is() {
        let text = "package a\n\nfun f() = 1\n";
        assert_eq!(undo_reformatting(text, text), text);
        assert_eq!(undo_reformatting("", ""), "");
    }
}
