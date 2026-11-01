//! Whether a mention of `import.meta.glob` or `Astro.glob` is a call.
//!
//! The scan reads every page and stylesheet too, because a page may import a
//! picture. A docs folder that explains these functions (`import.meta.glob(...)`
//! in prose or in a fenced example) would otherwise count as a call and start
//! the TypeScript engine for every Markdown move.

/// Whether the glob function written at `start..end` of a file with
/// `extension` is called there. Plain Markdown and stylesheets cannot hold a
/// call. In `.mdx`, fenced blocks and inline code are examples, not code. Code
/// files and `.astro` count when the name is followed by `(` or by type
/// arguments (`<`).
pub(super) fn is_call(text: &[u8], start: usize, end: usize, extension: &str) -> bool {
    match extension {
        "md" | "css" => false,
        "mdx" => !in_fence(text, start) && !in_code_span(text, start) && has_call_shape(text, end),
        _ => has_call_shape(text, end),
    }
}

fn has_call_shape(text: &[u8], end: usize) -> bool {
    let next = text[end..].iter().find(|byte| !byte.is_ascii_whitespace());
    matches!(next, Some(b'(' | b'<'))
}

/// Whether `pos` is inside a fenced code block (three or more backticks or
/// tildes, closed by a fence of the same character that is at least as long).
fn in_fence(text: &[u8], pos: usize) -> bool {
    let mut open: Option<(u8, usize)> = None;
    let mut line_start = 0;
    while line_start <= pos {
        let line_end = text[line_start..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(text.len(), |offset| line_start + offset);
        if let Some((fence, length, rest)) = fence_of(&text[line_start..line_end]) {
            match open {
                // A backtick fence cannot have a backtick in its info string:
                // that line is an inline code span.
                None if fence == b'~' || !rest.contains(&b'`') => open = Some((fence, length)),
                Some((opening, minimum))
                    if fence == opening && length >= minimum && rest.trim_ascii().is_empty() =>
                {
                    open = None
                }
                _ => {}
            }
        }
        line_start = line_end + 1;
    }
    open.is_some()
}

/// The fence character, its length and the rest of the line, when `line`
/// starts with a run of at least three backticks or tildes.
fn fence_of(line: &[u8]) -> Option<(u8, usize, &[u8])> {
    let line = line.trim_ascii_start();
    let fence = *line.first().filter(|byte| matches!(byte, b'`' | b'~'))?;
    let length = line.iter().take_while(|byte| **byte == fence).count();
    (length >= 3).then(|| (fence, length, &line[length..]))
}

/// Whether `pos` is between two backtick runs of the same length on its line.
/// A backtick that never closes is literal text and hides nothing.
fn in_code_span(text: &[u8], pos: usize) -> bool {
    let line_start = text[..pos]
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |index| index + 1);
    let line_end = text[pos..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(text.len(), |offset| pos + offset);
    let line = &text[line_start..line_end];
    let at = pos - line_start;

    let mut open: Option<(usize, usize)> = None;
    let mut index = 0;
    while index < line.len() {
        if line[index] != b'`' {
            index += 1;
            continue;
        }
        let length = line[index..]
            .iter()
            .take_while(|byte| **byte == b'`')
            .count();
        match open {
            None => open = Some((index, length)),
            Some((opened_at, opened_length)) if opened_length == length => {
                if opened_at < at && at < index {
                    return true;
                }
                open = None;
            }
            Some(_) => {}
        }
        index += length;
    }
    false
}
