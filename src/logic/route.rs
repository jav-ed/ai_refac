//! Choosing the language driver for each requested path.

use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const TYPESCRIPT_EXTENSIONS: &[&str] = &["ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs"];

/// The language that handles `source`, or `None` for a file refac does not
/// support (it is reported as skipped). A directory is routed by its content
/// and is an error when no supported language owns it.
pub fn language_of(source: &str, root: Option<&Path>) -> Result<Option<&'static str>> {
    let path = Path::new(source);
    let resolved = resolve(path, root);
    if resolved.is_dir() {
        if has_direct_typescript_file(&resolved) {
            return Ok(Some("typescript"));
        }
        if contains_kotlin_file(&resolved)? {
            return Ok(Some("kotlin"));
        }
        bail!(
            "Directory moves are only supported for TypeScript/JavaScript and Kotlin projects. \
             '{}' does not appear to contain TypeScript, JavaScript or Kotlin files.",
            source
        );
    }

    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("");
    Ok(match extension {
        "md" => Some("markdown"),
        "py" => Some("python"),
        "rs" => Some("rust"),
        "go" => Some("go"),
        "dart" => Some("dart"),
        "kt" => Some("kotlin"),
        extension if TYPESCRIPT_EXTENSIONS.contains(&extension) => Some("typescript"),
        _ => None,
    })
}

fn resolve(path: &Path, root: Option<&Path>) -> PathBuf {
    match root {
        Some(root) if !path.is_absolute() => root.join(path),
        _ => path.to_path_buf(),
    }
}

fn has_direct_typescript_file(dir: &Path) -> bool {
    std::fs::read_dir(dir).ok().is_some_and(|entries| {
        entries.flatten().any(|entry| {
            entry
                .path()
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| TYPESCRIPT_EXTENSIONS.contains(&extension))
        })
    })
}

/// Kotlin sources sit in package folders, so the first `.kt` file is usually
/// several levels down.
fn contains_kotlin_file(dir: &Path) -> Result<bool> {
    for entry in WalkDir::new(dir) {
        let entry = entry.with_context(|| format!("Cannot read inside {}", dir.display()))?;
        if entry.path().extension().and_then(|e| e.to_str()) == Some("kt") {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests;
