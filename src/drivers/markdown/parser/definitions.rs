use super::shared::{
    find_matching_bracket, is_suppressed, normalize_reference_label,
    parse_reference_definition_destination,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MarkdownReferenceDefinition {
    pub normalized_label: String,
    pub href_start: usize,
    pub href_end: usize,
    pub href: String,
}

pub(super) fn parse_reference_definitions(
    content: &str,
    suppressed: &[std::ops::Range<usize>],
) -> Vec<MarkdownReferenceDefinition> {
    let bytes = content.as_bytes();
    let mut definitions = Vec::new();
    let mut line_start = 0usize;

    while line_start < bytes.len() {
        let mut line_end = line_start;
        while line_end < bytes.len() && bytes[line_end] != b'\n' {
            line_end += 1;
        }

        if is_suppressed(line_start, suppressed) {
            line_start = line_end.saturating_add(1);
            continue;
        }

        let mut cursor = line_start;
        let mut indent = 0usize;
        while indent < 3 && bytes.get(cursor) == Some(&b' ') {
            cursor += 1;
            indent += 1;
        }

        if bytes.get(cursor) == Some(&b'[') {
            if let Some(label_end) = find_matching_bracket(bytes, cursor) {
                if bytes.get(label_end + 1) == Some(&b':') {
                    let normalized_label =
                        normalize_reference_label(&content[cursor + 1..label_end]);

                    if !normalized_label.is_empty() {
                        if let Some((href_start, href_end)) =
                            parse_reference_definition_destination(bytes, label_end + 1, line_end)
                        {
                            definitions.push(MarkdownReferenceDefinition {
                                normalized_label,
                                href_start,
                                href_end,
                                href: content[href_start..href_end].to_string(),
                            });
                        }
                    }
                }
            }
        }

        line_start = line_end.saturating_add(1);
    }

    definitions
}
