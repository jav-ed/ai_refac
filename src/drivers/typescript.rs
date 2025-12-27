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
        // Check if node is available
        match tokio::process::Command::new("node").arg("--version").output().await {
            std::result::Result::Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }

    async fn move_files(&self, file_map: Vec<(String, String)>) -> Result<()> {
        let script_path = super::resolve_resource_path("scripts/ts_refactor.js")?;
        let payload = serde_json::to_string(&file_map)?;
        
        // Call the script once with "batch" command
        let output = tokio::process::Command::new("node")
            .arg(script_path)
            .arg("batch")
            .arg(&payload)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("TS-Morph batch stderr: {}", stderr);
            anyhow::bail!("TS-Morph batch failed: {}", stderr);
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::info!("TS-Morph batch output: {}", stdout);

        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ts_availability() -> Result<()> {
        let driver = TypeScriptDriver;
        assert!(driver.check_availability().await?);
        Ok(())
    }

    #[tokio::test]
    async fn test_ts_move_e2e() -> Result<()> {
        let driver = TypeScriptDriver;
        if !driver.check_availability().await? {
            // Skip if node/script not ready (e.g. CI without node?)
            // For now, we expect it to work in this environment
            return Ok(());
        }

        let source = "test_e2e_source.ts";
        let target = "test_e2e_target.ts";

        // Clean up pre-existing
        let _ = tokio::fs::remove_file(source).await;
        let _ = tokio::fs::remove_file(target).await;

        tokio::fs::write(source, "export const x = 1;").await?;

        // Run move
        let result = driver.move_files(vec![(source.to_string(), target.to_string())]).await;
        
        // Assert success
        assert!(result.is_ok(), "Move failed: {:?}", result.err());
        assert!(!std::path::Path::new(source).exists());
        assert!(std::path::Path::new(target).exists());

        // Cleanup
        let _ = tokio::fs::remove_file(target).await;
        Ok(())
    }
}
