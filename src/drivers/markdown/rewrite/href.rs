use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};

use super::pathing::normalize_path;

#[derive(Debug, Clone)]
pub(super) struct ResolvedHref {
    pub absolute_path: PathBuf,
    pub fragment: Option<String>,
}

pub(super) fn classify_link_target(
    original_file: &Path,
    href: &str,
) -> Result<Option<ResolvedHref>> {
    if is_external_href(href) {
        return Ok(None);
    }

    let (path_part, fragment) = split_href_fragment(href);
    if path_part.is_empty() {
        return Ok(None);
    }

    let original_dir = original_file
        .parent()
        .ok_or_else(|| anyhow!("Missing parent directory for {}", original_file.display()))?;
    let absolute_path = normalize_path(original_dir.join(path_part));

    Ok(Some(ResolvedHref {
        absolute_path,
        fragment,
    }))
}

pub(super) fn rebuild_href(path: &str, fragment: Option<&str>) -> String {
    match fragment {
        Some(fragment) if !fragment.is_empty() => format!("{path}#{fragment}"),
        _ => path.to_string(),
    }
}

pub(super) fn split_href_fragment(href: &str) -> (&str, Option<String>) {
    if let Some((path, fragment)) = href.split_once('#') {
        (path, Some(fragment.to_string()))
    } else {
        (href, None)
    }
}

fn is_external_href(href: &str) -> bool {
    let trimmed = href.trim();
    trimmed.starts_with('#')
        || trimmed.starts_with('/')
        || has_uri_scheme(trimmed)
}

fn has_uri_scheme(href: &str) -> bool {
    let Some(colon_index) = href.find(':') else {
        return false;
    };

    let scheme = &href[..colon_index];
    let Some(first) = scheme.chars().next() else {
        return false;
    };

    if !first.is_ascii_alphabetic() {
        return false;
    }

    // Treat URI schemes as external, but avoid misclassifying Windows drive
    // letters like `C:\path\file.md`.
    if scheme.len() == 1 {
        return false;
    }

    scheme
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'))
}
