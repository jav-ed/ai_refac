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

    /// Executes a move/rename operation.
    /// 
    /// # Arguments
    /// * `source` - The source file path (relative).
    /// * `target` - The target file path (relative).
    async fn move_file(&self, source: &str, target: &str) -> Result<()>;
}

// Submodules for specific drivers (to be implemented)
pub mod python;
pub mod python_rope;
pub mod python_pyrefly;
pub mod typescript;
pub mod lsp_client;
pub mod rust;
