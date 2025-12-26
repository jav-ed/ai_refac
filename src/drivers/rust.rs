use super::RefactorDriver;
use anyhow::{Result, Ok};
use async_trait::async_trait;
use super::lsp_client::LspClient;

pub struct RustDriver {
    client: LspClient,
}

impl RustDriver {
    pub fn new() -> Self {
        Self {
            client: LspClient::new("rust-analyzer"),
        }
    }
}

#[async_trait]
impl RefactorDriver for RustDriver {
    fn lang(&self) -> &str {
        "rust"
    }

    async fn check_availability(&self) -> Result<bool> {
        // rust-analyzer usually is in PATH
        match tokio::process::Command::new("rust-analyzer").arg("--version").output().await {
            std::result::Result::Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }

    async fn move_files(&self, file_map: Vec<(String, String)>) -> Result<()> {
        // rust-analyzer doesn't use standard "init" command like pyrefly might
        // It's a standard LSP.
        
        self.client.initialize_and_rename_files(&[], file_map.clone()).await?;
        
        // Perform file moves
        for (source, target) in file_map {
            if let Some(parent) = std::path::Path::new(&target).parent() {
                tokio::fs::create_dir_all(parent).await?;
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
    async fn test_rust_availability() -> Result<()> {
        let driver = RustDriver::new();
        let avail = driver.check_availability().await?;
        // Should be true since we installed it
        assert!(avail, "rust-analyzer should be available");
        Ok(())
    }
}
