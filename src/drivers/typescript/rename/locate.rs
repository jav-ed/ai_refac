use super::edits::LineIndex;
use anyhow::{Result, bail};

/// More textual matches than this need an explicit `--line`: guessing among
/// hundreds of candidates would hide which symbol is renamed.
const MAX_OCCURRENCES: usize = 200;

/// A whole-word textual match of the symbol name. The engine, not this scan,
/// decides whether it is a renameable identifier: matches inside strings and
/// comments are rejected later by `prepareRename`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Occurrence {
    pub offset: usize,
    /// Zero-based line.
    pub line: u32,
    /// Zero-based byte column within the line.
    pub column: u32,
}

pub fn is_identifier_char(character: char) -> bool {
    matches!(character, '$' | '_' | '\u{200c}' | '\u{200d}') || character.is_alphanumeric()
}

/// `line` and `column` are 1-based; the column counts bytes like `rg --column`.
pub fn occurrences(
    text: &str,
    symbol: &str,
    line: Option<u32>,
    column: Option<u32>,
) -> Result<Vec<Occurrence>> {
    let index = LineIndex::new(text);
    let mut found = Vec::new();
    for (offset, _) in text.match_indices(symbol) {
        let before = text[..offset].chars().next_back();
        let after = text[offset + symbol.len()..].chars().next();
        if before.is_some_and(is_identifier_char) || after.is_some_and(is_identifier_char) {
            continue;
        }
        let (zero_line, zero_column) = index.position(offset);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_whole_words_only_with_one_based_filters() {
        let text = "const total = totals + subtotal;\nlet x = total$ + $total + total;\n";
        let all = occurrences(text, "total", None, None).unwrap();
        assert_eq!(
            all.iter().map(|o| (o.line, o.column)).collect::<Vec<_>>(),
            [(0, 6), (1, 26)]
        );
        let second_line = occurrences(text, "total", Some(2), None).unwrap();
        assert_eq!(second_line.len(), 1);
        let exact = occurrences(text, "total", Some(1), Some(7)).unwrap();
        assert_eq!(exact[0].offset, 6);
        assert!(occurrences(text, "total", Some(1), Some(8)).is_err());
        assert!(occurrences(text, "missing", None, None).is_err());
    }

    #[test]
    fn unicode_neighbours_are_part_of_the_identifier() {
        assert!(occurrences("const étotal = 1;", "total", None, None).is_err());
    }
}
