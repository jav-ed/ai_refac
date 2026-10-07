//! Rewriting class names in Android XML (manifest, layouts, navigation
//! graphs, preferences) without parsing it into a tree: only the changed
//! names are replaced in place, so formatting, comments and line endings
//! survive. The scanner finds element names and attribute values; a malformed
//! file is an error because a half-read file cannot be rewritten safely.

use anyhow::{Result, bail};

/// Attributes whose value may be a class name relative to the namespace
/// (`.ui.MainActivity`, or `MainActivity` in the manifest).
const RELATIVE_ATTRIBUTES: &[&str] = &[
    "android:name",
    "tools:context",
    "android:targetActivity",
    "android:parentActivityName",
    "android:fragment",
    "app:fragment",
    "class",
    "android:backupAgent",
    "android:manageSpaceActivity",
];

pub struct Names<'a> {
    pub namespace: &'a str,
    /// (old fully qualified name, new fully qualified name)
    pub renames: &'a [(String, String)],
    /// Only the manifest reads a bare `MainActivity` as `<namespace>.MainActivity`.
    pub is_manifest: bool,
}

enum Spot<'a> {
    Element,
    Attribute(&'a str),
}

struct Token<'a> {
    start: usize,
    end: usize,
    spot: Spot<'a>,
}

/// The text with every renamed class replaced, `None` when nothing changed.
pub fn rewrite(text: &str, names: &Names) -> Result<Option<String>> {
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    for token in scan(text)? {
        let value = &text[token.start..token.end];
        let replacement = match token.spot {
            Spot::Element => full_rename(value, names).map(str::to_string),
            Spot::Attribute(attribute) => attribute_rename(value, attribute, names),
        };
        if let Some(replacement) = replacement {
            edits.push((token.start, token.end, replacement));
        }
    }
    if edits.is_empty() {
        return Ok(None);
    }
    let mut updated = text.to_string();
    for (start, end, replacement) in edits.into_iter().rev() {
        updated.replace_range(start..end, &replacement);
    }
    Ok(Some(updated))
}

fn full_rename<'a>(value: &str, names: &'a Names) -> Option<&'a str> {
    names
        .renames
        .iter()
        .find(|(old, _)| old == value)
        .map(|(_, new)| new.as_str())
}

fn attribute_rename(value: &str, attribute: &str, names: &Names) -> Option<String> {
    if let Some(new) = full_rename(value, names) {
        return Some(new.to_string());
    }
    if !RELATIVE_ATTRIBUTES.contains(&attribute) {
        return None;
    }
    let bare = names.is_manifest && !value.contains('.') && !value.is_empty();
    if !value.starts_with('.') && !bare {
        return None;
    }
    let full = if bare {
        format!("{}.{value}", names.namespace)
    } else {
        format!("{}{value}", names.namespace)
    };
    let new = full_rename(&full, names)?;
    // Keep the file's style while the class stays under the namespace.
    let Some(rest) = new.strip_prefix(&format!("{}.", names.namespace)) else {
        return Some(new.to_string());
    };
    if bare && !rest.contains('.') {
        Some(rest.to_string())
    } else {
        Some(format!(".{rest}"))
    }
}

fn scan(text: &str) -> Result<Vec<Token<'_>>> {
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'<' {
            i += 1;
            continue;
        }
        let rest = &text[i..];
        let skip_to = if rest.starts_with("<!--") {
            Some("-->")
        } else if rest.starts_with("<![CDATA[") {
            Some("]]>")
        } else if rest.starts_with("<?") {
            Some("?>")
        } else if rest.starts_with("<!") {
            Some(">")
        } else {
            None
        };
        if let Some(end_marker) = skip_to {
            let Some(found) = rest.find(end_marker) else {
                bail!("Unterminated `{}` at byte {i}", &rest[..2]);
            };
            i += found + end_marker.len();
            continue;
        }
        i = scan_element(text, i, &mut tokens)?;
    }
    Ok(tokens)
}

/// Scan one start or end tag beginning at `<`; returns the index after it.
fn scan_element<'a>(text: &'a str, open: usize, tokens: &mut Vec<Token<'a>>) -> Result<usize> {
    let bytes = text.as_bytes();
    let mut j = open + 1;
    if bytes.get(j) == Some(&b'/') {
        j += 1;
    }
    let name_start = j;
    while j < bytes.len() && !is_name_end(bytes[j]) {
        j += 1;
    }
    tokens.push(Token {
        start: name_start,
        end: j,
        spot: Spot::Element,
    });
    loop {
        skip_whitespace(bytes, &mut j);
        match bytes.get(j) {
            None => bail!("Unterminated tag at byte {open}"),
            Some(b'>') => return Ok(j + 1),
            Some(b'/') => {
                j += 1;
                continue;
            }
            Some(_) => {}
        }
        let attribute_start = j;
        while j < bytes.len() && !is_name_end(bytes[j]) && bytes[j] != b'=' {
            j += 1;
        }
        let attribute = &text[attribute_start..j];
        skip_whitespace(bytes, &mut j);
        if bytes.get(j) != Some(&b'=') {
            bail!("Attribute `{attribute}` has no value (byte {j})");
        }
        j += 1;
        skip_whitespace(bytes, &mut j);
        let Some(&quote) = bytes
            .get(j)
            .filter(|byte| **byte == b'"' || **byte == b'\'')
        else {
            bail!("Attribute `{attribute}` is not quoted (byte {j})");
        };
        let value_start = j + 1;
        let Some(length) = text[value_start..].find(quote as char) else {
            bail!("Unterminated value of `{attribute}` (byte {j})");
        };
        tokens.push(Token {
            start: value_start,
            end: value_start + length,
            spot: Spot::Attribute(attribute),
        });
        j = value_start + length + 1;
    }
}

fn skip_whitespace(bytes: &[u8], j: &mut usize) {
    while *j < bytes.len() && bytes[*j].is_ascii_whitespace() {
        *j += 1;
    }
}

fn is_name_end(byte: u8) -> bool {
    byte.is_ascii_whitespace() || byte == b'>' || byte == b'/'
}

#[cfg(test)]
mod tests;
