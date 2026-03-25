use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};

use crate::drivers::complete_filesystem_moves;

use super::parser::parse_markdown_links;

mod href;
mod pathing;

use href::{classify_link_target, rebuild_href};
use pathing::{
    ResolvedMove, collect_markdown_files, determine_workspace_root, read_markdown_files,
    relative_link_from, resolve_moves,
};

pub(crate) async fn move_markdown_files(
    file_map: Vec<(String, String)>,
    root_path: Option<&Path>,
) -> Result<()> {
    let resolved_moves = resolve_moves(&file_map, root_path)?;
    if resolved_moves.is_empty() {
        return Ok(());
    }

    let workspace_root = determine_workspace_root(&resolved_moves, root_path)?;
    let markdown_files = collect_markdown_files(&workspace_root)?;
    let file_contents = read_markdown_files(&markdown_files).await?;

    let rewrite_plan = build_rewrite_plan(&resolved_moves, &file_contents)?;

    complete_filesystem_moves(&file_map, root_path).await?;

    for (final_path, content) in rewrite_plan {
        if let Some(parent) = final_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&final_path, content)
            .await
            .with_context(|| {
                format!(
                    "Failed to write updated markdown content to {}",
                    final_path.display()
                )
            })?;
    }

    Ok(())
}

fn build_rewrite_plan(
    resolved_moves: &[ResolvedMove],
    file_contents: &HashMap<PathBuf, String>,
) -> Result<HashMap<PathBuf, String>> {
    let move_lookup: HashMap<PathBuf, PathBuf> = resolved_moves
        .iter()
        .map(|entry| (entry.source_abs.clone(), entry.target_abs.clone()))
        .collect();

    let mut plan = HashMap::new();

    for (original_path, content) in file_contents {
        let final_path = move_lookup
            .get(original_path)
            .cloned()
            .unwrap_or_else(|| original_path.clone());

        let updated = rewrite_markdown_links(content, original_path, &final_path, &move_lookup)?;

        if updated != *content || final_path != *original_path {
            plan.insert(final_path, updated);
        }
    }

    Ok(plan)
}

fn rewrite_markdown_links(
    content: &str,
    original_path: &Path,
    final_path: &Path,
    move_lookup: &HashMap<PathBuf, PathBuf>,
) -> Result<String> {
    let parsed = parse_markdown_links(content);
    if parsed.targets.is_empty() {
        return Ok(content.to_string());
    }

    let final_dir = final_path
        .parent()
        .ok_or_else(|| anyhow!("Missing parent directory for {}", final_path.display()))?;

    let mut rewritten = content.to_string();
    let mut replacements = Vec::new();

    for link in parsed.targets {
        let Some(target) = classify_link_target(original_path, &link.href)? else {
            continue;
        };

        let mut new_target_abs = target.absolute_path.clone();
        if let Some(mapped) = move_lookup.get(&target.absolute_path) {
            new_target_abs = mapped.clone();
        }

        let should_rewrite = final_path != original_path || new_target_abs != target.absolute_path;
        if !should_rewrite {
            continue;
        }

        let relative = relative_link_from(final_dir, &new_target_abs)?;
        let rebuilt = rebuild_href(&relative, target.fragment.as_deref());
        if rebuilt != link.href {
            replacements.push((link.href_start, link.href_end, rebuilt));
        }
    }

    // Apply highest-position replacements first so earlier byte offsets are not shifted.
    replacements.sort_by_key(|(start, _, _)| *start);
    for (start, end, replacement) in replacements.into_iter().rev() {
        rewritten.replace_range(start..end, &replacement);
    }

    Ok(rewritten)
}

#[cfg(test)]
mod tests {
    use super::{rebuild_href, relative_link_from, rewrite_markdown_links};
    use crate::drivers::markdown::rewrite::href::split_href_fragment;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn rewrites_links_when_target_moves() {
        let content = "# Index\n\nSee [Guide](./guide.md#deep-dive).\n";
        let original = PathBuf::from("/tmp/project/index.md");
        let final_path = original.clone();
        let lookup = HashMap::from([(
            PathBuf::from("/tmp/project/guide.md"),
            PathBuf::from("/tmp/project/reference/guide.md"),
        )]);

        let rewritten = rewrite_markdown_links(content, &original, &final_path, &lookup).unwrap();

        assert!(rewritten.contains("(./reference/guide.md#deep-dive)"));
    }

    #[test]
    fn rewrites_links_when_file_itself_moves() {
        let content = "# Guide\n\nSee [Sibling](./sibling.md).\n";
        let original = PathBuf::from("/tmp/project/guide.md");
        let final_path = PathBuf::from("/tmp/project/docs/guide.md");
        let lookup = HashMap::new();

        let rewritten = rewrite_markdown_links(content, &original, &final_path, &lookup).unwrap();

        assert!(rewritten.contains("(../sibling.md)"));
    }

    #[test]
    fn rewrites_reference_definitions_when_target_moves() {
        let content = "# Index\n\nSee [Guide][guide].\n\n[guide]: ./guide.md#deep-dive \"Title\"\n";
        let original = PathBuf::from("/tmp/project/index.md");
        let final_path = original.clone();
        let lookup = HashMap::from([(
            PathBuf::from("/tmp/project/guide.md"),
            PathBuf::from("/tmp/project/reference/guide.md"),
        )]);

        let rewritten = rewrite_markdown_links(content, &original, &final_path, &lookup).unwrap();

        assert!(rewritten.contains("[guide]: ./reference/guide.md#deep-dive \"Title\""));
        assert!(rewritten.contains("See [Guide][guide]."));
    }

    #[test]
    fn rewrites_reference_definitions_when_file_itself_moves() {
        let content = "# Guide\n\nSee [Sibling][sibling].\n\n[sibling]: ./sibling.md\n";
        let original = PathBuf::from("/tmp/project/guide.md");
        let final_path = PathBuf::from("/tmp/project/docs/guide.md");
        let lookup = HashMap::new();

        let rewritten = rewrite_markdown_links(content, &original, &final_path, &lookup).unwrap();

        assert!(rewritten.contains("[sibling]: ../sibling.md"));
        assert!(rewritten.contains("See [Sibling][sibling]."));
    }

    #[test]
    fn preserves_fragment_rebuild() {
        let (path, fragment) = split_href_fragment("./guide.md#anchor");
        let rebuilt = rebuild_href(path, fragment.as_deref());
        assert_eq!(rebuilt, "./guide.md#anchor");
    }

    #[test]
    fn rewrites_both_inline_and_trailing_reference_definition_when_file_moves() {
        // Regression: reference definitions are collected before inline links in the targets
        // vector, but they appear after them in the file. The old code used .rev() without
        // sorting, so applying the inline-link replacement first shifted the string and the
        // reference-definition replacement hit the wrong byte range, corrupting the output
        // (e.g. "[home]../../index.md" instead of "[home]: ../../index.md").
        let content = "# Guide\n\nSee [Setup](./setup.md).\n\n[home]: ../index.md \"Home\"\n";
        let original = PathBuf::from("/tmp/project/docs/guide.md");
        let final_path = PathBuf::from("/tmp/project/docs/sub/guide.md");
        let lookup = HashMap::new();

        let rewritten = rewrite_markdown_links(content, &original, &final_path, &lookup).unwrap();

        assert!(
            rewritten.contains("[home]: ../../index.md \"Home\""),
            "reference definition colon was corrupted: {rewritten:?}"
        );
        assert!(
            rewritten.contains("(../setup.md)"),
            "inline link was not recalculated: {rewritten:?}"
        );
    }

    #[test]
    fn prefixes_same_directory_relative_links() {
        let path = relative_link_from(
            &PathBuf::from("/tmp/project/docs"),
            &PathBuf::from("/tmp/project/docs/guide.md"),
        )
        .unwrap();

        assert_eq!(path, "./guide.md");
    }
}
