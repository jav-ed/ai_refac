use super::RefactorDriver;
use super::complete_filesystem_moves;
use super::lsp_client::LspClient;
use super::lsp_client::SymbolRenameRequest;
use anyhow::{Context, Result};
use async_trait::async_trait;
use lsp_types::Position;
use std::path::{Path, PathBuf};

pub struct RustDriver;

impl RustDriver {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RefactorDriver for RustDriver {
    fn lang(&self) -> &str {
        "rust"
    }

    async fn check_availability(&self) -> Result<bool> {
        let ra_bin = self.get_rust_analyzer_command();
        tracing::info!("Checking rust-analyzer availability at: {}", ra_bin);
        match tokio::process::Command::new(&ra_bin)
            .arg("--version")
            .output()
            .await
        {
            std::result::Result::Ok(output) => {
                let success = output.status.success();
                tracing::info!("rust-analyzer --version success: {}", success);
                Ok(success)
            }
            Err(e) => {
                tracing::warn!("Failed to executed rust-analyzer --version: {:?}", e);
                Ok(false)
            }
        }
    }

    async fn move_files(
        &self,
        file_map: Vec<(String, String)>,
        root_path: Option<&std::path::Path>,
    ) -> Result<()> {
        let ra_bin = self.get_rust_analyzer_command();
        let client = LspClient::new(&ra_bin);
        let root_dir = resolve_root_dir(root_path)?;

        // Pass 1: process cross-dir moves immediately (shims, no LSP).
        // Collect same-dir symbol renames for a single batched LSP session.
        // Same-dir non-symbol moves (mod.rs / no-op identifier) are processed
        // individually because they go through willRenameFiles, not textDocument/rename.
        let mut lsp_batch: Vec<(String, String, RustSymbolRenameRequest)> = Vec::new();

        for (source, target) in &file_map {
            let source_abs = resolve_abs_path(&root_dir, Path::new(source));
            let target_abs = resolve_abs_path(&root_dir, Path::new(target));

            if source_abs.parent() != target_abs.parent()
                && source_abs.file_name().and_then(|n| n.to_str()) != Some("mod.rs")
                && target_abs.file_name().and_then(|n| n.to_str()) != Some("mod.rs")
            {
                apply_cross_dir_rust_move_with_shims(&root_dir, &source_abs, &target_abs)
                    .await?;
                continue;
            }

            if let Some(request) =
                build_rust_symbol_rename_request(&root_dir, &source_abs, &target_abs)?
            {
                lsp_batch.push((source.clone(), target.clone(), request));
            } else {
                client
                    .initialize_and_rename_files(
                        &[],
                        vec![(source.clone(), target.clone())],
                        Some(root_dir.as_path()),
                        Some("rust"),
                        &["rs"],
                    )
                    .await?;
                let single = vec![(source.clone(), target.clone())];
                complete_filesystem_moves(&single, Some(root_dir.as_path())).await?;
            }
        }

        // Pass 2: run all same-dir symbol renames in one LSP session.
        if !lsp_batch.is_empty() {
            let requests: Vec<SymbolRenameRequest> = lsp_batch
                .iter()
                .map(|(source, target, request)| {
                    let source_abs = resolve_abs_path(&root_dir, Path::new(source));
                    let target_abs = resolve_abs_path(&root_dir, Path::new(target));
                    let mut pending_moves = std::collections::HashMap::new();
                    pending_moves.insert(target_abs, source_abs);
                    SymbolRenameRequest {
                        document_path: request.document_path.clone(),
                        position: request.position,
                        new_name: request.new_name.clone(),
                        pending_moves,
                    }
                })
                .collect();

            client
                .initialize_and_rename_symbols_batch(
                    &[],
                    Some(root_dir.as_path()),
                    requests,
                    "rust",
                )
                .await?;

            let filesystem_moves: Vec<(String, String)> = lsp_batch
                .iter()
                .map(|(src, tgt, _)| (src.clone(), tgt.clone()))
                .collect();
            complete_filesystem_moves(&filesystem_moves, Some(root_dir.as_path())).await?;
        }

        Ok(())
    }
}

impl RustDriver {
    fn get_rust_analyzer_command(&self) -> String {
        // 1. Check if in PATH
        if which::which("rust-analyzer").is_ok() {
            tracing::info!("Found rust-analyzer in PATH");
            return "rust-analyzer".to_string();
        }

        // 2. Check standard cargo bin location
        if let Some(home) = std::env::var_os("HOME") {
            let home_path = std::path::Path::new(&home);
            let cargo_bin = home_path.join(".cargo").join("bin").join("rust-analyzer");
            tracing::info!(
                "Checking cargo_bin: {:?} (exists: {})",
                cargo_bin,
                cargo_bin.exists()
            );
            if cargo_bin.exists() {
                return cargo_bin.to_string_lossy().to_string();
            }
        } else {
            tracing::warn!("HOME environment variable not set");
        }

        tracing::warn!("rust-analyzer not found in PATH or .cargo/bin");
        "rust-analyzer".to_string()
    }
}

struct RustSymbolRenameRequest {
    document_path: PathBuf,
    position: Position,
    new_name: String,
}

struct RustModuleDeclaration {
    file_path: PathBuf,
    line_start: usize,
    line_end: usize,
    visibility_prefix: String,
    module_name: String,
}

fn build_rust_symbol_rename_request(
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
        .context("Rust source file is missing a valid stem")?;
    let new_name = target_abs
        .file_stem()
        .and_then(|stem| stem.to_str())
        .context("Rust target file is missing a valid stem")?;

    if old_name == new_name
        || !is_valid_rust_identifier(old_name)
        || !is_valid_rust_identifier(new_name)
    {
        return Ok(None);
    }

    for candidate in rust_module_search_paths(root_dir, source_abs)? {
        let content = std::fs::read_to_string(&candidate)?;
        if let Some(position) = find_rust_module_name_position(&content, old_name) {
            return Ok(Some(RustSymbolRenameRequest {
                document_path: candidate,
                position,
                new_name: new_name.to_string(),
            }));
        }
    }

    Ok(None)
}

async fn apply_cross_dir_rust_move_with_shims(
    root_dir: &Path,
    source_abs: &Path,
    target_abs: &Path,
) -> Result<()> {
    let old_module_path = rust_module_path(root_dir, source_abs)?;
    let target_module_path = rust_module_path(root_dir, target_abs)?;
    let old_name = old_module_path
        .last()
        .cloned()
        .context("Rust source file is missing a logical module name")?;
    let target_name = target_module_path
        .last()
        .cloned()
        .context("Rust target file is missing a logical module name")?;

    let declaration = find_rust_module_declaration(root_dir, source_abs)?.with_context(|| {
        format!(
            "Could not find a Rust module declaration for {:?}",
            source_abs
        )
    })?;

    if let Some(parent) = target_abs.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::rename(source_abs, target_abs).await?;

    let target_parent_file = ensure_rust_target_parent_module_file(
        root_dir,
        &target_module_path[..target_module_path.len().saturating_sub(1)],
        &declaration.visibility_prefix,
    )?;

    let relative_target = diff_paths(
        target_abs,
        declaration
            .file_path
            .parent()
            .context("Rust declaration file is missing a parent directory")?,
    )
    .context("Could not compute relative path from Rust module declaration to target file")?;
    let relative_target = relative_target.to_string_lossy().replace('\\', "/");

    let declaration_replacement = format!(
        "#[path = \"{relative_target}\"]\n{}mod {};",
        declaration.visibility_prefix, declaration.module_name
    );
    replace_byte_range_in_file(
        &declaration.file_path,
        declaration.line_start,
        declaration.line_end,
        &declaration_replacement,
    )?;

    let old_module_abs = format!("crate::{}", old_module_path.join("::"));
    let alias_line = if old_name == target_name {
        format!("{}use {old_module_abs};", declaration.visibility_prefix)
    } else {
        format!(
            "{}use {old_module_abs} as {target_name};",
            declaration.visibility_prefix
        )
    };
    ensure_rust_alias_line(&target_parent_file, &alias_line)?;

    Ok(())
}

fn rust_module_search_paths(root_dir: &Path, source_abs: &Path) -> Result<Vec<PathBuf>> {
    let mut candidates = Vec::new();

    if let Some(parent_dir) = source_abs.parent() {
        if parent_dir == root_dir.join("src") {
            candidates.push(root_dir.join("src/lib.rs"));
            candidates.push(root_dir.join("src/main.rs"));
            candidates.push(root_dir.join("src/mod.rs"));
        } else {
            candidates.push(parent_dir.with_extension("rs"));
            candidates.push(parent_dir.join("mod.rs"));
        }
    }

    collect_workspace_rust_files(root_dir, &mut candidates)?;

    let mut deduped = Vec::new();
    for candidate in candidates {
        if !candidate.exists() || deduped.contains(&candidate) {
            continue;
        }
        deduped.push(candidate);
    }

    Ok(deduped)
}

fn find_rust_module_declaration(
    root_dir: &Path,
    source_abs: &Path,
) -> Result<Option<RustModuleDeclaration>> {
    let module_name = source_abs
        .file_stem()
        .and_then(|stem| stem.to_str())
        .context("Rust source file is missing a valid stem")?;

    for candidate in rust_module_search_paths(root_dir, source_abs)? {
        let content = std::fs::read_to_string(&candidate)?;
        if let Some((line_start, line_end, visibility_prefix)) =
            find_rust_module_declaration_line(&content, module_name)
        {
            return Ok(Some(RustModuleDeclaration {
                file_path: candidate,
                line_start,
                line_end,
                visibility_prefix,
                module_name: module_name.to_string(),
            }));
        }
    }

    Ok(None)
}

fn collect_workspace_rust_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
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

            collect_workspace_rust_files(&path, files)?;
            continue;
        }

        if file_type.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }

    Ok(())
}

fn find_rust_module_name_position(content: &str, module_name: &str) -> Option<Position> {
    for (line_index, line) in content.lines().enumerate() {
        if line.trim_start().starts_with("//") {
            continue;
        }
        let mut search_start = 0;
        while let Some(found) = line[search_start..].find("mod ") {
            let mod_offset = search_start + found;
            let name_start = mod_offset + 4;
            let name_end = rust_identifier_end(line, name_start);

            if name_end > name_start && &line[name_start..name_end] == module_name {
                return Some(Position::new(
                    line_index as u32,
                    utf16_len(&line[..name_start]) as u32,
                ));
            }

            search_start = name_end.max(name_start + 1);
        }
    }

    None
}

fn find_rust_module_declaration_line(
    content: &str,
    module_name: &str,
) -> Option<(usize, usize, String)> {
    let mut offset = 0;
    for line in content.split_inclusive('\n') {
        let line_body = line.strip_suffix('\n').unwrap_or(line);

        // Skip comment lines — they may mention module names but are not declarations.
        if line_body.trim_start().starts_with("//") {
            offset += line.len();
            continue;
        }

        let mut search_start = 0;

        while let Some(found) = line_body[search_start..].find("mod ") {
            let mod_offset = search_start + found;
            let name_start = mod_offset + 4;
            let name_end = rust_identifier_end(line_body, name_start);

            if name_end > name_start
                && &line_body[name_start..name_end] == module_name
                && line_body[name_end..].trim_start().starts_with(';')
            {
                return Some((
                    offset,
                    offset + line.len(),
                    line_body[..mod_offset].to_string(),
                ));
            }

            search_start = name_end.max(name_start + 1);
        }

        offset += line.len();
    }

    None
}

fn rust_identifier_end(line: &str, start: usize) -> usize {
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

fn is_valid_rust_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(ch) if ch == '_' || ch.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn utf16_len(value: &str) -> usize {
    value.encode_utf16().count()
}

fn resolve_root_dir(root_path: Option<&Path>) -> Result<PathBuf> {
    match root_path {
        Some(root) => Ok(root.to_path_buf()),
        None => Ok(std::env::current_dir()?),
    }
}

fn resolve_abs_path(root_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root_dir.join(path)
    }
}

fn crate_root_file(root_dir: &Path) -> Result<PathBuf> {
    let lib_rs = root_dir.join("src/lib.rs");
    if lib_rs.exists() {
        return Ok(lib_rs);
    }

    let main_rs = root_dir.join("src/main.rs");
    if main_rs.exists() {
        return Ok(main_rs);
    }

    anyhow::bail!("Could not find Rust crate root file (src/lib.rs or src/main.rs)")
}

fn rust_module_path(root_dir: &Path, file_path: &Path) -> Result<Vec<String>> {
    let src_dir = root_dir.join("src");
    let rel = file_path
        .strip_prefix(&src_dir)
        .with_context(|| format!("{:?} is outside {:?}", file_path, src_dir))?;

    let mut segments = rel
        .parent()
        .map(|parent| {
            parent
                .components()
                .map(|component| component.as_os_str().to_string_lossy().into_owned())
                .filter(|segment| !segment.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if file_path.file_name().and_then(|name| name.to_str()) != Some("mod.rs") {
        segments.push(
            file_path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .context("Rust file is missing a valid stem")?
                .to_string(),
        );
    }

    Ok(segments)
}

fn ensure_rust_target_parent_module_file(
    root_dir: &Path,
    parent_segments: &[String],
    visibility_prefix: &str,
) -> Result<PathBuf> {
    if parent_segments.is_empty() {
        return crate_root_file(root_dir);
    }

    let parent_file = ensure_rust_target_parent_module_file(
        root_dir,
        &parent_segments[..parent_segments.len() - 1],
        visibility_prefix,
    )?;
    let segment = parent_segments
        .last()
        .context("Missing target parent segment")?;

    let src_dir = root_dir.join("src");
    let dir_path = src_dir.join(parent_segments.join("/"));
    let mod_rs = dir_path.join("mod.rs");
    let flat_rs = dir_path.with_extension("rs");
    let target_file = if mod_rs.exists() {
        mod_rs
    } else if flat_rs.exists() {
        flat_rs
    } else {
        std::fs::create_dir_all(&dir_path)?;
        std::fs::write(&mod_rs, "")?;
        mod_rs
    };

    ensure_rust_module_declaration_exists(&parent_file, segment, visibility_prefix)?;
    Ok(target_file)
}

fn ensure_rust_module_declaration_exists(
    module_file: &Path,
    segment: &str,
    visibility_prefix: &str,
) -> Result<()> {
    let content = std::fs::read_to_string(module_file)?;
    if find_rust_module_declaration_line(&content, segment).is_some() {
        return Ok(());
    }

    let declaration = format!("{}mod {};", visibility_prefix, segment);
    append_rust_line(module_file, &declaration)
}

fn ensure_rust_alias_line(module_file: &Path, alias_line: &str) -> Result<()> {
    let content = std::fs::read_to_string(module_file)?;
    if content.lines().any(|line| line.trim() == alias_line) {
        return Ok(());
    }

    append_rust_line(module_file, alias_line)
}

fn append_rust_line(path: &Path, line: &str) -> Result<()> {
    let mut content = std::fs::read_to_string(path).unwrap_or_default();
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(line);
    content.push('\n');
    std::fs::write(path, content)?;
    Ok(())
}

fn replace_byte_range_in_file(
    path: &Path,
    start: usize,
    end: usize,
    replacement: &str,
) -> Result<()> {
    let mut content = std::fs::read_to_string(path)?;
    content.replace_range(start..end, &format!("{replacement}\n"));
    std::fs::write(path, content)?;
    Ok(())
}

fn diff_paths(target: &Path, base: &Path) -> Option<PathBuf> {
    let target_components = target.components().collect::<Vec<_>>();
    let base_components = base.components().collect::<Vec<_>>();
    let common_len = target_components
        .iter()
        .zip(base_components.iter())
        .take_while(|(left, right)| left == right)
        .count();

    let mut result = PathBuf::new();
    for _ in common_len..base_components.len() {
        result.push("..");
    }
    for component in &target_components[common_len..] {
        result.push(component.as_os_str());
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // Environment probe only — run with `cargo test -- --ignored` to verify rust-analyzer is installed.
    #[tokio::test]
    #[ignore]
    async fn test_rust_availability() -> Result<()> {
        let driver = RustDriver::new();
        let avail = driver.check_availability().await?;
        assert!(avail, "rust-analyzer not found in PATH");
        Ok(())
    }

    #[test]
    fn test_find_rust_module_name_position() {
        let content = "pub(crate) mod alpha;\nuse crate::alpha::value;\n";
        let position = find_rust_module_name_position(content, "alpha").unwrap();
        assert_eq!(position.line, 0);
        assert_eq!(position.character, 15);
    }

    #[tokio::test]
    async fn test_rust_move_updates_module_references() -> Result<()> {
        let driver = RustDriver::new();
        if !driver.check_availability().await? {
            eprintln!("rust-analyzer not found, skipping test");
            return Ok(());
        }

        let temp_dir = tempfile::tempdir()?;
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )?;
        fs::create_dir_all(temp_dir.path().join("src"))?;
        fs::write(
            temp_dir.path().join("src/a.rs"),
            "pub fn value() -> i32 { 1 }\n",
        )?;
        fs::write(
            temp_dir.path().join("src/main.rs"),
            "mod a;\nfn main() {\n    println!(\"{}\", a::value());\n}\n",
        )?;

        driver
            .move_files(
                vec![("src/a.rs".to_string(), "src/b.rs".to_string())],
                Some(temp_dir.path()),
            )
            .await?;

        let main = fs::read_to_string(temp_dir.path().join("src/main.rs"))?;
        assert!(main.contains("mod b;"));
        assert!(main.contains("b::value()"));
        assert!(temp_dir.path().join("src/b.rs").exists());
        assert!(!temp_dir.path().join("src/a.rs").exists());

        Ok(())
    }

    #[tokio::test]
    async fn test_rust_cross_dir_move_keeps_project_buildable() -> Result<()> {
        let driver = RustDriver::new();
        if !driver.check_availability().await? {
            eprintln!("rust-analyzer not found, skipping test");
            return Ok(());
        }

        let temp_dir = tempfile::Builder::new()
            .prefix("refac-rust-test-")
            .tempdir_in(std::env::temp_dir())?;
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )?;
        fs::create_dir_all(temp_dir.path().join("src/engine"))?;
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "pub mod engine;\n\npub fn root_tick() {\n    crate::physics::update();\n}\n",
        )?;
        fs::write(
            temp_dir.path().join("src/engine/mod.rs"),
            "pub mod physics;\npub mod renderer;\n\npub fn tick() {\n    physics::update();\n}\n",
        )?;
        fs::write(
            temp_dir.path().join("src/engine/renderer.rs"),
            "pub fn render() {}\n",
        )?;
        fs::write(
            temp_dir.path().join("src/engine/physics.rs"),
            "use super::renderer;\n\npub fn update() {\n    renderer::render();\n}\n",
        )?;

        driver
            .move_files(
                vec![(
                    "src/engine/physics.rs".to_string(),
                    "src/physics.rs".to_string(),
                )],
                Some(temp_dir.path()),
            )
            .await?;

        let lib_rs = fs::read_to_string(temp_dir.path().join("src/lib.rs"))?;
        let engine_mod = fs::read_to_string(temp_dir.path().join("src/engine/mod.rs"))?;
        assert!(lib_rs.contains("pub use crate::engine::physics;"));
        assert!(engine_mod.contains("#[path = \"../physics.rs\"]"));

        let cargo_check = std::process::Command::new("cargo")
            .arg("check")
            .current_dir(temp_dir.path())
            .output()?;
        assert!(
            cargo_check.status.success(),
            "cargo check failed: {}",
            String::from_utf8_lossy(&cargo_check.stderr)
        );

        Ok(())
    }
}
