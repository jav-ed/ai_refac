use super::RefactorDriver;
use anyhow::{Result, Ok};
use async_trait::async_trait;

pub struct TypeScriptDriver;

#[async_trait]
impl RefactorDriver for TypeScriptDriver {
    fn lang(&self) -> &str {
        "typescript"
    }

    async fn check_availability(&self) -> Result<bool> {
        // TODO: Check if node and ts-morph are installed
        Ok(true)
    }

    async fn move_file(&self, source: &str, target: &str) -> Result<()> {
        println!("TypeScriptDriver: Moving {} -> {}", source, target);
        // TODO: Call actual node script
        Ok(())
    }
}
