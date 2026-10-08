//! A second, independent reader of Markdown links for the move tests.
//!
//! It is deliberately simple and knows only the shapes the fixtures use
//! (`](x)`, `](<x>)`, `src="x"`, `href="x"`, `[ref]: x`), so a test can check
//! the property that matters: after any move, every Markdown file still links
//! to the same files as before, wherever they went.

use super::project::Tree;
use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

/// `(markdown file, file or folder it links to)`, for links whose target exists.
pub type Edges = BTreeSet<(String, String)>;

pub fn edges(tree: &Tree) -> Edges {
    let mut edges = Edges::new();
    for (file, bytes) in tree {
        if !is_markdown(file) {
            continue;
        }
        let text = String::from_utf8_lossy(bytes);
        for raw in raw_links(&text) {
            let Some(target) = resolve(file, &raw) else {
                continue;
            };
            let folder = format!("{target}/");
            if tree.contains_key(&target) || tree.keys().any(|path| path.starts_with(&folder)) {
                edges.insert((file.clone(), target));
            }
        }
    }
    edges
}

/// `edges` as they must read after `place` says where each path went.
pub fn moved_edges(edges: &Edges, place: &dyn Fn(&str) -> String) -> Edges {
    edges
        .iter()
        .map(|(file, target)| (place(file), place(target)))
        .collect()
}

pub fn is_markdown(path: &str) -> bool {
    [".md", ".markdown", ".mdx"]
        .iter()
        .any(|extension| path.ends_with(extension))
}

fn raw_links(text: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find("](") {
        rest = &rest[at + 2..];
        let end = rest.find(')').unwrap_or(rest.len());
        let inner = rest[..end].trim_start_matches('<');
        let inner = inner.split(['>', ' ']).next().unwrap_or("");
        links.push(inner.to_string());
    }
    for attribute in ["src=\"", "href=\"", "src='"] {
        let mut rest = text;
        while let Some(at) = rest.find(attribute) {
            rest = &rest[at + attribute.len()..];
            let end = rest.find(['"', '\'']).unwrap_or(rest.len());
            links.push(rest[..end].to_string());
        }
    }
    for line in text.lines() {
        if let Some((label, destination)) = line.split_once("]: ")
            && label.starts_with('[')
        {
            links.push(
                destination
                    .trim()
                    .trim_matches('<')
                    .split(" \"")
                    .next()
                    .unwrap_or("")
                    .to_string(),
            );
        }
    }
    links
}

/// The project-relative path a link written in `file` points at, or `None`
/// for anything that is not a relative file path.
fn resolve(file: &str, raw: &str) -> Option<String> {
    let raw = raw.split(['#', '?']).next()?;
    if raw.is_empty() || raw.starts_with('/') || raw.contains("://") || raw.starts_with("mailto:") {
        return None;
    }
    let decoded = raw.replace("%20", " ");
    let joined = Path::new(file).parent()?.join(decoded);
    let mut normalized = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(segment) => normalized.push(segment),
            _ => {}
        }
    }
    Some(normalized.to_string_lossy().into_owned())
}
