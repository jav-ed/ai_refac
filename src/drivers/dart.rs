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

    async fn move_files(&self, file_map: Vec<(String, String)>, root_path: Option<&std::path::Path>) -> Result<()> {
        // The command to start LSP is `dart language-server`
        self.client.initialize_and_rename_files(&["language-server"], file_map.clone(), root_path).await?;
        
        // Perform file moves
        for (source, target) in file_map {
            let (source_abs, target_abs) = if let Some(root) = root_path {
                (root.join(&source), root.join(&target))
            } else {
                (std::path::PathBuf::from(&source), std::path::PathBuf::from(&target))
            };

            if let Some(parent) = target_abs.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::rename(source_abs, target_abs).await?;
        }

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
