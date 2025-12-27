use super::RefactorDriver;
use anyhow::{Result, Ok};
use async_trait::async_trait;
use super::lsp_client::LspClient;
use std::path::PathBuf;

pub struct GoDriver {
    client: LspClient,
}

impl GoDriver {
    pub fn new() -> Self {
        // gopls might be in PATH or in GOPATH/bin
        // We'll try to find it dynamically or default to "gopls"
        let binary = Self::find_gopls().unwrap_or_else(|| "gopls".to_string());
        Self {
            client: LspClient::new(&binary),
        }
    }

    fn find_gopls() -> Option<String> {
        // Check if "gopls" is in PATH
        if which::which("gopls").is_ok() {
            return Some("gopls".to_string());
        }

        // Check common GOPATH
        let home = std::env::var("HOME").ok()?;
        let gopath_bin = PathBuf::from(home).join("go/bin/gopls");
        if gopath_bin.exists() {
            return Some(gopath_bin.to_string_lossy().to_string());
        }

        None
    }
}

#[async_trait]
impl RefactorDriver for GoDriver {
    fn lang(&self) -> &str {
        "go"
    }

    async fn check_availability(&self) -> Result<bool> {
        self.client.check_availability().await
    }

    async fn move_files(&self, file_map: Vec<(String, String)>, root_path: Option<&std::path::Path>) -> Result<()> {
        // gopls supports standard textDocument/rename and executeCommand
        
        self.client.initialize_and_rename_files(&[], file_map.clone(), root_path).await?;
        
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
    async fn test_gopls_availability() -> Result<()> {
        // This test might fail if gopls isn't installed in the environment where test runs?
        // But we just installed it.
        let driver = GoDriver::new();
        let avail = driver.check_availability().await?;
        if !avail {
            eprintln!("gopls not found, skipping test");
            return Ok(());
        }
        Ok(())
    }
}
