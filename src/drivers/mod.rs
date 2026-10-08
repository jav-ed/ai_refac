use anyhow::Result;
use async_trait::async_trait;
use std::io::ErrorKind;

pub mod preview;
pub use preview::MovePreview;

/// Represents a generic refactoring driver.
///
/// # Internal Docs
/// Each language (TS, Python, Rust) will implement this trait.
/// The CLI orchestration layer dispatches to these drivers based on file
/// extension.
#[async_trait]
pub trait RefactorDriver: Send + Sync {
    /// Returns the language identifier this driver handles (e.g., "typescript").
    fn lang(&self) -> &str;

    /// Checks if the driver is available (e.g., is the underlying tool installed?).
    async fn check_availability(&self) -> Result<bool>;

    /// Executes a batch move/rename operation.
    ///
    /// # Arguments
    /// * `file_map` - A list of (source, target) paths.
    /// * `root_path` - Optional project root path for resolving relative paths.
    async fn move_files(
        &self,
        file_map: Vec<(String, String)>,
        root_path: Option<&std::path::Path>,
    ) -> Result<()>;

    /// Plans the same move as `move_files` and changes nothing the caller can
    /// see: the paths that would move, the edits per file, and the notes. It
    /// refuses what the real move refuses. Required, not defaulted, so a new
    /// driver cannot forget to support `--dry-run`.
    async fn plan_move(
        &self,
        file_map: Vec<(String, String)>,
        root_path: Option<&std::path::Path>,
    ) -> Result<MovePreview>;

    /// Like `move_files`, and returns what the caller should be told besides
    /// success: a rename that rode along, names left behind in files the driver
    /// does not edit. Drivers with nothing to add keep this default.
    async fn move_files_with_notes(
        &self,
        file_map: Vec<(String, String)>,
        root_path: Option<&std::path::Path>,
    ) -> Result<Vec<String>> {
        self.move_files(file_map, root_path).await?;
        Ok(Vec::new())
    }
}

// Submodules for specific drivers (to be implemented)
pub mod dart;
pub mod go;
pub mod kotlin;
pub mod lsp;
pub mod markdown;
pub mod python;
pub mod rust;
pub mod symbol;
pub mod typescript;

pub async fn complete_filesystem_moves(
    file_map: &[(String, String)],
    root_path: Option<&std::path::Path>,
) -> Result<()> {
    for (source, target) in file_map {
        let (source_abs, target_abs) = if let Some(root) = root_path {
            (root.join(source), root.join(target))
        } else {
            (
                std::path::PathBuf::from(source),
                std::path::PathBuf::from(target),
            )
        };

        if let Some(parent) = target_abs.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        match tokio::fs::rename(&source_abs, &target_abs).await {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {
                if target_abs.exists() {
                    tracing::info!(
                        "Skipping filesystem move because target already exists: {:?} -> {:?}",
                        source_abs,
                        target_abs
                    );
                    continue;
                }

                return Err(error.into());
            }
            Err(error) => return Err(error.into()),
        }
    }

    Ok(())
}

/// Finds a file that ships with the checkout (`scripts/ts_refactor.ts`,
/// `pyrefly.toml`, `.venv/...`), looking in three places and naming all of them
/// when it is nowhere:
///
/// 1. next to the executable and in every folder above it, which finds the
///    checkout for `target/release/refac` and for a symlink to it;
/// 2. the checkout this binary was built from, which is where `scripts/` is
///    when the binary was installed elsewhere (`cargo install --path .`) or
///    built into another `CARGO_TARGET_DIR`;
/// 3. the current directory.
pub fn resolve_resource_path(relative_path: &str) -> Result<std::path::PathBuf> {
    let mut looked = Vec::new();

    let exe_path = std::env::current_exe()?;
    let mut current_dir = exe_path.parent();
    while let Some(dir) = current_dir {
        let candidate = dir.join(relative_path);
        if candidate.exists() {
            return Ok(std::fs::canonicalize(candidate)?);
        }
        current_dir = dir.parent();
    }
    looked.push(format!(
        "next to {} and in the folders above it",
        exe_path.display()
    ));

    let built_from = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    if built_from.exists() {
        return Ok(std::fs::canonicalize(built_from)?);
    }
    looked.push(format!(
        "in the checkout this binary was built from ({})",
        env!("CARGO_MANIFEST_DIR")
    ));

    let cwd = std::env::current_dir()?;
    let in_cwd = cwd.join(relative_path);
    if in_cwd.exists() {
        return Ok(std::fs::canonicalize(in_cwd)?);
    }
    looked.push(format!("in the current directory ({})", cwd.display()));

    anyhow::bail!(
        "Could not find {relative_path}. Looked {}. refac keeps its helper files in its checkout; run it from a build of that checkout, or keep the checkout where it was built.",
        looked.join(", ")
    )
}

#[cfg(test)]
mod tests;
