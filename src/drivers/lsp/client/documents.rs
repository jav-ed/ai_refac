//! The project files a language server is shown before it is asked anything.

use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn collect_workspace_documents(
    root_dir: &Path,
    file_extensions: &[&str],
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_workspace_documents_recursive(root_dir, file_extensions, &mut files)?;
    Ok(files)
}

fn collect_workspace_documents_recursive(
    dir: &Path,
    file_extensions: &[&str],
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            if should_skip_workspace_dir(&path) {
                continue;
            }

            collect_workspace_documents_recursive(&path, file_extensions, files)?;
            continue;
        }

        if file_type.is_file()
            && path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| file_extensions.contains(&ext))
                .unwrap_or(false)
        {
            files.push(path);
        }
    }

    Ok(())
}

fn should_skip_workspace_dir(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(".git")
            | Some(".venv")
            | Some("__pycache__")
            | Some("node_modules")
            | Some("target")
            | Some(".dart_tool")
            | Some("build")
            | Some("dist")
            | Some("out")
    )
}
