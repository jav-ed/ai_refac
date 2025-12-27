use anyhow::Result;
use async_trait::async_trait;

/// Represents a generic refactoring driver.
/// 
/// # Internal Docs
/// Each language (TS, Python, Rust) will implement this trait.
/// The `RefactorServer` will hold a collection of these drivers and dispatch
/// based on file extension.
#[async_trait]
pub trait RefactorDriver: Send + Sync {
    /// Returns the language identifier this driver handles (e.g., "typescript").
    fn lang(&self) -> &str;

    /// Checks if the driver is available (e.g., is the underlying tool installed?).
    async fn check_availability(&self) -> Result<bool>;

    /// Executes a batch move/rename operation.
    /// 
    /// # Arguments
    /// * `file_map` - A list of (source, target) relative paths.
    async fn move_files(&self, file_map: Vec<(String, String)>) -> Result<()>;
}

// Submodules for specific drivers (to be implemented)
pub mod python;
pub mod python_rope;
pub mod python_pyrefly;
pub mod typescript;
pub mod lsp_client;
pub mod rust;
pub mod go;
pub mod dart;

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
        if dir.parent().is_none() { break; }
        current_dir = dir.parent();
    }
    
    // Fallback: Check CWD
    let cwd_res = std::env::current_dir()?.join(relative_path);
    if cwd_res.exists() {
        return Ok(std::fs::canonicalize(cwd_res)?);
    }

    anyhow::bail!("Could not find resource: {}", relative_path)
}
