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
