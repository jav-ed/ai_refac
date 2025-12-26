use super::RefactorDriver;
use anyhow::{Result, Ok};
use async_trait::async_trait;

pub struct RustDriver;

#[async_trait]
impl RefactorDriver for RustDriver {
    fn lang(&self) -> &str {
        "rust"
    }

    async fn check_availability(&self) -> Result<bool> {
        // TODO: Check if rust-analyzer is installed
        Ok(true)
    }

    async fn move_file(&self, source: &str, target: &str) -> Result<()> {
        println!("RustDriver: Moving {} -> {}", source, target);
        // TODO: Call rust-analyzer or cargo fix
        Ok(())
    }
}
