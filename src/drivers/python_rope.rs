use super::RefactorDriver;
use anyhow::{Result, Ok};
use async_trait::async_trait;

pub struct RopeDriver;

#[async_trait]
impl RefactorDriver for RopeDriver {
    fn lang(&self) -> &str {
        "python"
    }

    async fn check_availability(&self) -> Result<bool> {
        // Try local venv first, then fallback to system
        let python_bin = ".venv/bin/python";
        
        let path = std::path::Path::new(python_bin);
        if !path.exists() {
             return Ok(tokio::process::Command::new("python3").arg("-c").arg("import rope").output().await?.status.success());
        }

        let output = tokio::process::Command::new(python_bin)
            .arg("-c")
            .arg("import rope")
            .output()
            .await?;
        Ok(output.status.success())
    }


    async fn move_files(&self, file_map: Vec<(String, String)>, root_path: Option<&std::path::Path>) -> Result<()> {
        let script_path = super::resolve_resource_path("scripts/python_refactor.py")?;
        let payload = serde_json::to_string(&file_map)?;

        // Determine python binary
        let mut python_bin = "python3".to_string();
        if std::path::Path::new(".venv/bin/python").exists() {
            python_bin = ".venv/bin/python".to_string();
        }

        let mut cmd = tokio::process::Command::new(&python_bin);
        cmd.arg(script_path)
           .arg("batch")
           .arg(&payload);

        if let Some(r) = root_path {
            cmd.arg("--root").arg(r.to_string_lossy().to_string());
        }

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("Rope batch stderr: {}", stderr);
            anyhow::bail!("Rope batch failed: {}", stderr);
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::info!("Rope batch output: {}", stdout);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rope_availability() -> Result<()> {
        let driver = RopeDriver;
        // Just smoke test availability check
        let _ = driver.check_availability().await?;
        Ok(())
    }
}
