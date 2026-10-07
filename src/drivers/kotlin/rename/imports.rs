//! The edits the Kotlin server makes outside every reference of a renamed
//! symbol, which the engine's inside-a-reference check has to expect: the
//! symbol's own import line.

use crate::drivers::lsp_rename::language::EditedText;

/// Which edits of one file are import tidying. The server lists no reference
/// for the import of an extension, yet it rewrites or drops that line, and it
/// tidies the blank lines around a dropped import.
pub fn exempt(edited: &EditedText) -> Vec<bool> {
    let import_edited = edited
        .spans
        .iter()
        .any(|(start, end)| edits_import_of_symbol(edited.before, *start, *end, edited.symbol));
    edited
        .spans
        .iter()
        .zip(&edited.new_texts)
        .map(|((start, end), new_text)| {
            edits_import_of_symbol(edited.before, *start, *end, edited.symbol)
                || (import_edited && removes_blank_lines(edited.before, *start, *end, new_text))
        })
        .collect()
}

/// A deletion of whole blank lines, nothing else.
fn removes_blank_lines(before: &str, start: usize, end: usize, new_text: &str) -> bool {
    let removed = &before[start..end];
    new_text.is_empty()
        && !removed.is_empty()
        && removed.ends_with('\n')
        && removed.chars().all(char::is_whitespace)
        && (start == 0 || before[..start].ends_with('\n'))
}

/// An edit on the `import ...<symbol>` line of the symbol being renamed. The
/// server lists no reference for that import, yet it edits it: it rewrites the
/// name, or drops the whole line when the new name no longer needs it (a member
/// of the same name now wins over the imported extension). Dropping the line
/// shows up in the usage comparison as lost usages when it changes meaning.
fn edits_import_of_symbol(before: &str, start: usize, end: usize, symbol: &str) -> bool {
    let line_start = before[..start].rfind('\n').map_or(0, |at| at + 1);
    let line_end = before[start..]
        .find('\n')
        .map_or(before.len(), |at| start + at);
    // The line break itself may go with the line.
    let reach = (line_end + 1).min(before.len());
    if end > reach {
        return false;
    }
    let line = before[line_start..line_end].trim_end();
    if !line.trim_start().starts_with("import ") {
        return false;
    }
    // `import a.b.symbol` or `import a.b.symbol as alias`: the symbol is the
    // last path segment.
    let path = line.split(" as ").next().unwrap_or(line).trim_end();
    path.strip_suffix(symbol)
        .is_some_and(|prefix| prefix.ends_with('.'))
}

#[cfg(test)]
mod tests;
