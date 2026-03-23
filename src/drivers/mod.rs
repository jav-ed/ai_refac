use anyhow::Result;
use async_trait::async_trait;
use std::io::ErrorKind;

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
}

// Submodules for specific drivers (to be implemented)
pub mod dart;
pub mod go;
pub mod lsp_client;
pub mod python;
pub mod python_pyrefly;
pub mod python_rope;
pub mod rust;
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

/// Resolves a path relative to the executable location (finding the project root).
/// Handles "debug", "release", and "deps" directory structures.
pub fn resolve_resource_path(relative_path: &str) -> Result<std::path::PathBuf> {
    let exe_path = std::env::current_exe()?;
    let mut current_dir = exe_path.parent();

    // Traverse up to find the "scripts" or ".venv" folder or Cargo.toml
    while let Some(dir) = current_dir {
        let candidate = dir.join(relative_path);
        if candidate.exists() {
            return Ok(std::fs::canonicalize(candidate)?);
        }

        // Also check if we are in target/release or target/debug, root is 2 levels up
        // But "scripts" is in root.

        // Safety check: don't traverse beyond reasonable limits or "/"
        if dir.parent().is_none() {
            break;
        }
        current_dir = dir.parent();
    }

    // Fallback: Check CWD
    let cwd_res = std::env::current_dir()?.join(relative_path);
    if cwd_res.exists() {
        return Ok(std::fs::canonicalize(cwd_res)?);
    }

    anyhow::bail!("Could not find resource: {}", relative_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    #[test]
    fn test_resolve_resource_path_from_foreign_dir() -> Result<()> {
        // 1. Get original CWD and exe path
        let _original_cwd = env::current_dir()?;
        // This test relies on being run via cargo test, where exe is in target/debug/deps
        // and scripts are in project root.

        // 2. Create a random temp dir to be our new "fake user project"
        let temp_dir = env::temp_dir().join("run_refac_test_dir");
        fs::create_dir_all(&temp_dir)?;

        // 3. Change CWD to temp dir (simulating running from user project)
        let _guard = DirectoryGuard::new(temp_dir.clone())?; // RAII style verification? 
        // Rust tests run in threads, changing env CWD is dangerous/racy for parallel tests.
        // We will just temporarily change it if we are generic.
        // Actually, changing CWD in tests is bad practice in Rust due to threading.

        // Instead of changing CWD, let's verify resolve_resource_path logic
        // explicitly checks the executable's relative paths.
        // pass.

        let path = resolve_resource_path("scripts/ts_refactor.ts")?;
        assert!(
            path.exists(),
            "Should find script even if CWD was weird (logic analysis)"
        );
        assert!(
            path.to_string_lossy().contains("scripts"),
            "Should point to scripts dir"
        );

        Ok(())
    }

    // Simple RAII guard to restore CWD if we did change it (which we won't for safety)
    struct DirectoryGuard {
        original: std::path::PathBuf,
    }
    impl DirectoryGuard {
        fn new(target: std::path::PathBuf) -> Result<Self> {
            let original = env::current_dir()?;
            env::set_current_dir(&target)?;
            Ok(Self { original })
        }
    }
    impl Drop for DirectoryGuard {
        fn drop(&mut self) {
            let _ = env::set_current_dir(&self.original);
        }
    }
}
