use anyhow::{Context, Result};
use lsp_types::Position;
use std::path::{Path, PathBuf};

pub struct RustSymbolRenameRequest {
    pub document_path: PathBuf,
    pub position: Position,
    pub new_name: String,
}

pub fn build_symbol_rename_request(
    root_dir: &Path,
    source_abs: &Path,
    target_abs: &Path,
) -> Result<Option<RustSymbolRenameRequest>> {
    if source_abs.parent() != target_abs.parent() {
        return Ok(None);
    }

    if source_abs.file_name().and_then(|name| name.to_str()) == Some("mod.rs")
        || target_abs.file_name().and_then(|name| name.to_str()) == Some("mod.rs")
    {
        return Ok(None);
    }

    let old_name = source_abs
        .file_stem()
        .and_then(|stem| stem.to_str())
        .context("Rust source file is missing a valid UTF-8 stem")?;
    let new_name = target_abs
        .file_stem()
        .and_then(|stem| stem.to_str())
        .context("Rust target file is missing a valid UTF-8 stem")?;

    if old_name == new_name || !is_identifier(old_name) || !is_identifier(new_name) {
        return Ok(None);
    }

    for candidate in module_search_paths(root_dir, source_abs)? {
        let content = std::fs::read_to_string(&candidate)?;
        if let Some(position) = module_name_position(&content, old_name) {
            return Ok(Some(RustSymbolRenameRequest {
                document_path: candidate,
                position,
                new_name: new_name.to_string(),
            }));
        }
    }

    Ok(None)
}

pub fn absolute_path(root_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root_dir.join(path)
    }
}

fn module_search_paths(root_dir: &Path, source_abs: &Path) -> Result<Vec<PathBuf>> {
    let mut candidates = Vec::new();
    if let Some(parent_dir) = source_abs.parent() {
        if parent_dir == root_dir.join("src") {
            candidates.extend([
                root_dir.join("src/lib.rs"),
                root_dir.join("src/main.rs"),
                root_dir.join("src/mod.rs"),
            ]);
        } else {
            candidates.extend([parent_dir.with_extension("rs"), parent_dir.join("mod.rs")]);
        }
    }

    collect_rust_files(root_dir, &mut candidates)?;
    let mut deduped = Vec::new();
    for candidate in candidates {
        if candidate.exists() && !deduped.contains(&candidate) {
            deduped.push(candidate);
        }
    }
    Ok(deduped)
}

fn collect_rust_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            if matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some(".git" | "target")
            ) {
                continue;
            }
            collect_rust_files(&path, files)?;
        } else if file_type.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("rs")
        {
            files.push(path);
        }
    }
    Ok(())
}

fn module_name_position(content: &str, module_name: &str) -> Option<Position> {
    for (line_index, line) in content.lines().enumerate() {
        if line.trim_start().starts_with("//") {
            continue;
        }
        let mut search_start = 0;
        while let Some(found) = line[search_start..].find("mod ") {
            let name_start = search_start + found + 4;
            let name_end = identifier_end(line, name_start);
            if name_end > name_start && &line[name_start..name_end] == module_name {
                return Some(Position::new(
                    line_index as u32,
                    line[..name_start].encode_utf16().count() as u32,
                ));
            }
            search_start = name_end.max(name_start + 1);
        }
    }
    None
}

fn identifier_end(line: &str, start: usize) -> usize {
    line.as_bytes()[start..]
        .iter()
        .position(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
        .map(|offset| start + offset)
        .unwrap_or(line.len())
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(ch) if ch == '_' || ch.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::module_name_position;

    #[test]
    fn finds_utf16_module_position() {
        let position = module_name_position("pub(crate) mod alpha;\n", "alpha").unwrap();
        assert_eq!(position.line, 0);
        assert_eq!(position.character, 15);
    }
}
