//! Choosing the language driver for each requested path.

use crate::drivers::RefactorDriver;
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const TYPESCRIPT_EXTENSIONS: &[&str] = &["ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs"];

/// Files Markdown points at that no language backend owns: moving one moves
/// the file and fixes the Markdown links to it, in the same pass as the
/// Markdown files of the request. Formats that code reads or
/// imports (`json`, `yaml`, `toml`, `css`, `html`, `txt`, `csv`) are not here,
/// because a move would leave those references broken without a word.
const ASSET_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "svg", "webp", "avif", "ico", "bmp", "tif", "tiff", "heic", "pdf",
    "mp4", "webm", "mov", "mp3", "wav", "ogg", "flac", "woff", "woff2", "ttf", "otf", "zip",
];

/// A folder that holds any of these cannot be moved as a document folder:
/// the references in code would be left behind. The languages refac supports
/// are listed with the common others.
const CODE_EXTENSIONS: &[&str] = &[
    "ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs", "py", "pyi", "rs", "go", "dart", "kt",
    "kts", "java", "scala", "c", "h", "cc", "cpp", "hpp", "cs", "rb", "php", "swift", "sh", "bash",
    "lua", "vue", "svelte", "gradle", "m", "mm",
];

/// The driver that moves the files of `lang` (a name `language_of` returns).
pub fn driver_for(lang: &str) -> Result<Box<dyn RefactorDriver>> {
    let driver: Box<dyn RefactorDriver> = match lang {
        "markdown" => Box::new(crate::drivers::markdown::MarkdownDriver::new()),
        "python" => Box::new(crate::drivers::python::PythonDriver::new()),
        "typescript" => Box::new(crate::drivers::typescript::TypeScriptDriver),
        "rust" => Box::new(crate::drivers::rust::RustDriver::new()),
        "go" => Box::new(crate::drivers::go::GoDriver::new()),
        "dart" => Box::new(crate::drivers::dart::DartDriver::new()),
        "kotlin" => Box::new(crate::drivers::kotlin::KotlinDriver),
        _ => bail!("Unsupported language: {}", lang),
    };
    Ok(driver)
}

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
        if is_document_folder(&resolved)? {
            return Ok(Some("markdown"));
        }
        bail!(
            "Directory moves are supported for TypeScript/JavaScript and Kotlin projects and for \
             folders that hold only Markdown and asset files. '{}' holds none of those, or holds \
             code of another language.",
            source
        );
    }

    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("");
    Ok(match extension {
        extension if is_markdown_extension(extension) => Some("markdown"),
        extension if ASSET_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str()) => {
            Some("markdown")
        }
        "py" => Some("python"),
        "rs" => Some("rust"),
        "go" => Some("go"),
        "dart" => Some("dart"),
        "kt" => Some("kotlin"),
        extension if TYPESCRIPT_EXTENSIONS.contains(&extension) => Some("typescript"),
        _ => None,
    })
}

fn is_markdown_extension(extension: &str) -> bool {
    matches!(
        extension.to_ascii_lowercase().as_str(),
        "md" | "markdown" | "mdx"
    )
}

/// A folder of Markdown files and assets: at least one of them, and no code
/// (an empty folder is not one, and a folder with code would leave the
/// references in that code behind).
fn is_document_folder(dir: &Path) -> Result<bool> {
    let mut documents = false;
    for entry in WalkDir::new(dir) {
        let entry = entry.with_context(|| format!("Cannot read inside {}", dir.display()))?;
        let Some(extension) = entry.path().extension().and_then(|e| e.to_str()) else {
            continue;
        };
        let extension = extension.to_ascii_lowercase();
        if CODE_EXTENSIONS.contains(&extension.as_str()) {
            return Ok(false);
        }
        documents |=
            is_markdown_extension(&extension) || ASSET_EXTENSIONS.contains(&extension.as_str());
    }
    Ok(documents)
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
