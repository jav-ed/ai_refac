use super::RefactorDriver;
use anyhow::{Ok, Result};
use async_trait::async_trait;

pub struct TypeScriptDriver;

impl TypeScriptDriver {
    fn get_bun_command(&self) -> String {
        // 1. Try generic "bun"
        if std::process::Command::new("bun")
            .arg("--version")
            .output()
            .is_ok()
        {
            return "bun".to_string();
        }

        // 2. Try User's Home (Linux/macOS)
        if let std::result::Result::Ok(home) = std::env::var("HOME") {
            let path = std::path::Path::new(&home).join(".bun/bin/bun");
            if path.exists() {
                return path.to_string_lossy().to_string();
            }
        }

        // 3. Fallback to generic
        "bun".to_string()
    }
}

#[async_trait]
impl RefactorDriver for TypeScriptDriver {
    fn lang(&self) -> &str {
        "typescript"
    }

    async fn check_availability(&self) -> Result<bool> {
        // Check if script exists first
        if super::resolve_resource_path("scripts/ts_refactor.ts").is_err() {
            tracing::warn!("TypeScript driver unavailable: 'scripts/ts_refactor.ts' not found.");
            return Ok(false);
        }

        let bun_cmd = self.get_bun_command();
        // Check if bun is available
        match tokio::process::Command::new(&bun_cmd)
            .arg("--version")
            .output()
            .await
        {
            std::result::Result::Ok(output) => {
                if !output.status.success() {
                    tracing::warn!(
                        "Bun availability check failed. Stderr: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    return Ok(false);
                }
                Ok(true)
            }
            Err(e) => {
                tracing::warn!(
                    "Bun availability check command failed to spawn ('{}'): {}",
                    bun_cmd,
                    e
                );
                Ok(false)
            }
        }
    }

    async fn move_files(
        &self,
        file_map: Vec<(String, String)>,
        root_path: Option<&std::path::Path>,
    ) -> Result<()> {
        let script_path = super::resolve_resource_path("scripts/ts_refactor.ts")?;
        let payload = serde_json::to_string(&file_map)?;
        let bun_cmd = self.get_bun_command();

        // Call the script using found bun
        let mut cmd = tokio::process::Command::new(&bun_cmd);
        cmd.arg(script_path).arg("batch").arg(&payload);

        if let Some(r) = root_path {
            cmd.arg(r.to_string_lossy().to_string());
        }

        let output = cmd.output().await?;

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
        let result = driver
            .move_files(vec![(source.to_string(), target.to_string())], None)
            .await;

        // Assert success
        assert!(result.is_ok(), "Move failed: {:?}", result.err());
        assert!(!std::path::Path::new(source).exists());
        assert!(std::path::Path::new(target).exists());

        // Cleanup
        let _ = tokio::fs::remove_file(target).await;
        Ok(())
    }
}
