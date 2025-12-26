use super::RefactorDriver;
use anyhow::{Result, Ok};
use async_trait::async_trait;

pub struct PythonDriver;

#[async_trait]
impl RefactorDriver for PythonDriver {
    fn lang(&self) -> &str {
        "python"
    }

    async fn check_availability(&self) -> Result<bool> {
        // TODO: Check if python and rope are installed
        Ok(true)
    }

    async fn move_file(&self, source: &str, target: &str) -> Result<()> {
        println!("PythonDriver: Moving {} -> {}", source, target);
        // TODO: Call actual rope script
        Ok(())
    }
}
