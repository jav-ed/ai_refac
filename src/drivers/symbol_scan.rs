//! Finding where a symbol name appears in a file, the first step of a rename:
//! the language server decides which matches are really that symbol; this scan
//! only supplies candidate positions.

use anyhow::{Result, bail};

/// More textual matches than this need an explicit `--line`: guessing among
/// hundreds of candidates would hide which symbol is renamed.
const MAX_OCCURRENCES: usize = 200;

/// A whole-word textual match of the symbol name. Matches inside strings and
/// comments are rejected later by `prepareRename`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Occurrence {
    pub offset: usize,
    /// Zero-based line.
    pub line: u32,
    /// Zero-based byte column within the line.
    pub column: u32,
}

/// Zero-based line and byte column of an offset, counting only `\n` as a line
/// break.
pub fn line_column(text: &str) -> impl Fn(usize) -> (u32, u32) + '_ {
    let mut starts = vec![0];
    starts.extend(text.match_indices('\n').map(|(index, _)| index + 1));
    move |offset| {
        let line = starts.partition_point(|start| *start <= offset) - 1;
        (line as u32, (offset - starts[line]) as u32)
    }
}

/// `line` and `column` are 1-based; the column counts bytes like `rg --column`.
/// `position` maps an offset to its zero-based line and byte column.
pub fn occurrences(
    text: &str,
    symbol: &str,
    line: Option<u32>,
    column: Option<u32>,
    is_identifier_char: fn(char) -> bool,
    position: impl Fn(usize) -> (u32, u32),
) -> Result<Vec<Occurrence>> {
    let mut found = Vec::new();
    for (offset, _) in text.match_indices(symbol) {
        let before = text[..offset].chars().next_back();
        let after = text[offset + symbol.len()..].chars().next();
        if before.is_some_and(is_identifier_char) || after.is_some_and(is_identifier_char) {
            continue;
        }
        let (zero_line, zero_column) = position(offset);
        if line.is_some_and(|wanted| wanted != zero_line + 1)
            || column.is_some_and(|wanted| wanted != zero_column + 1)
        {
            continue;
        }
        found.push(Occurrence {
            offset,
            line: zero_line,
            column: zero_column,
        });
    }
    if found.is_empty() {
        let place = match (line, column) {
            (Some(line), Some(column)) => format!(" at {line}:{column}"),
            (Some(line), None) => format!(" on line {line}"),
            _ => String::new(),
        };
        bail!("`{symbol}` does not appear as an identifier{place} in the file");
    }
    if found.len() > MAX_OCCURRENCES && line.is_none() {
        bail!(
            "`{symbol}` appears {} times in the file; pass --line to choose the declaration or usage to rename",
            found.len()
        );
    }
    Ok(found)
}
