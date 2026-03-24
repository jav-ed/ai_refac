use std::collections::HashMap;

mod definitions;
mod inline;
mod shared;

use definitions::parse_reference_definitions;
use inline::parse_inline_links_and_reference_usages;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MarkdownLinkTargetKind {
    Inline,
    ReferenceDefinition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarkdownLinkTarget {
    pub kind: MarkdownLinkTargetKind,
    pub href_start: usize,
    pub href_end: usize,
    pub href: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MarkdownReferenceUsageKind {
    Full,
    Collapsed,
    Shortcut,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarkdownReferenceUsage {
    pub kind: MarkdownReferenceUsageKind,
    pub normalized_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedMarkdownLinks {
    pub targets: Vec<MarkdownLinkTarget>,
    pub reference_usages: Vec<MarkdownReferenceUsage>,
}

pub(crate) fn parse_markdown_links(content: &str) -> ParsedMarkdownLinks {
    let reference_definitions = parse_reference_definitions(content);
    let mut definition_counts = HashMap::new();
    let mut targets = Vec::with_capacity(reference_definitions.len());

    for definition in reference_definitions {
        *definition_counts
            .entry(definition.normalized_label.clone())
            .or_insert(0usize) += 1;

        targets.push(MarkdownLinkTarget {
            kind: MarkdownLinkTargetKind::ReferenceDefinition,
            href_start: definition.href_start,
            href_end: definition.href_end,
            href: definition.href,
        });
    }

    let reference_usages =
        parse_inline_links_and_reference_usages(content, &definition_counts, &mut targets);

    ParsedMarkdownLinks {
        targets,
        reference_usages,
    }
}

#[cfg(test)]
mod tests {
    use super::{MarkdownLinkTargetKind, MarkdownReferenceUsageKind, parse_markdown_links};

    #[test]
    fn parses_inline_links_and_images() {
        let content = "See [Guide](./guide.md) and ![Diagram](./diagram.png).";
        let parsed = parse_markdown_links(content);

        assert_eq!(parsed.targets.len(), 2);
        assert_eq!(parsed.targets[0].kind, MarkdownLinkTargetKind::Inline);
        assert_eq!(parsed.targets[0].href, "./guide.md");
        assert_eq!(parsed.targets[1].href, "./diagram.png");
    }

    #[test]
    fn preserves_anchor_fragments_in_href_slice() {
        let content = "[Deep Dive](../docs/guide.md#details)";
        let parsed = parse_markdown_links(content);

        assert_eq!(parsed.targets.len(), 1);
        assert_eq!(parsed.targets[0].href, "../docs/guide.md#details");
    }

    #[test]
    fn parses_reference_definitions_and_usages() {
        let content = "\
See [Guide][guide ref], [overview][], and [shortcut].
\n\
[guide ref]: ./guide.md#deep \"Guide Title\"
\n\
[overview]: <../overview.md>
\n\
[shortcut]: ./shortcut.md
";
        let parsed = parse_markdown_links(content);

        assert_eq!(parsed.targets.len(), 3);
        assert_eq!(
            parsed
                .targets
                .iter()
                .filter(|target| target.kind == MarkdownLinkTargetKind::ReferenceDefinition)
                .count(),
            3
        );
        assert_eq!(parsed.targets[0].href, "./guide.md#deep");
        assert_eq!(parsed.targets[1].href, "../overview.md");
        assert_eq!(parsed.targets[2].href, "./shortcut.md");

        assert_eq!(parsed.reference_usages.len(), 3);
        assert_eq!(
            parsed.reference_usages[0].kind,
            MarkdownReferenceUsageKind::Full
        );
        assert_eq!(parsed.reference_usages[0].normalized_label, "guide ref");
        assert_eq!(
            parsed.reference_usages[1].kind,
            MarkdownReferenceUsageKind::Collapsed
        );
        assert_eq!(parsed.reference_usages[1].normalized_label, "overview");
        assert_eq!(
            parsed.reference_usages[2].kind,
            MarkdownReferenceUsageKind::Shortcut
        );
        assert_eq!(parsed.reference_usages[2].normalized_label, "shortcut");
    }

    #[test]
    fn skips_ambiguous_shortcut_references() {
        let content = "\
See [Guide].
\n\
[guide]: ./one.md
\n\
[ Guide ]: ./two.md
";
        let parsed = parse_markdown_links(content);

        assert!(parsed.reference_usages.is_empty());
    }
}
