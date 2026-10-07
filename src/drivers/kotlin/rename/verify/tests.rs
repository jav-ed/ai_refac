use super::*;

const RUNNER: &str =
    "package a\n\nimport com.example.Helper\nimport com.example.util.undecorate\n\nclass Runner\n";

fn span(text: &str, needle: &str) -> (usize, usize) {
    let start = text.find(needle).unwrap();
    (start, start + needle.len())
}

#[test]
fn the_import_of_the_symbol_may_be_rewritten_or_dropped() {
    let (start, end) = span(RUNNER, "import com.example.util.undecorate");
    assert!(edits_import_of_symbol(RUNNER, start, end, "undecorate"));
    // The line break goes with the line.
    assert!(edits_import_of_symbol(RUNNER, start, end + 1, "undecorate"));
    // Part of the name, as a character-level edit.
    let (start, end) = span(RUNNER, "un");
    assert!(edits_import_of_symbol(RUNNER, start, end, "undecorate"));
}

#[test]
fn other_imports_and_other_lines_are_not_the_symbols_import() {
    let (start, end) = span(RUNNER, "import com.example.Helper");
    assert!(!edits_import_of_symbol(RUNNER, start, end, "undecorate"));
    let (start, end) = span(RUNNER, "class Runner");
    assert!(!edits_import_of_symbol(RUNNER, start, end, "undecorate"));
    // Only the last path segment names the symbol.
    let text = "import undecorate.Other\n";
    let (start, end) = span(text, "undecorate");
    assert!(!edits_import_of_symbol(text, start, end, "undecorate"));
    // An edit that reaches past the import line is not an import edit.
    let (start, _) = span(RUNNER, "import com.example.util.undecorate");
    let (_, end) = span(RUNNER, "class Runner");
    assert!(!edits_import_of_symbol(RUNNER, start, end, "undecorate"));
}

#[test]
fn an_aliased_import_still_names_the_symbol() {
    let text = "import com.example.util.undecorate as plain\n";
    let (start, end) = span(text, "undecorate");
    assert!(edits_import_of_symbol(text, start, end, "undecorate"));
}

#[test]
fn only_whole_blank_lines_count_as_tidying() {
    let text = "import a.B\n\n\nclass C\n";
    let (start, end) = (11, 12);
    assert!(removes_blank_lines(text, start, end, ""));
    // Replacing instead of removing, or touching code, is not tidying.
    assert!(!removes_blank_lines(text, start, end, "x"));
    let (start, end) = span(text, "class C\n");
    assert!(!removes_blank_lines(text, start, end, ""));
    // Whitespace that starts mid-line joins tokens.
    let (start, end) = (10, 12);
    assert!(!removes_blank_lines(text, start, end, ""));
    assert!(!removes_blank_lines(text, 3, 3, ""));
}
