//! Where the bodies of `macro_rules!` definitions are. rust-analyzer renames
//! through macro *calls* but never inside a definition, so a name that the
//! macro body writes stays behind and breaks the build.

use crate::drivers::lsp_rename::language::UnrenamedPlace;

const WHAT: &str = "a macro_rules! definition";

/// The body (between the outer delimiters) of every `macro_rules!` in `text`.
pub fn bodies(text: &str) -> Vec<UnrenamedPlace> {
    let bytes = text.as_bytes();
    let mut places = Vec::new();
    let mut from = 0;
    while let Some(found) = text[from..].find("macro_rules!") {
        let after_keyword = from + found + "macro_rules!".len();
        from = after_keyword;
        let Some(open) = text[after_keyword..]
            .find(['{', '(', '['])
            .map(|at| after_keyword + at)
        else {
            break;
        };
        // Only a name sits between the keyword and the opening delimiter.
        let name = &text[after_keyword..open];
        if !name.trim().chars().all(|c| c == '_' || c.is_alphanumeric()) {
            continue;
        }
        let Some(close) = matching_close(bytes, open) else {
            continue;
        };
        places.push(UnrenamedPlace {
            start: open + 1,
            end: close,
            what: WHAT,
        });
        from = close;
    }
    places
}

/// The index of the delimiter that closes the one at `open`, skipping
/// strings, character literals and comments.
fn matching_close(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut at = open;
    while at < bytes.len() {
        match bytes[at] {
            b'{' | b'(' | b'[' => depth += 1,
            b'}' | b')' | b']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(at);
                }
            }
            b'"' => at = end_of_string(bytes, at),
            b'\'' => at = end_of_char(bytes, at),
            b'/' if bytes.get(at + 1) == Some(&b'/') => {
                at = bytes[at..]
                    .iter()
                    .position(|byte| *byte == b'\n')
                    .map_or(bytes.len(), |end| at + end);
            }
            b'/' if bytes.get(at + 1) == Some(&b'*') => {
                at = find(bytes, at + 2, b"*/").map_or(bytes.len(), |end| end + 1);
            }
            _ => {}
        }
        at += 1;
    }
    None
}

/// The index of the closing quote of the string starting at `open`.
fn end_of_string(bytes: &[u8], open: usize) -> usize {
    let mut at = open + 1;
    while at < bytes.len() {
        match bytes[at] {
            b'\\' => at += 1,
            b'"' => return at,
            _ => {}
        }
        at += 1;
    }
    bytes.len()
}

/// A character literal (`'x'`, `'\n'`) is skipped; a lifetime (`'a`) is not.
fn end_of_char(bytes: &[u8], open: usize) -> usize {
    match (
        bytes.get(open + 1),
        bytes.get(open + 2),
        bytes.get(open + 3),
    ) {
        (Some(b'\\'), _, _) => bytes[open + 2..]
            .iter()
            .position(|byte| *byte == b'\'')
            .map_or(open, |end| open + 2 + end),
        (Some(_), Some(b'\''), _) => open + 2,
        _ => open,
    }
}

fn find(bytes: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    bytes[from..]
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|at| from + at)
}

#[cfg(test)]
mod tests;
