use super::super::RefactorDriver;
use super::super::complete_filesystem_moves;
use super::super::lsp_client::LspClient;
use anyhow::{Ok, Result};
use async_trait::async_trait;

pub struct PyreflyDriver {
    client: LspClient,
    bin_path: String,
}

impl PyreflyDriver {
    pub fn new() -> Self {
        let bin_path = super::super::resolve_resource_path(".venv/bin/pyrefly")
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".venv/bin/pyrefly".to_string());

        Self {
            client: LspClient::new(&bin_path),
            bin_path,
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

    async fn move_files(
        &self,
        file_map: Vec<(String, String)>,
        root_path: Option<&std::path::Path>,
    ) -> Result<()> {
        let bin = &self.bin_path;

        // Ensure init - check if pyrefly.toml exists in the project root.
        // We resolve it relative to binary location
        // NOTE: If root_path is provided (user project), pyrefly might expect initialization there?
        // But we are using the bundled pyrefly. For now keep as is.
        let config_path = super::super::resolve_resource_path("pyrefly.toml")
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "pyrefly.toml".to_string());

        if !std::path::Path::new(&config_path).exists() {
            // We can't easily run "init" effectively if we aren't in the right dir,
            // but assuming we are using the bundled pyrefly, it might expect to be initialized.
            // For now, let's try to run init if missing, using the resolved binary.
            let _ = tokio::process::Command::new(bin).arg("init").output().await;
        }

        // Use generic client with batch support
        self.client
            .initialize_and_rename_files(
                &["lsp"],
                file_map.clone(),
                root_path,
                Some("python"),
                &["py"],
            )
            .await?;

        complete_filesystem_moves(&file_map, root_path).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Environment probe only — run with `cargo test -- --ignored` to verify pyrefly is installed.
    #[tokio::test]
    #[ignore]
    async fn test_pyrefly_availability() -> Result<()> {
        let driver = PyreflyDriver::new();
        let avail = driver.check_availability().await?;
        assert!(avail, "pyrefly not found in .venv or PATH");
        Ok(())
    }
}
