//! What gopls is asked for when files move: a Go package is a directory, so a
//! move to another directory is the rename of the package's import path, asked
//! at the `package` line of the moved file.

use anyhow::{Context, Result};
use lsp_types::Position;
use std::path::{Path, PathBuf};

pub(super) struct GoPackageRenameRequest {
    pub(super) document_path: PathBuf,
    pub(super) position: Position,
    pub(super) new_name: String,
}

pub(super) fn build_go_package_rename_request(
    root_dir: &Path,
    source_abs: &Path,
    target_abs: &Path,
) -> Result<Option<GoPackageRenameRequest>> {
    if source_abs.parent() == target_abs.parent() {
        return Ok(None);
    }

    let source_content = std::fs::read_to_string(source_abs)?;
    let position = find_go_package_name_position(&source_content)
        .context("Could not find a package declaration in the Go source file")?;
    let new_name = build_go_target_package_path(root_dir, target_abs)?;

    Ok(Some(GoPackageRenameRequest {
        document_path: source_abs.to_path_buf(),
        position,
        new_name,
    }))
}

fn build_go_target_package_path(root_dir: &Path, target_abs: &Path) -> Result<String> {
    let go_mod = std::fs::read_to_string(root_dir.join("go.mod")).context(
        "Go refactors that move files across directories require go.mod at project root",
    )?;
    let module_path =
        parse_go_module_path(&go_mod).context("Could not parse module path from go.mod")?;
    let target_dir = target_abs
        .parent()
        .context("Go target path is missing a parent directory")?;
    let rel_target_dir = target_dir.strip_prefix(root_dir).with_context(|| {
        format!(
            "Go target directory {:?} is outside project root",
            target_dir
        )
    })?;

    let rel = rel_target_dir
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("/");

    if rel.is_empty() {
        Ok(module_path)
    } else {
        Ok(format!("{module_path}/{rel}"))
    }
}

pub(super) fn build_go_post_lsp_source_path(
    source_abs: &Path,
    target_abs: &Path,
    did_invoke_package_rename: bool,
) -> Option<PathBuf> {
    if !did_invoke_package_rename || source_abs.parent() == target_abs.parent() {
        return None;
    }

    let source_name = source_abs.file_name()?;
    Some(target_abs.parent()?.join(source_name))
}

pub(super) fn parse_go_module_path(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        if let Some(value) = trimmed.strip_prefix("module ") {
            return Some(value.trim().trim_matches('"').to_string());
        }
    }

    None
}

pub(super) fn find_go_package_name_position(content: &str) -> Option<Position> {
    for (line_index, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("package ") {
            continue;
        }

        let indent = line.len() - trimmed.len();
        let name_start = indent + "package ".len();
        let name_end = go_identifier_end(line, name_start);
        if name_end > name_start {
            return Some(Position::new(
                line_index as u32,
                utf16_len(&line[..name_start]) as u32,
            ));
        }
    }

    None
}

fn go_identifier_end(line: &str, start: usize) -> usize {
    let bytes = line.as_bytes();
    let mut end = start;

    while end < bytes.len() {
        let byte = bytes[end];
        if byte.is_ascii_alphanumeric() || byte == b'_' {
            end += 1;
        } else {
            break;
        }
    }

    end
}

fn utf16_len(value: &str) -> usize {
    value.encode_utf16().count()
}

pub(super) fn resolve_root_dir(root_path: Option<&Path>) -> Result<PathBuf> {
    match root_path {
        Some(root) => Ok(root.to_path_buf()),
        None => Ok(std::env::current_dir()?),
    }
}

pub(super) fn resolve_abs_path(root_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root_dir.join(path)
    }
}
