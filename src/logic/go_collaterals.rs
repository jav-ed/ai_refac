//! Files gopls moved that nobody asked to move. Go moves a whole package when
//! one of its files changes folder, so the answer of a Go move lists them.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// After a successful Go batch move, scan the target directories for .go files
/// that were not in the requested file map — these are collateral moves performed
/// by gopls as part of its package-level rename.
pub(super) fn detect_go_collaterals(
    file_map: &[(String, String)],
    root: Option<&Path>,
) -> Vec<PathBuf> {
    // Build the set of requested target absolute paths.
    let requested: HashSet<PathBuf> = file_map
        .iter()
        .map(|(_, target)| {
            let path = Path::new(target);
            match root {
                Some(root) if !path.is_absolute() => root.join(path),
                _ => path.to_path_buf(),
            }
        })
        .collect();

    // Collect the unique target directories.
    let target_dirs: HashSet<PathBuf> = requested
        .iter()
        .filter_map(|path| path.parent().map(Path::to_path_buf))
        .collect();

    // Any .go file in those directories that was not explicitly requested is collateral.
    let mut collaterals = Vec::new();
    for dir in &target_dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|extension| extension.to_str()) == Some("go")
                    && !requested.contains(&path)
                {
                    collaterals.push(path);
                }
            }
        }
    }
    collaterals.sort();
    collaterals
}
