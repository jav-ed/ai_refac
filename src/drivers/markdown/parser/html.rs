//! Finds the paths in raw HTML inside Markdown: the value of `href`, `src`,
//! `poster`, and the URLs of `srcset`, as byte ranges of the value itself.
//!
//! READMEs mix Markdown with `<img src="docs/logo.png">` and
//! `<a href="docs/guide.md">`; those are references to files like any other.
//! The CommonMark parser hands over HTML in pieces (a block arrives line by
//! line), so the scanner keeps its place between pieces: a tag opened on one
//! line may get its attributes on the next. Comments and the content of
//! `pre`, `code`, `script`, `style`, and `textarea` are text, not references.

use std::ops::Range;

#[cfg(test)]
mod tests;

/// Elements whose content is shown or run, never followed.
const RAW_TEXT_ELEMENTS: [&str; 5] = ["pre", "code", "script", "style", "textarea"];
const LINK_ATTRIBUTES: [&str; 3] = ["href", "src", "poster"];

#[derive(Debug, Default)]
enum State {
    #[default]
    Text,
    Comment,
    /// Inside `<name ...`, waiting for the attributes or the closing `>`.
    Tag {
        name: String,
        closing: bool,
    },
}

#[derive(Debug, Default)]
pub(super) struct HtmlScanner {
    state: State,
    /// The raw-text element whose end tag is awaited.
    inside: Option<&'static str>,
}

impl HtmlScanner {
    /// Forget a tag or comment left open by the previous piece. Called when
    /// something that is not HTML arrives, because then the tag really ended.
    pub(super) fn interrupted(&mut self) {
        self.state = State::Text;
    }

    /// Scan the HTML piece found at `base..base + text.len()` of the file and
    /// push the range of every path in it onto `found`.
    pub(super) fn scan(&mut self, text: &str, base: usize, found: &mut Vec<Range<usize>>) {
        let bytes = text.as_bytes();
        let mut at = 0;
        while at < bytes.len() {
            at = match std::mem::take(&mut self.state) {
                State::Text => self.scan_text(text, at),
                State::Comment => self.scan_comment(text, at),
                State::Tag { name, closing } => self.scan_tag(text, at, name, closing, base, found),
            };
        }
    }

    fn scan_text(&mut self, text: &str, at: usize) -> usize {
        let bytes = text.as_bytes();
        let Some(open) = text[at..].find('<').map(|offset| at + offset) else {
            return bytes.len();
        };
        if text[open..].starts_with("<!--") {
            self.state = State::Comment;
            return open + 4;
        }
        let closing = bytes.get(open + 1) == Some(&b'/');
        let name_start = open + if closing { 2 } else { 1 };
        let name_end = name_start
            + text[name_start..]
                .bytes()
                .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
                .count();
        if name_end == name_start || !bytes[name_start].is_ascii_alphabetic() {
            // A lone `<` is text.
            return open + 1;
        }
        self.state = State::Tag {
            name: text[name_start..name_end].to_ascii_lowercase(),
            closing,
        };
        name_end
    }

    fn scan_comment(&mut self, text: &str, at: usize) -> usize {
        match text[at..].find("-->") {
            Some(end) => at + end + 3,
            None => {
                self.state = State::Comment;
                text.len()
            }
        }
    }

    fn scan_tag(
        &mut self,
        text: &str,
        mut at: usize,
        name: String,
        closing: bool,
        base: usize,
        found: &mut Vec<Range<usize>>,
    ) -> usize {
        let bytes = text.as_bytes();
        loop {
            while at < bytes.len() && (bytes[at].is_ascii_whitespace() || bytes[at] == b'/') {
                at += 1;
            }
            if at >= bytes.len() {
                self.state = State::Tag { name, closing };
                return at;
            }
            if bytes[at] == b'>' {
                self.tag_ended(&name, closing);
                return at + 1;
            }

            let name_end = at
                + text[at..]
                    .bytes()
                    .take_while(|byte| !byte.is_ascii_whitespace() && !b"=>/".contains(byte))
                    .count();
            let attribute = text[at..name_end].to_ascii_lowercase();
            at = name_end;
            while at < bytes.len() && bytes[at].is_ascii_whitespace() {
                at += 1;
            }
            if bytes.get(at) != Some(&b'=') {
                continue;
            }
            at += 1;
            while at < bytes.len() && bytes[at].is_ascii_whitespace() {
                at += 1;
            }

            let Some((value, next)) = attribute_value(text, at) else {
                // The value continues in a piece that is not here yet.
                self.state = State::Tag { name, closing };
                return text.len();
            };
            at = next;
            if !closing && self.inside.is_none() {
                collect(&attribute, text, &value, base, found);
            }
        }
    }

    fn tag_ended(&mut self, name: &str, closing: bool) {
        match (self.inside, closing) {
            (Some(open), true) if open == name => self.inside = None,
            (None, false) => {
                self.inside = RAW_TEXT_ELEMENTS
                    .into_iter()
                    .find(|element| *element == name);
            }
            _ => {}
        }
    }
}

fn is_quote(byte: u8) -> bool {
    byte == b'"' || byte == b'\''
}

/// The value starting at `at`, quoted or bare (up to whitespace or `>`), and
/// where scanning goes on after it. `None` when a quote is not closed within
/// this piece or the value has not started yet.
fn attribute_value(text: &str, at: usize) -> Option<(Range<usize>, usize)> {
    let bytes = text.as_bytes();
    match bytes.get(at) {
        Some(&quote) if is_quote(quote) => {
            let end = text[at + 1..].find(quote as char)? + at + 1;
            Some((at + 1..end, end + 1))
        }
        Some(_) => {
            let end = at
                + text[at..]
                    .bytes()
                    .take_while(|byte| !byte.is_ascii_whitespace() && *byte != b'>')
                    .count();
            Some((at..end, end))
        }
        None => None,
    }
}

fn collect(
    attribute: &str,
    text: &str,
    value: &Range<usize>,
    base: usize,
    found: &mut Vec<Range<usize>>,
) {
    if LINK_ATTRIBUTES.contains(&attribute) {
        if !value.is_empty() {
            found.push(base + value.start..base + value.end);
        }
    } else if attribute == "srcset" {
        // `a.png 1x, b.png 2x`: every candidate starts with its URL.
        let mut offset = value.start;
        for candidate in text[value.clone()].split(',') {
            let leading = candidate.len() - candidate.trim_start().len();
            let url = candidate.split_whitespace().next().unwrap_or("");
            if !url.is_empty() {
                let start = offset + leading;
                found.push(base + start..base + start + url.len());
            }
            offset += candidate.len() + 1;
        }
    }
}
