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
        let ra_bin = self.get_rust_analyzer_command();
        tracing::info!("Checking rust-analyzer availability at: {}", ra_bin);
        match tokio::process::Command::new(&ra_bin).arg("--version").output().await {
            std::result::Result::Ok(output) => {
                let success = output.status.success();
                tracing::info!("rust-analyzer --version success: {}", success);
                Ok(success)
            },
            Err(e) => {
                tracing::warn!("Failed to executed rust-analyzer --version: {:?}", e);
                Ok(false)
            },
        }
    }

    async fn move_files(&self, file_map: Vec<(String, String)>, root_path: Option<&std::path::Path>) -> Result<()> {
        let ra_bin = self.get_rust_analyzer_command();
        let client = LspClient::new(&ra_bin);
        
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

impl RustDriver {
    fn get_rust_analyzer_command(&self) -> String {
        // 1. Check if in PATH
        if which::which("rust-analyzer").is_ok() {
            tracing::info!("Found rust-analyzer in PATH");
            return "rust-analyzer".to_string();
        }

        // 2. Check standard cargo bin location
        if let Some(home) = std::env::var_os("HOME") {
            let home_path = std::path::Path::new(&home);
            let cargo_bin = home_path.join(".cargo").join("bin").join("rust-analyzer");
            tracing::info!("Checking cargo_bin: {:?} (exists: {})", cargo_bin, cargo_bin.exists());
            if cargo_bin.exists() {
                return cargo_bin.to_string_lossy().to_string();
            }
        } else {
            tracing::warn!("HOME environment variable not set");
        }

        tracing::warn!("rust-analyzer not found in PATH or .cargo/bin");
        "rust-analyzer".to_string()
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
