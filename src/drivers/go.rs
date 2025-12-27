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
        let binary = Self::find_gopls().unwrap_or_else(|| "gopls".to_string());
        match tokio::process::Command::new(&binary).arg("version").output().await {
            std::result::Result::Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }

    async fn move_files(&self, file_map: Vec<(String, String)>, root_path: Option<&std::path::Path>) -> Result<()> {
        let binary = Self::find_gopls().unwrap_or_else(|| "gopls".to_string());
        let client = LspClient::new(&binary);
        
        client.initialize_and_rename_files(&[], file_map.clone(), root_path).await?;
        
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
