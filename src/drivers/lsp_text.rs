//! Applying LSP text edits to document content. Positions count UTF-16 code
//! units, the protocol default, so edits from servers that do not negotiate
//! UTF-8 land on the right characters.

use anyhow::Result;
use lsp_types::{Position, TextEdit};

pub fn apply_text_edits(content: &str, edits: Vec<TextEdit>) -> Result<String> {
    let line_offsets = line_start_offsets(content);
    let mut edits_with_offsets = Vec::with_capacity(edits.len());

    for edit in edits {
        let start = position_to_byte_offset(content, &line_offsets, edit.range.start)?;
        let end = position_to_byte_offset(content, &line_offsets, edit.range.end)?;

        if start > end {
            anyhow::bail!(
                "Invalid edit range: start {:?} is after end {:?}",
                start,
                end
            );
        }

        edits_with_offsets.push((start, end, edit.new_text));
    }

    edits_with_offsets.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));

    let mut updated = content.to_string();
    let mut previous_start = None;
    for (start, end, new_text) in edits_with_offsets {
        // Overlapping edits are invalid in LSP; applying them would corrupt the text.
        if previous_start.is_some_and(|previous| end > previous) {
            anyhow::bail!("Overlapping text edits around byte {start}");
        }
        updated.replace_range(start..end, &new_text);
        previous_start = Some(start);
    }

    Ok(updated)
}

/// Converts between byte offsets and LSP positions of one document. Columns
/// count UTF-16 code units, the protocol default.
pub struct TextIndex<'a> {
    content: &'a str,
    line_offsets: Vec<usize>,
}

impl<'a> TextIndex<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            content,
            line_offsets: line_start_offsets(content),
        }
    }

    pub fn offset(&self, position: Position) -> Result<usize> {
        position_to_byte_offset(self.content, &self.line_offsets, position)
    }

    pub fn position(&self, offset: usize) -> Position {
        let line = self.line_offsets.partition_point(|start| *start <= offset) - 1;
        let column: usize = self.content[self.line_offsets[line]..offset]
            .chars()
            .map(char::len_utf16)
            .sum();
        Position::new(line as u32, column as u32)
    }
}

fn line_start_offsets(content: &str) -> Vec<usize> {
    let mut offsets = vec![0];
    for (index, byte) in content.bytes().enumerate() {
        if byte == b'\n' {
            offsets.push(index + 1);
        }
    }
    offsets
}

fn position_to_byte_offset(
    content: &str,
    line_offsets: &[usize],
    position: Position,
) -> Result<usize> {
    let line = position.line as usize;
    if line >= line_offsets.len() {
        anyhow::bail!(
            "LSP edit line {} is out of bounds for document with {} lines",
            line,
            line_offsets.len()
        );
    }

    let line_start = line_offsets[line];
    let next_line_start = line_offsets.get(line + 1).copied().unwrap_or(content.len());
    let line_end = trim_line_ending(content, line_start, next_line_start);
    let line_text = &content[line_start..line_end];
    let target_character = position.character as usize;

    if target_character == 0 {
        return Ok(line_start);
    }

    let mut utf16_offset = 0;
    for (byte_index, ch) in line_text.char_indices() {
        if utf16_offset >= target_character {
            return Ok(line_start + byte_index);
        }

        let next_utf16_offset = utf16_offset + ch.len_utf16();
        if next_utf16_offset > target_character {
            return Ok(line_start + byte_index);
        }

        utf16_offset = next_utf16_offset;
    }

    Ok(line_end)
}

fn trim_line_ending(content: &str, line_start: usize, next_line_start: usize) -> usize {
    let mut end = next_line_start;

    if end > line_start && content.as_bytes()[end - 1] == b'\n' {
        end -= 1;
    }

    if end > line_start && content.as_bytes()[end - 1] == b'\r' {
        end -= 1;
    }

    end
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::Range;

    #[test]
    fn test_apply_text_edits_rewrites_ascii_content() -> Result<()> {
        let content = "mod a;\nfn main() { a::value(); }\n";
        let edits = vec![
            TextEdit {
                range: Range {
                    start: Position::new(0, 4),
                    end: Position::new(0, 5),
                },
                new_text: "b".to_string(),
            },
            TextEdit {
                range: Range {
                    start: Position::new(1, 12),
                    end: Position::new(1, 13),
                },
                new_text: "b".to_string(),
            },
        ];

        let updated = apply_text_edits(content, edits)?;
        assert_eq!(updated, "mod b;\nfn main() { b::value(); }\n");
        Ok(())
    }

    #[test]
    fn test_apply_text_edits_handles_utf16_positions() -> Result<()> {
        let content = "🙂value\n";
        let edits = vec![TextEdit {
            range: Range {
                start: Position::new(0, 2),
                end: Position::new(0, 7),
            },
            new_text: "name".to_string(),
        }];

        let updated = apply_text_edits(content, edits)?;
        assert_eq!(updated, "🙂name\n");
        Ok(())
    }

    #[test]
    fn text_index_round_trips_utf16_positions() {
        let content = "ab\n🙂value\n";
        let index = TextIndex::new(content);
        let offset = index.offset(Position::new(1, 2)).unwrap();
        assert_eq!(&content[offset..], "value\n");
        assert_eq!(index.position(offset), Position::new(1, 2));
        assert_eq!(index.position(0), Position::new(0, 0));
        assert!(index.offset(Position::new(9, 0)).is_err());
    }

    #[test]
    fn test_apply_text_edits_rejects_overlapping_edits() {
        let edit = |start: u32, end: u32| TextEdit {
            range: Range {
                start: Position::new(0, start),
                end: Position::new(0, end),
            },
            new_text: "x".to_string(),
        };
        let error = apply_text_edits("abcdef", vec![edit(1, 4), edit(3, 5)]).unwrap_err();
        assert!(error.to_string().contains("Overlapping"), "{error}");
        assert!(apply_text_edits("abcdef", vec![edit(1, 3), edit(3, 5)]).is_ok());
    }
}
