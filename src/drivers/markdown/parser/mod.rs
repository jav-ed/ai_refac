//! Finds the destination of every link, image, and reference definition in a
//! Markdown file, as the byte range of the destination text itself (inside
//! `<...>` when the author wrote angle brackets), so the rewrite can replace
//! exactly that text and nothing else.
//!
//! Which constructs are links is decided by a CommonMark parser, not by this
//! file: code blocks (fenced and indented), code spans, HTML comments, and
//! front matter never produce a link, and a reference definition is found
//! wherever CommonMark finds one (a definition may continue on the next line).
//! The parser reports where a whole link or definition sits; `destination.rs`
//! narrows that to the destination. Raw HTML is not Markdown, so `html.rs`
//! reads the `href`, `src`, and `srcset` values out of it.

mod destination;
mod html;
#[cfg(test)]
mod tests;

use anyhow::{Result, bail};
use html::HtmlScanner;
use pulldown_cmark::{Event, LinkType, Options, Parser, Tag, TagEnd};
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarkdownLinkTarget {
    pub href_start: usize,
    pub href_end: usize,
    pub href: String,
}

/// A link or image whose text the parser is still inside.
struct OpenLink {
    /// Only inline links carry their destination at the link itself. Reference
    /// links (full, collapsed, shortcut) are rewritten at their definition.
    inline: bool,
    /// End of everything seen inside the link text so far.
    text_end: usize,
    /// The destination as the parser read it, to check the range against.
    destination: String,
}

/// Every destination in the file, in file order.
pub(crate) fn parse_markdown_links(content: &str) -> Result<Vec<MarkdownLinkTarget>> {
    let parser = Parser::new_ext(content, options());
    let mut definitions: Vec<(Range<usize>, String)> = parser
        .reference_definitions()
        .iter()
        .map(|(_, definition)| (definition.span.clone(), definition.dest.to_string()))
        .collect();
    definitions.sort_by_key(|(span, _)| span.start);

    let mut destinations = Vec::new();
    for (span, expected) in definitions {
        destinations.extend(destination::of_definition(content, span, &expected)?);
    }

    let mut open: Vec<OpenLink> = Vec::new();
    let mut html = HtmlScanner::default();
    let mut html_destinations = Vec::new();
    for (event, range) in parser.into_offset_iter() {
        match event {
            Event::Start(
                Tag::Link {
                    link_type,
                    dest_url,
                    ..
                }
                | Tag::Image {
                    link_type,
                    dest_url,
                    ..
                },
            ) => {
                if let Some(parent) = open.last_mut() {
                    parent.text_end = parent.text_end.max(range.end);
                }
                // The text starts after `[` or `![`; an empty text ends there.
                let opener = if content[range.start..].starts_with('!') {
                    2
                } else {
                    1
                };
                open.push(OpenLink {
                    inline: link_type == LinkType::Inline,
                    text_end: range.start + opener,
                    destination: dest_url.to_string(),
                });
            }
            Event::End(TagEnd::Link | TagEnd::Image) => {
                let Some(link) = open.pop() else {
                    bail!("The Markdown parser closed a link it never opened");
                };
                if link.inline {
                    destinations.extend(destination::of_inline_link(
                        content,
                        link.text_end,
                        &link.destination,
                    )?);
                }
                if let Some(parent) = open.last_mut() {
                    parent.text_end = parent.text_end.max(range.end);
                }
            }
            Event::Html(_) | Event::InlineHtml(_) => {
                if let Some(link) = open.last_mut() {
                    link.text_end = link.text_end.max(range.end);
                }
                html.scan(&content[range.clone()], range.start, &mut html_destinations);
            }
            _ => {
                html.interrupted();
                if let Some(link) = open.last_mut() {
                    link.text_end = link.text_end.max(range.end);
                }
            }
        }
    }
    destinations.extend(html_destinations);

    destinations.sort_by_key(|range| range.start);
    Ok(destinations
        .into_iter()
        .map(|range| MarkdownLinkTarget {
            href: content[range.clone()].to_string(),
            href_start: range.start,
            href_end: range.end,
        })
        .collect())
}

fn options() -> Options {
    // Tables, footnotes, and front matter change where links and definitions
    // are: a footnote definition `[^1]: text` is not a link definition, and a
    // link in YAML front matter is not a link. Everything else (math, wiki
    // links, smart punctuation, ...) would only reinterpret text refac leaves
    // alone.
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
        | Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS
}
