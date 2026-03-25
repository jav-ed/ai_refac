use super::RefactorDriver;
use anyhow::{Ok, Result};
use async_trait::async_trait;

pub struct TypeScriptDriver;

impl TypeScriptDriver {
    fn get_bun_command(&self) -> String {
        // 1. Try generic "bun"
        if std::process::Command::new("bun")
            .arg("--version")
            .output()
            .is_ok()
        {
            return "bun".to_string();
        }

        // 2. Try User's Home (Linux/macOS)
        if let std::result::Result::Ok(home) = std::env::var("HOME") {
            let path = std::path::Path::new(&home).join(".bun/bin/bun");
            if path.exists() {
                return path.to_string_lossy().to_string();
            }
        }

        // 3. Fallback to generic
        "bun".to_string()
    }
}

#[async_trait]
impl RefactorDriver for TypeScriptDriver {
    fn lang(&self) -> &str {
        "typescript"
    }

    async fn check_availability(&self) -> Result<bool> {
        // Check if script exists first
        if super::resolve_resource_path("scripts/ts_refactor.ts").is_err() {
            tracing::warn!("TypeScript driver unavailable: 'scripts/ts_refactor.ts' not found.");
            return Ok(false);
        }

        let bun_cmd = self.get_bun_command();
        // Check if bun is available
        match tokio::process::Command::new(&bun_cmd)
            .arg("--version")
            .output()
            .await
        {
            std::result::Result::Ok(output) => {
                if !output.status.success() {
                    tracing::warn!(
                        "Bun availability check failed. Stderr: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    return Ok(false);
                }
                Ok(true)
            }
            Err(e) => {
                tracing::warn!(
                    "Bun availability check command failed to spawn ('{}'): {}",
                    bun_cmd,
                    e
                );
                Ok(false)
            }
        }
    }

    async fn move_files(
        &self,
        file_map: Vec<(String, String)>,
        root_path: Option<&std::path::Path>,
    ) -> Result<()> {
        let script_path = super::resolve_resource_path("scripts/ts_refactor.ts")?;
        let bun_cmd = self.get_bun_command();

        // Ensure ts-morph is installed — bun install is idempotent and fast when up to date.
        let script_dir = script_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Could not determine scripts directory"))?;
        let ts_morph_dir = script_dir.join("node_modules").join("ts-morph");
        if !ts_morph_dir.exists() {
            tracing::info!("ts-morph not found in {:?}, running bun install...", script_dir);
            let install = tokio::process::Command::new(&bun_cmd)
                .arg("install")
                .current_dir(script_dir)
                .output()
                .await?;
            if !install.status.success() {
                anyhow::bail!(
                    "bun install failed in {:?}: {}",
                    script_dir,
                    String::from_utf8_lossy(&install.stderr)
                );
            }
        }

        let payload = serde_json::to_string(&file_map)?;

        // Call the script using found bun
        let mut cmd = tokio::process::Command::new(&bun_cmd);
        cmd.arg(script_path).arg("batch").arg(&payload);

        if let Some(r) = root_path {
            cmd.arg(r.to_string_lossy().to_string());
        }

        let output = tokio::time::timeout(
            std::time::Duration::from_secs(300),
            cmd.output(),
        )
        .await
        .map_err(|_| anyhow::anyhow!(
            "TypeScript refactor timed out after 5 minutes. \
             The project may be too large or bun/ts-morph hung. \
             Try passing --project-path to the package root (the folder with tsconfig.json), \
             not the monorepo root."
        ))??;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("TS-Morph batch stderr: {}", stderr);
            anyhow::bail!("TS-Morph batch failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::info!("TS-Morph batch output: {}", stdout);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Environment probe only — run with `cargo test -- --ignored` to verify bun is installed.
    #[tokio::test]
    #[ignore]
    async fn test_ts_availability() -> Result<()> {
        let driver = TypeScriptDriver;
        assert!(driver.check_availability().await?);
        Ok(())
    }

    /// Verifies that moving a directory rewrites import paths in files outside the moved folder.
    #[tokio::test]
    async fn test_ts_directory_move_updates_external_imports() -> Result<()> {
        let driver = TypeScriptDriver;
        if !driver.check_availability().await? {
            return Ok(());
        }

        let tmp = tempfile::tempdir()?;
        let root = tmp.path();

        // Minimal tsconfig so ts-morph loads the project properly
        tokio::fs::write(
            root.join("tsconfig.json"),
            r#"{"compilerOptions":{"target":"es2020","module":"commonjs"},"include":["src/**/*"]}"#,
        ).await?;

        tokio::fs::create_dir_all(root.join("src/utils")).await?;

        // src/utils/format.ts — inside the folder being moved
        tokio::fs::write(
            root.join("src/utils/format.ts"),
            "export function fmt(s: string) { return s.trim(); }\n",
        ).await?;

        // src/app.ts — outside, imports from utils/
        tokio::fs::write(
            root.join("src/app.ts"),
            "import { fmt } from \"./utils/format\";\nconsole.log(fmt(\"hi\"));\n",
        ).await?;

        // Move src/utils → src/helpers
        let result = driver
            .move_files(
                vec![(
                    root.join("src/utils").to_string_lossy().into_owned(),
                    root.join("src/helpers").to_string_lossy().into_owned(),
                )],
                Some(root),
            )
            .await;

        assert!(result.is_ok(), "Directory move failed: {:?}", result.err());
        assert!(!root.join("src/utils").exists(), "src/utils should be gone");
        assert!(root.join("src/helpers").exists(), "src/helpers should exist");
        assert!(root.join("src/helpers/format.ts").exists(), "file inside moved dir should exist");

        let app = tokio::fs::read_to_string(root.join("src/app.ts")).await?;
        assert!(
            app.contains("./helpers/format") || app.contains("helpers/format"),
            "external import was not updated after directory move — got:\n{app}"
        );

        Ok(())
    }

    /// Verifies that moving a TS file also rewrites import paths in files that imported it.
    /// This is the core value prop — previously there was no test for this.
    #[tokio::test]
    async fn test_ts_move_updates_imports() -> Result<()> {
        let driver = TypeScriptDriver;
        if !driver.check_availability().await? {
            return Ok(());
        }

        let tmp = tempfile::tempdir()?;
        let root = tmp.path();

        // lib.ts — the file that will be moved
        tokio::fs::write(root.join("lib.ts"), "export const greeting = \"hello\";").await?;

        // consumer.ts — imports from lib.ts, its import path must be updated after the move
        tokio::fs::write(
            root.join("consumer.ts"),
            "import { greeting } from \"./lib\";\nconsole.log(greeting);\n",
        )
        .await?;

        // Move lib.ts → utils/lib.ts
        tokio::fs::create_dir(root.join("utils")).await?;
        let result = driver
            .move_files(
                vec![(
                    root.join("lib.ts").to_string_lossy().into_owned(),
                    root.join("utils/lib.ts").to_string_lossy().into_owned(),
                )],
                Some(root),
            )
            .await;

        assert!(result.is_ok(), "Move failed: {:?}", result.err());
        assert!(!root.join("lib.ts").exists(), "source should be gone");
        assert!(root.join("utils/lib.ts").exists(), "target should exist");

        let consumer = tokio::fs::read_to_string(root.join("consumer.ts")).await?;
        assert!(
            consumer.contains("./utils/lib") || consumer.contains("utils/lib"),
            "import path was not updated in consumer.ts — got:\n{consumer}"
        );

        Ok(())
    }

}
