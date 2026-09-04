use super::{apply::TextReplacement, module_graph::ResolvedModule, workspace::SemanticWorkspace};
use anyhow::{Context, Result, bail};
use ra_ap_ide::{FileId, FilePosition, FindAllRefsConfig, RaFixtureConfig, TextRange};
use ra_ap_syntax::{
    AstNode, Edition, SourceFile,
    ast::{self, HasName},
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub fn module_reference_edits(
    workspace: &SemanticWorkspace,
    source: &ResolvedModule,
    target_segments: &[String],
) -> Result<Vec<TextReplacement>> {
    let file_id = super::module_graph::declaration_file_id(workspace, source)?;
    let config = FindAllRefsConfig {
        search_scope: None,
        ra_fixture: RaFixtureConfig::default(),
        exclude_imports: false,
        exclude_tests: false,
    };
    let results = workspace
        .analysis()
        .find_all_refs(
            FilePosition {
                file_id,
                offset: source.name_range.start(),
            },
            &config,
        )?
        .context("rust-analyzer could not search references for the source module")?;

    let mut contents = HashMap::<FileId, (PathBuf, String)>::new();
    let mut edits = Vec::new();
    for result in results {
        for (reference_file_id, references) in result.references {
            let (path, content) = match contents.get(&reference_file_id) {
                Some(value) => value,
                None => {
                    let path = workspace.file_path(reference_file_id)?;
                    if !path.starts_with(workspace.root()) {
                        bail!(
                            "A reference to `crate::{}` resolves outside the Cargo workspace: {}",
                            source.segments.join("::"),
                            path.display()
                        );
                    }
                    let content = std::fs::read_to_string(&path)?;
                    contents.insert(reference_file_id, (path, content));
                    contents.get(&reference_file_id).unwrap()
                }
            };

            let same_crate = workspace.file_belongs_to_crate(reference_file_id, source.krate)?;
            for (range, _) in references {
                if path == &source.declaration_file && range == source.name_range {
                    continue;
                }
                if let Some(edit) = reference_edit(
                    path,
                    content,
                    range,
                    &source.segments,
                    target_segments,
                    same_crate,
                )? {
                    edits.push(edit);
                }
            }
        }
    }
    Ok(edits)
}

pub fn super_path_edits(
    path: &Path,
    content: &str,
    file_module: &[String],
) -> Result<Vec<TextReplacement>> {
    let parse = SourceFile::parse(content, Edition::CURRENT);
    if !parse.errors().is_empty() {
        bail!(
            "Cannot move {} because it contains Rust syntax errors",
            path.display()
        );
    }

    let mut edits = Vec::new();
    for candidate in parse
        .tree()
        .syntax()
        .descendants()
        .filter_map(ast::Path::cast)
    {
        if candidate
            .syntax()
            .ancestors()
            .skip(1)
            .any(|ancestor| ast::Path::cast(ancestor).is_some())
        {
            continue;
        }

        let text = candidate.syntax().text().to_string();
        let segments = simple_segments(&text)?;
        let super_count = segments
            .iter()
            .take_while(|segment| segment.as_str() == "super")
            .count();
        if super_count == 0 {
            continue;
        }

        let mut context = file_module.to_vec();
        let mut inline_modules = candidate
            .syntax()
            .ancestors()
            .skip(1)
            .filter_map(ast::Module::cast)
            .filter_map(|module| module.name().map(|name| name.text().to_string()))
            .collect::<Vec<_>>();
        inline_modules.reverse();
        context.extend(inline_modules);

        if super_count > context.len() {
            bail!(
                "Path `{text}` in {} climbs above the crate root",
                path.display()
            );
        }
        context.truncate(context.len() - super_count);
        context.extend(segments.into_iter().skip(super_count));
        let replacement = if context.is_empty() {
            "crate".to_string()
        } else {
            format!("crate::{}", context.join("::"))
        };
        edits.push(TextReplacement::from_range(
            path.to_path_buf(),
            candidate.syntax().text_range(),
            replacement,
        ));
    }
    Ok(edits)
}

fn reference_edit(
    path: &Path,
    content: &str,
    reference: TextRange,
    source: &[String],
    target: &[String],
    same_crate: bool,
) -> Result<Option<TextReplacement>> {
    let parse = SourceFile::parse(content, Edition::CURRENT);
    if !parse.errors().is_empty() {
        bail!(
            "Cannot rewrite module reference in {} because it contains Rust syntax errors",
            path.display()
        );
    }
    let token = parse
        .tree()
        .syntax()
        .token_at_offset(reference.start())
        .find(|token| token.text_range().contains_range(reference))
        .with_context(|| {
            format!(
                "Could not locate rust-analyzer reference in {}",
                path.display()
            )
        })?;
    if matches!(token.text(), "self" | "super") {
        return Ok(None);
    }
    let path_node = token
        .parent_ancestors()
        .filter_map(ast::Path::cast)
        .filter(|node| node.syntax().text_range().contains_range(reference))
        .last()
        .with_context(|| {
            format!(
                "Reference at byte {} in {} is not a rewritable Rust path; macro-generated and documentation references are not supported in v1",
                usize::from(reference.start()),
                path.display()
            )
        })?;

    let prefix_range = TextRange::new(path_node.syntax().text_range().start(), reference.end());
    let prefix_text = content
        .get(usize::from(prefix_range.start())..usize::from(prefix_range.end()))
        .context("rust-analyzer reference range is outside the source file")?;
    let leaf = simple_segments(prefix_text)?;
    let inherited = inherited_use_segments(&path_node)?;
    let mut full = inherited.clone();
    full.extend(leaf);

    let desired = desired_reference_path(&full, source, target, same_crate)?;
    let Some(desired) = desired else {
        return Ok(None);
    };
    if !desired.starts_with(&inherited) {
        bail!(
            "Grouped use in {} cannot express the new module path without restructuring the use item; split the relative group and retry",
            path.display()
        );
    }
    let replacement = desired[inherited.len()..].join("::");
    Ok(Some(TextReplacement::from_range(
        path.to_path_buf(),
        prefix_range,
        replacement,
    )))
}

fn desired_reference_path(
    full: &[String],
    source: &[String],
    target: &[String],
    same_crate: bool,
) -> Result<Option<Vec<String>>> {
    let source_start = full.len().checked_sub(source.len());
    let encodes_source = source_start.is_some_and(|start| full[start..] == *source);
    if !same_crate && !encodes_source {
        return Ok(None);
    }
    if !same_crate && source_start == Some(0) {
        return Ok(None);
    }

    if same_crate {
        let mut canonical = vec!["crate".to_string()];
        canonical.extend_from_slice(target);
        return Ok(Some(canonical));
    }

    let start =
        source_start.context("External reference does not encode the source module path")?;
    let mut rewritten = full[..start].to_vec();
    rewritten.extend_from_slice(target);
    Ok(Some(rewritten))
}

fn inherited_use_segments(path: &ast::Path) -> Result<Vec<String>> {
    let mut prefixes = path
        .syntax()
        .ancestors()
        .skip(1)
        .filter_map(ast::UseTree::cast)
        .filter_map(|tree| tree.path())
        .filter(|candidate| candidate.syntax() != path.syntax())
        .map(|path| simple_segments(&path.syntax().text().to_string()))
        .collect::<Result<Vec<_>>>()?;
    prefixes.reverse();
    Ok(prefixes.into_iter().flatten().collect())
}

fn simple_segments(text: &str) -> Result<Vec<String>> {
    let segments = text
        .split("::")
        .map(str::trim)
        .map(str::to_string)
        .collect::<Vec<_>>();
    if segments.is_empty()
        || segments.iter().any(|segment| {
            segment.is_empty()
                || !segment
                    .chars()
                    .all(|character| character == '_' || character.is_ascii_alphanumeric())
        })
    {
        bail!("Unsupported complex Rust path `{text}` in move-module v1");
    }
    Ok(segments)
}
