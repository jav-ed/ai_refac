//! Rewriting the links of one Markdown file for a set of moves.
//!
//! A link changes when the file it points at is moved, or when the file that
//! contains it is moved (every relative link in it is then measured from a new
//! place). A link that already says the right thing is left byte for byte as it
//! was.

use super::href;
use super::parser::parse_markdown_links;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub(super) struct Rewritten {
    pub content: String,
    pub links_updated: usize,
}

/// `content` was written for a file at `original` and the file will live at
/// `final_path`. `destination` says where a path ends up when it is moved (and
/// returns it unchanged when it is not).
pub(super) fn rewrite(
    content: &str,
    original: &Path,
    final_path: &Path,
    destination: &dyn Fn(&Path) -> PathBuf,
) -> Result<Rewritten> {
    let targets = parse_markdown_links(content)
        .with_context(|| format!("Cannot read the links of {}", original.display()))?;
    let final_dir = final_path
        .parent()
        .with_context(|| format!("Missing parent directory for {}", final_path.display()))?;

    let mut replacements = Vec::new();
    for link in targets {
        let Some(reference) = href::parse(&link.href) else {
            continue;
        };
        let target = href::resolve(original, reference.path).with_context(|| {
            format!(
                "Cannot resolve the link '{}' in {}",
                link.href,
                original.display()
            )
        })?;
        let new_target = destination(&target);
        // What the link says may still be right from the new place (file and
        // target moved together, or neither moved), however it is spelled.
        if href::resolve(final_path, reference.path) == Some(new_target.clone()) {
            continue;
        }

        let wrapped =
            content[..link.href_start].ends_with('<') && content[link.href_end..].starts_with('>');
        let path =
            href::write(reference.path, final_dir, &new_target, wrapped).with_context(|| {
                format!(
                    "Cannot write a link from {} to {}",
                    final_dir.display(),
                    new_target.display()
                )
            })?;
        let rebuilt = format!("{path}{}", reference.suffix);
        if rebuilt != link.href {
            replacements.push((link.href_start, link.href_end, rebuilt));
        }
    }

    // Replace from the end so earlier byte offsets stay valid.
    let links_updated = replacements.len();
    let mut rewritten = content.to_string();
    replacements.sort_by_key(|(start, _, _)| *start);
    for (start, end, replacement) in replacements.into_iter().rev() {
        rewritten.replace_range(start..end, &replacement);
    }

    Ok(Rewritten {
        content: rewritten,
        links_updated,
    })
}

#[cfg(test)]
mod tests;
