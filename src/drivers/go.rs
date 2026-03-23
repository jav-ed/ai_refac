use super::RefactorDriver;
use super::complete_filesystem_moves;
use super::lsp_client::LspClient;
use anyhow::{Context, Result};
use async_trait::async_trait;
use lsp_types::Position;
use std::path::{Path, PathBuf};

pub struct GoDriver;

impl GoDriver {
    pub fn new() -> Self {
        // gopls might be in PATH or in GOPATH/bin
        // We'll try to find it dynamically or default to "gopls"
        Self
    }

    fn find_gopls() -> Option<String> {
        // Check if "gopls" is in PATH
        if which::which("gopls").is_ok() {
            return Some("gopls".to_string());
        }

        // Check common GOPATH
        let home = std::env::var("HOME").ok()?;
        let gopath_bin = PathBuf::from(home).join("go/bin/gopls");
        if gopath_bin.exists() {
            return Some(gopath_bin.to_string_lossy().to_string());
        }

        None
    }
}

#[async_trait]
impl RefactorDriver for GoDriver {
    fn lang(&self) -> &str {
        "go"
    }

    async fn check_availability(&self) -> Result<bool> {
        let binary = Self::find_gopls().unwrap_or_else(|| "gopls".to_string());
        match tokio::process::Command::new(&binary)
            .arg("version")
            .output()
            .await
        {
            std::result::Result::Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }

    async fn move_files(
        &self,
        file_map: Vec<(String, String)>,
        root_path: Option<&std::path::Path>,
    ) -> Result<()> {
        let binary = Self::find_gopls().unwrap_or_else(|| "gopls".to_string());
        let client = LspClient::new(&binary);
        let root_dir = resolve_root_dir(root_path)?;

        for (source, target) in file_map {
            let single_move = vec![(source.clone(), target.clone())];
            let source_abs = resolve_abs_path(&root_dir, Path::new(&source));
            let target_abs = resolve_abs_path(&root_dir, Path::new(&target));

            if let Some(request) =
                build_go_package_rename_request(&root_dir, &source_abs, &target_abs)?
            {
                client
                    .initialize_and_rename_symbol(
                        &[],
                        Some(root_dir.as_path()),
                        &request.document_path,
                        request.position,
                        &request.new_name,
                    )
                    .await?;
            }

            if let Some(lsp_source_abs) =
                build_go_post_lsp_source_path(&source_abs, &target_abs, source_abs != target_abs)
            {
                complete_go_filesystem_move(&lsp_source_abs, &target_abs).await?;
            } else {
                complete_filesystem_moves(&single_move, Some(root_dir.as_path())).await?;
            }
        }

        Ok(())
    }
}

struct GoPackageRenameRequest {
    document_path: PathBuf,
    position: Position,
    new_name: String,
}

fn build_go_package_rename_request(
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

fn build_go_post_lsp_source_path(
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

fn parse_go_module_path(content: &str) -> Option<String> {
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

fn find_go_package_name_position(content: &str) -> Option<Position> {
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

async fn complete_go_filesystem_move(source_abs: &Path, target_abs: &Path) -> Result<()> {
    if source_abs == target_abs {
        return Ok(());
    }

    if target_abs.exists() && !source_abs.exists() {
        return Ok(());
    }

    if let Some(parent) = target_abs.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    match tokio::fs::rename(source_abs, target_abs).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && target_abs.exists() => Ok(()),
        Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn test_gopls_availability() -> Result<()> {
        // This test might fail if gopls isn't installed in the environment where test runs?
        // But we just installed it.
        let driver = GoDriver::new();
        let avail = driver.check_availability().await?;
        if !avail {
            eprintln!("gopls not found, skipping test");
            return Ok(());
        }
        Ok(())
    }

    #[test]
    fn test_parse_go_module_path() {
        let go_mod = "module example.com/demo\n\ngo 1.22\n";
        assert_eq!(
            parse_go_module_path(go_mod).as_deref(),
            Some("example.com/demo")
        );
    }

    #[test]
    fn test_find_go_package_name_position() {
        let position =
            find_go_package_name_position("package util\n\nfunc Value() int { return 1 }\n")
                .unwrap();
        assert_eq!(position.line, 0);
        assert_eq!(position.character, 8);
    }

    #[tokio::test]
    async fn test_go_move_updates_imports() -> Result<()> {
        let driver = GoDriver::new();
        if !driver.check_availability().await? {
            eprintln!("gopls not found, skipping test");
            return Ok(());
        }

        let temp_dir = tempfile::Builder::new()
            .prefix("refac-go-test-")
            .tempdir_in(std::env::temp_dir())?;
        fs::write(
            temp_dir.path().join("go.mod"),
            "module example.com/demo\n\ngo 1.22\n",
        )?;
        fs::create_dir_all(temp_dir.path().join("util"))?;
        fs::write(
            temp_dir.path().join("util/util.go"),
            "package util\n\nfunc Value() int { return 1 }\n",
        )?;
        fs::write(
            temp_dir.path().join("main.go"),
            "package main\n\nimport (\n    \"fmt\"\n    \"example.com/demo/util\"\n)\n\nfunc main() {\n    fmt.Println(util.Value())\n}\n",
        )?;

        driver
            .move_files(
                vec![("util/util.go".to_string(), "util2/util.go".to_string())],
                Some(temp_dir.path()),
            )
            .await?;

        let main = fs::read_to_string(temp_dir.path().join("main.go"))?;
        let util = fs::read_to_string(temp_dir.path().join("util2/util.go"))?;
        assert!(main.contains("\"example.com/demo/util2\""));
        assert!(main.contains("util2.Value()"));
        assert!(util.contains("package util2"));
        assert!(temp_dir.path().join("util2/util.go").exists());

        Ok(())
    }

    #[tokio::test]
    async fn test_go_move_updates_imports_when_filename_changes() -> Result<()> {
        let driver = GoDriver::new();
        if !driver.check_availability().await? {
            eprintln!("gopls not found, skipping test");
            return Ok(());
        }

        let temp_dir = tempfile::Builder::new()
            .prefix("refac-go-rename-test-")
            .tempdir_in(std::env::temp_dir())?;
        fs::write(
            temp_dir.path().join("go.mod"),
            "module example.com/demo\n\ngo 1.22\n",
        )?;
        fs::create_dir_all(temp_dir.path().join("util"))?;
        fs::write(
            temp_dir.path().join("util/util.go"),
            "package util\n\nfunc Value() int { return 1 }\n",
        )?;
        fs::write(
            temp_dir.path().join("main.go"),
            "package main\n\nimport (\n    \"fmt\"\n    \"example.com/demo/util\"\n)\n\nfunc main() {\n    fmt.Println(util.Value())\n}\n",
        )?;

        driver
            .move_files(
                vec![("util/util.go".to_string(), "util2/renamed.go".to_string())],
                Some(temp_dir.path()),
            )
            .await?;

        let main = fs::read_to_string(temp_dir.path().join("main.go"))?;
        let renamed = fs::read_to_string(temp_dir.path().join("util2/renamed.go"))?;
        assert!(main.contains("\"example.com/demo/util2\""));
        assert!(main.contains("util2.Value()"));
        assert!(renamed.contains("package util2"));
        assert!(temp_dir.path().join("util2/renamed.go").exists());
        assert!(!temp_dir.path().join("util/util.go").exists());

        let go_build = std::process::Command::new("go")
            .args(["build", "./..."])
            .current_dir(temp_dir.path())
            .output()?;
        assert!(
            go_build.status.success(),
            "go build failed: {}",
            String::from_utf8_lossy(&go_build.stderr)
        );

        Ok(())
    }
}
