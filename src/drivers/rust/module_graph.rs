use super::workspace::SemanticWorkspace;
use anyhow::{Context, Result, bail};
use ra_ap_hir::{Crate, Module};
use ra_ap_ide::{FileId, TextRange};
use ra_ap_syntax::{
    AstNode,
    ast::{HasAttrs, HasName, HasVisibility},
};
use std::path::PathBuf;

#[derive(Clone)]
pub struct ResolvedModule {
    pub module: Module,
    pub krate: Crate,
    pub segments: Vec<String>,
    pub definition_file: PathBuf,
    pub declaration_file: PathBuf,
    pub declaration_range: TextRange,
    pub name_range: TextRange,
    pub declaration_text: String,
    pub visibility: String,
    pub is_mod_rs: bool,
}

pub fn parse_module_path(value: &str) -> Result<Vec<String>> {
    let mut parts = value.split("::");
    if parts.next() != Some("crate") {
        bail!("Rust module path must start with `crate::`: {value}");
    }

    let segments = parts.map(str::to_string).collect::<Vec<_>>();
    if segments.is_empty() {
        bail!("The crate root cannot be moved; provide a child module path");
    }
    for segment in &segments {
        if !is_identifier(segment) {
            bail!("Unsupported Rust module segment `{segment}` in `{value}`");
        }
    }
    Ok(segments)
}

pub fn resolve_source(
    workspace: &SemanticWorkspace,
    source_segments: &[String],
) -> Result<ResolvedModule> {
    let database = workspace.database();
    let mut matches = Vec::new();

    for krate in Crate::all(database) {
        if !workspace.is_local_crate(krate)? {
            continue;
        }
        for module in krate.modules(database) {
            if module_segments(module, database) != source_segments {
                continue;
            }
            let resolved = resolve_module(workspace, krate, module)?;
            if !matches.iter().any(|existing: &ResolvedModule| {
                existing.declaration_file == resolved.declaration_file
                    && existing.declaration_range == resolved.declaration_range
            }) {
                matches.push(resolved);
            }
        }
    }

    match matches.len() {
        0 => bail!(
            "Could not resolve `crate::{}` to an out-of-line module in {}",
            source_segments.join("::"),
            workspace.root().display()
        ),
        1 => Ok(matches.remove(0)),
        count => {
            let locations = matches
                .iter()
                .map(|module| module.declaration_file.display().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            bail!(
                "`crate::{}` is ambiguous across {count} workspace crates: {locations}",
                source_segments.join("::")
            )
        }
    }
}

pub fn find_module(
    workspace: &SemanticWorkspace,
    krate: Crate,
    segments: &[String],
) -> Result<Option<ResolvedModule>> {
    if segments.is_empty() {
        return Ok(None);
    }
    let database = workspace.database();
    for module in krate.modules(database) {
        if module_segments(module, database) == segments {
            return resolve_module(workspace, krate, module).map(Some);
        }
    }
    Ok(None)
}

pub fn collect_subtree(
    workspace: &SemanticWorkspace,
    root: &ResolvedModule,
) -> Result<Vec<ResolvedModule>> {
    let database = workspace.database();
    let mut modules = Vec::new();
    let mut pending = vec![root.module];
    while let Some(module) = pending.pop() {
        let resolved = resolve_module(workspace, root.krate, module)?;
        for child in module.children(database) {
            if child.is_inline(database) {
                if child
                    .children(database)
                    .any(|nested| !nested.is_inline(database))
                {
                    bail!(
                        "Inline module `crate::{}` declares an out-of-line child; this layout is not supported by move-module v1",
                        module_segments(child, database).join("::")
                    );
                }
                continue;
            }
            pending.push(child);
        }
        modules.push(resolved);
    }
    Ok(modules)
}

fn resolve_module(
    workspace: &SemanticWorkspace,
    krate: Crate,
    module: Module,
) -> Result<ResolvedModule> {
    let database = workspace.database();
    if module.is_inline(database) {
        bail!(
            "Inline module `crate::{}` is not supported by move-module v1",
            module_segments(module, database).join("::")
        );
    }
    if module.has_path(database) {
        bail!(
            "Module `crate::{}` uses #[path]; move-module never follows or creates path shims",
            module_segments(module, database).join("::")
        );
    }

    let source_file = module
        .as_source_file_id(database)
        .context("Resolved module has no physical source file")?;
    let definition_file = workspace.file_path(source_file.file_id(database))?;
    let declaration = module
        .declaration_source(database)
        .context("Resolved module has no source declaration")?;
    let declaration_file_id = declaration
        .file_id
        .original_file(database)
        .file_id(database);
    let declaration_file = workspace.file_path(declaration_file_id)?;
    let declaration_range = declaration.value.syntax().text_range();
    let name_range = declaration
        .value
        .name()
        .context("Rust module declaration has no name")?
        .syntax()
        .text_range();
    let declaration_content = std::fs::read_to_string(&declaration_file)?;
    let declaration_text = text_at(&declaration_content, declaration_range)?.to_string();
    let visibility = declaration
        .value
        .visibility()
        .map(|value| value.syntax().text().to_string())
        .unwrap_or_default();

    if declaration.value.attrs().next().is_some() {
        bail!(
            "Module declaration for `crate::{}` has attributes; cfg and attributed module moves are not supported in v1",
            module_segments(module, database).join("::")
        );
    }
    if !matches!(visibility.as_str(), "" | "pub" | "pub(crate)") {
        bail!(
            "Module `crate::{}` uses unsupported visibility `{visibility}`; v1 supports private, pub, and pub(crate)",
            module_segments(module, database).join("::")
        );
    }

    Ok(ResolvedModule {
        module,
        krate,
        segments: module_segments(module, database),
        definition_file,
        declaration_file,
        declaration_range,
        name_range,
        declaration_text,
        visibility,
        is_mod_rs: module.is_mod_rs(database),
    })
}

fn module_segments(module: Module, database: &dyn ra_ap_hir::db::HirDatabase) -> Vec<String> {
    let edition = module.krate(database).edition(database);
    module
        .path_segments(database)
        .map(|name| name.display(database, edition).to_string())
        .collect()
}

fn text_at(content: &str, range: TextRange) -> Result<&str> {
    content
        .get(usize::from(range.start())..usize::from(range.end()))
        .context("rust-analyzer returned a source range outside the declaration file")
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(ch) if ch == '_' || ch.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

pub fn declaration_file_id(
    workspace: &SemanticWorkspace,
    module: &ResolvedModule,
) -> Result<FileId> {
    let declaration = module
        .module
        .declaration_source(workspace.database())
        .context("Resolved module has no declaration source")?;
    Ok(declaration
        .file_id
        .original_file(workspace.database())
        .file_id(workspace.database()))
}
