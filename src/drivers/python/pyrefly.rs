use super::super::{MovePreview, RefactorDriver, complete_filesystem_moves};
use crate::drivers::lsp::client::{LspClient, PendingChange, apply_pending_changes, summarize};
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
        let changes = self.plan(&file_map, root_path).await?;
        apply_pending_changes(changes).await?;

        complete_filesystem_moves(&file_map, root_path).await?;

        Ok(())
    }

    async fn plan_move(
        &self,
        file_map: Vec<(String, String)>,
        root_path: Option<&std::path::Path>,
    ) -> Result<MovePreview> {
        let changes = self.plan(&file_map, root_path).await?;
        let root = match root_path {
            Some(root) => std::path::absolute(root)?,
            None => std::env::current_dir()?,
        };
        let mut preview = MovePreview {
            moves: file_map
                .iter()
                .map(|(from, to)| (root.join(from), root.join(to)))
                .collect(),
            ..Default::default()
        };
        summarize(&[changes])?.apply_to(&mut preview, "Pyrefly server");
        Ok(preview)
    }
}

impl PyreflyDriver {
    /// The server's plan for the moves; nothing is written.
    async fn plan(
        &self,
        file_map: &[(String, String)],
        root_path: Option<&std::path::Path>,
    ) -> Result<Vec<PendingChange>> {
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
            .plan_file_renames(&["lsp"], file_map.to_vec(), root_path, "python", &["py"])
            .await
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

    // Real-server check of the Pyrefly path (the dispatcher normally prefers Rope):
    // run with `cargo test -- --ignored` where pyrefly is installed. A missing
    // server is a loud failure here, never a skip.
    #[tokio::test]
    #[ignore]
    async fn test_pyrefly_move_rewrites_importer() -> Result<()> {
        let driver = PyreflyDriver::new();
        assert!(
            driver.check_availability().await?,
            "pyrefly not found in .venv or PATH"
        );
        let dir = tempfile::tempdir()?;
        let root = dir.path();
        std::fs::create_dir_all(root.join("pkg"))?;
        std::fs::write(root.join("pkg/__init__.py"), "")?;
        std::fs::write(root.join("pkg/util.py"), "def helper():\n    return 1\n")?;
        std::fs::write(
            root.join("main.py"),
            "from pkg.util import helper\n\nprint(helper())\n",
        )?;

        driver
            .move_files(
                vec![(
                    root.join("pkg/util.py").to_string_lossy().into_owned(),
                    root.join("pkg/tools.py").to_string_lossy().into_owned(),
                )],
                Some(root),
            )
            .await?;

        assert!(root.join("pkg/tools.py").exists());
        assert!(!root.join("pkg/util.py").exists());
        let main = std::fs::read_to_string(root.join("main.py"))?;
        assert!(
            main.contains("from pkg.tools import helper"),
            "importer was not rewritten:\n{main}"
        );
        Ok(())
    }

    // Pyrefly is the fallback of the dispatcher, so its plan is checked on its
    // own: the plan names the importer and writes nothing.
    #[tokio::test]
    #[ignore]
    async fn test_pyrefly_plan_move_names_the_importer_and_writes_nothing() -> Result<()> {
        let driver = PyreflyDriver::new();
        assert!(
            driver.check_availability().await?,
            "pyrefly not found in .venv or PATH"
        );
        let dir = tempfile::tempdir()?;
        let root = dir.path();
        std::fs::create_dir_all(root.join("pkg"))?;
        std::fs::write(root.join("pkg/__init__.py"), "")?;
        std::fs::write(root.join("pkg/util.py"), "def helper():\n    return 1\n")?;
        let main = "from pkg.util import helper\n\nprint(helper())\n";
        std::fs::write(root.join("main.py"), main)?;

        let preview = driver
            .plan_move(
                vec![("pkg/util.py".to_string(), "pkg/tools.py".to_string())],
                Some(root),
            )
            .await?;

        assert_eq!(preview.moves.len(), 1);
        assert!(preview.edits.keys().any(|path| path.ends_with("main.py")));
        assert_eq!(std::fs::read_to_string(root.join("main.py"))?, main);
        assert!(root.join("pkg/util.py").exists());
        Ok(())
    }
}
