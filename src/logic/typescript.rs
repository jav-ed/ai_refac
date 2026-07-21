use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub const MAX_FILES_PER_MOVE: usize = 30;

fn is_typescript_or_javascript(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("ts" | "tsx" | "js" | "jsx")
    )
}

fn resolve_source(source: &str, root: Option<&Path>) -> PathBuf {
    let path = Path::new(source);
    if path.is_absolute() {
        path.to_path_buf()
    } else if let Some(project_root) = root {
        project_root.join(path)
    } else {
        path.to_path_buf()
    }
}

/// Count distinct source files before moving anything. Directory request counts
/// alone hide the actual ts-morph workload and make the memory limit meaningless.
pub fn count_source_files(file_map: &[(String, String)], root: Option<&Path>) -> Result<usize> {
    let mut files = HashSet::new();

    for (source, _) in file_map {
        let resolved = resolve_source(source, root);
        if resolved.is_file() {
            if is_typescript_or_javascript(&resolved) {
                files.insert(resolved);
            }
            continue;
        }

        for entry in WalkDir::new(&resolved) {
            let entry = entry?;
            if entry.file_type().is_file() && is_typescript_or_javascript(entry.path()) {
                files.insert(entry.path().to_path_buf());
            }
        }
    }

    Ok(files.len())
}
