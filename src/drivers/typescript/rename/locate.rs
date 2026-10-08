use super::edits::LineIndex;
use crate::drivers::symbol::scan;
use anyhow::Result;

pub use crate::drivers::symbol::scan::Occurrence;

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
    scan::occurrences(text, symbol, line, column, is_identifier_char, |offset| {
        index.position(offset)
    })
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
