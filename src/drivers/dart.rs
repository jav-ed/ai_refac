use super::RefactorDriver;
use anyhow::{Result, Ok};
use async_trait::async_trait;
use super::lsp_client::LspClient;

pub struct DartDriver {
    client: LspClient,
}

impl DartDriver {
    pub fn new() -> Self {
        Self {
            client: LspClient::new("dart"),
        }
    }
}

#[async_trait]
impl RefactorDriver for DartDriver {
    fn lang(&self) -> &str {
        "dart"
    }

    async fn check_availability(&self) -> Result<bool> {
        match tokio::process::Command::new("dart").arg("--version").output().await {
            std::result::Result::Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }

    async fn move_file(&self, source: &str, target: &str) -> Result<()> {
        // The command to start LSP is `dart language-server`
        self.client.initialize_and_rename(&["language-server"], source, target).await?;
        
        // Perform file move
        tokio::fs::rename(source, target).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dart_availability() -> Result<()> {
        let driver = DartDriver::new();
        let avail = driver.check_availability().await?;
        assert!(avail, "dart should be available");
        Ok(())
    }
}
