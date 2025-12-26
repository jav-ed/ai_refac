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

    async fn move_file(&self, source: &str, target: &str) -> Result<()> {
        let script_path = "scripts/python_refactor.py";
        let python_bin = ".venv/bin/python";
        
        let bin = if std::path::Path::new(python_bin).exists() { python_bin } else { "python3" };

        let output = tokio::process::Command::new(bin)
            .arg(script_path)
            .arg(source)
            .arg(target)
            .output()
            .await?;

        if !output.status.success() {
             let error = String::from_utf8_lossy(&output.stderr);
             anyhow::bail!("Python refactor failed: {}", error);
        }

        tracing::info!("Python refactor output: {}", String::from_utf8_lossy(&output.stdout));
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
