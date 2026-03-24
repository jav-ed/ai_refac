use std::collections::HashMap;

use super::shared::{find_matching_bracket, normalize_reference_label, parse_inline_destination};
use super::{
    MarkdownLinkTarget, MarkdownLinkTargetKind, MarkdownReferenceUsage, MarkdownReferenceUsageKind,
};

pub(super) fn parse_inline_links_and_reference_usages(
    content: &str,
    definition_counts: &HashMap<String, usize>,
    targets: &mut Vec<MarkdownLinkTarget>,
) -> Vec<MarkdownReferenceUsage> {
    let bytes = content.as_bytes();
    let mut reference_usages = Vec::new();
    let mut index = 0usize;

    while index < bytes.len() {
        let (label_start, next_index, is_image) = match bytes[index] {
            b'[' => (index, index + 1, false),
            b'!' if bytes.get(index + 1) == Some(&b'[') => (index + 1, index + 2, true),
            _ => {
                index += 1;
                continue;
            }
        };

        let Some(label_end) = find_matching_bracket(bytes, label_start) else {
            index = next_index;
            continue;
        };

        if bytes.get(label_end + 1) == Some(&b'(') {
            let Some((href_start, href_end, closing_paren)) =
                parse_inline_destination(bytes, label_end + 1)
            else {
                index = label_end + 1;
                continue;
            };

            targets.push(MarkdownLinkTarget {
                kind: MarkdownLinkTargetKind::Inline,
                href_start,
                href_end,
                href: content[href_start..href_end].to_string(),
            });

            index = closing_paren + 1;
            continue;
        }

        if !is_image && bytes.get(label_end + 1) == Some(&b'[') {
            let reference_start = label_end + 1;
            let Some(reference_end) = find_matching_bracket(bytes, reference_start) else {
                index = label_end + 1;
                continue;
            };

            let raw_reference = &content[reference_start + 1..reference_end];
            let normalized_label = if raw_reference.is_empty() {
                normalize_reference_label(&content[label_start + 1..label_end])
            } else {
                normalize_reference_label(raw_reference)
            };

            let kind = if raw_reference.is_empty() {
                MarkdownReferenceUsageKind::Collapsed
            } else {
                MarkdownReferenceUsageKind::Full
            };

            if !normalized_label.is_empty() {
                reference_usages.push(MarkdownReferenceUsage {
                    kind,
                    normalized_label,
                });
            }

            index = reference_end + 1;
            continue;
        }

        let shortcut_label = normalize_reference_label(&content[label_start + 1..label_end]);
        if !is_image
            && bytes.get(label_end + 1) != Some(&b':')
            && definition_counts.get(&shortcut_label) == Some(&1usize)
        {
            reference_usages.push(MarkdownReferenceUsage {
                kind: MarkdownReferenceUsageKind::Shortcut,
                normalized_label: shortcut_label,
            });
        }

        index = label_end + 1;
    }

    reference_usages
}
