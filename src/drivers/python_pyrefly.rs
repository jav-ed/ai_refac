use super::RefactorDriver;
use anyhow::{Result, Ok};
use async_trait::async_trait;
use super::lsp_client::LspClient;

pub struct PyreflyDriver {
    client: LspClient,
}

impl PyreflyDriver {
    pub fn new() -> Self {
        Self {
            client: LspClient::new(".venv/bin/pyrefly"),
        }
    }
}

#[async_trait]
impl RefactorDriver for PyreflyDriver {
    fn lang(&self) -> &str {
        "python"
    }

    async fn check_availability(&self) -> Result<bool> {
        self.client.check_availability().await
    }

    async fn move_files(&self, file_map: Vec<(String, String)>) -> Result<()> {
        let bin = ".venv/bin/pyrefly";
        
        // Ensure init
        if !std::path::Path::new("pyrefly.toml").exists() {
             let _ = tokio::process::Command::new(bin).arg("init").output().await;
        }

        // Use generic client with batch support
        self.client.initialize_and_rename_files(&["lsp"], file_map.clone()).await?;
        
        // Perform file moves
        for (source, target) in file_map {
            if let Some(parent) = std::path::Path::new(&target).parent() {
                if !parent.exists() {
                     tokio::fs::create_dir_all(parent).await?;
                }
            }
            tokio::fs::rename(source, target).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pyrefly_availability() -> Result<()> {
        let driver = PyreflyDriver::new();
        let avail = driver.check_availability().await?;
        assert!(avail, "Pyrefly should be available in .venv");
        Ok(())
    }
}
