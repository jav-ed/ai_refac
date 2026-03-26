use super::super::RefactorDriver;
use anyhow::{Ok, Result};
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
            return Ok(tokio::process::Command::new("python3")
                .arg("-c")
                .arg("import rope")
                .output()
                .await?
                .status
                .success());
        }

        let output = tokio::process::Command::new(python_bin)
            .arg("-c")
            .arg("import rope")
            .output()
            .await?;
        Ok(output.status.success())
    }

    async fn move_files(
        &self,
        file_map: Vec<(String, String)>,
        root_path: Option<&std::path::Path>,
    ) -> Result<()> {
        let script_path = super::super::resolve_resource_path("scripts/python_refactor.py")?;

        // Rope requires paths relative to the project root.
        // Strip the root prefix from absolute paths so the script gets relative paths.
        let normalized: Vec<(String, String)> = if let Some(root) = root_path {
            file_map
                .into_iter()
                .map(|(src, tgt)| {
                    let src_rel = std::path::Path::new(&src)
                        .strip_prefix(root)
                        .map(|p| p.to_string_lossy().into_owned())
                        .unwrap_or(src);
                    let tgt_rel = std::path::Path::new(&tgt)
                        .strip_prefix(root)
                        .map(|p| p.to_string_lossy().into_owned())
                        .unwrap_or(tgt);
                    (src_rel, tgt_rel)
                })
                .collect()
        } else {
            file_map
        };

        let payload = serde_json::to_string(&normalized)?;

        // Determine python binary
        let mut python_bin = "python3".to_string();
        if std::path::Path::new(".venv/bin/python").exists() {
            python_bin = ".venv/bin/python".to_string();
        }

        let mut cmd = tokio::process::Command::new(&python_bin);
        cmd.arg(script_path).arg("batch").arg(&payload);

        if let Some(r) = root_path {
            cmd.arg("--root").arg(r.to_string_lossy().to_string());
        }

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("Rope batch stderr: {}", stderr);
            anyhow::bail!("Rope batch failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::info!("Rope batch output: {}", stdout);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rope_move_updates_imports() -> Result<()> {
        let driver = RopeDriver;
        if !driver.check_availability().await? {
            eprintln!("rope not available, skipping test");
            return Ok(());
        }

        let temp_dir = tempfile::Builder::new()
            .prefix("refac-rope-test-")
            .tempdir_in(std::env::temp_dir())?;

        std::fs::create_dir_all(temp_dir.path().join("pkg/utils"))?;
        std::fs::create_dir_all(temp_dir.path().join("pkg/core"))?;
        std::fs::write(temp_dir.path().join("pkg/__init__.py"), "")?;
        std::fs::write(temp_dir.path().join("pkg/utils/__init__.py"), "")?;
        std::fs::write(temp_dir.path().join("pkg/core/__init__.py"), "")?;
        std::fs::write(
            temp_dir.path().join("pkg/utils/helpers.py"),
            "def helper(): return 42\n",
        )?;
        std::fs::write(
            temp_dir.path().join("pkg/core/app.py"),
            "from pkg.utils.helpers import helper\n\nresult = helper()\n",
        )?;

        driver
            .move_files(
                vec![(
                    "pkg/utils/helpers.py".to_string(),
                    "pkg/core/helpers.py".to_string(),
                )],
                Some(temp_dir.path()),
            )
            .await?;

        assert!(
            temp_dir.path().join("pkg/core/helpers.py").exists(),
            "file should exist at new location"
        );
        assert!(
            !temp_dir.path().join("pkg/utils/helpers.py").exists(),
            "file should be gone from old location"
        );

        let app = std::fs::read_to_string(temp_dir.path().join("pkg/core/app.py"))?;
        assert!(
            app.contains("from pkg.core.helpers import helper"),
            "Rope did not update import in app.py — got:\n{app}"
        );

        Ok(())
    }
}
