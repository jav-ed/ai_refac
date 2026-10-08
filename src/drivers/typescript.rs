use super::{MovePreview, RefactorDriver};
use anyhow::{Context, Ok, Result};
use async_trait::async_trait;
use std::path::PathBuf;

mod dependencies;
mod process;
pub mod rename;

pub struct TypeScriptDriver;

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

        let bun_cmd = dependencies::bun_command();
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
        let stdout = run_helper(&file_map, root_path, false).await?;
        tracing::info!("TypeScript/Oxc batch output: {}", stdout);
        Ok(())
    }

    /// The helper plans the whole move before it writes anything; with
    /// `--dry-run` it prints that plan instead of applying it. The check that
    /// each rewritten specifier resolves needs the moved files, so it runs
    /// only on a real move.
    async fn plan_move(
        &self,
        file_map: Vec<(String, String)>,
        root_path: Option<&std::path::Path>,
    ) -> Result<MovePreview> {
        let stdout = run_helper(&file_map, root_path, true).await?;
        parse_plan(&stdout)
    }
}

/// Runs `scripts/ts_refactor.ts` on the pairs and returns what it printed.
async fn run_helper(
    file_map: &[(String, String)],
    root_path: Option<&std::path::Path>,
    dry_run: bool,
) -> Result<String> {
    let script_path = super::resolve_resource_path("scripts/ts_refactor.ts")?;
    let bun_cmd = dependencies::bun_command();

    // Install the locked parser/resolver dependencies when this checkout is new.
    let script_dir = script_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Could not determine scripts directory"))?;
    dependencies::ensure_installed(
        &bun_cmd,
        script_dir,
        &["oxc-parser", "oxc-resolver", "typescript"],
    )
    .await?;

    let payload = serde_json::to_string(file_map)?;

    // Call the script using found bun
    let mut cmd = tokio::process::Command::new(&bun_cmd);
    cmd.arg(script_path).arg("batch").arg(&payload);

    if let Some(r) = root_path {
        cmd.arg(r.to_string_lossy().to_string());
    }
    if dry_run {
        cmd.arg("--dry-run");
    }

    let output = process::run(&mut cmd, process::Limits::from_env()?).await?;

    if !output.status.success() {
        // The error carries the helper's stderr to the caller; logging it as
        // well would put a second copy in front of the JSON error (`--json`).
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("TypeScript/Oxc batch failed: {}", stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// The plan the helper prints with `--dry-run`: `{"moves": [{from, to}],
/// "files": [{path, edits}]}`, one JSON line.
#[derive(serde::Deserialize)]
struct HelperPlan {
    moves: Vec<HelperMove>,
    files: Vec<HelperFile>,
}

#[derive(serde::Deserialize)]
struct HelperMove {
    from: PathBuf,
    to: PathBuf,
}

#[derive(serde::Deserialize)]
struct HelperFile {
    path: PathBuf,
    edits: usize,
}

fn parse_plan(stdout: &str) -> Result<MovePreview> {
    let line = stdout
        .lines()
        .rev()
        .find(|line| line.trim_start().starts_with('{'))
        .ok_or_else(|| anyhow::anyhow!("The TypeScript helper printed no plan: {stdout:?}"))?;
    let plan: HelperPlan = serde_json::from_str(line)
        .context("The TypeScript helper printed a plan refac cannot read")?;
    let mut preview = MovePreview {
        moves: plan
            .moves
            .into_iter()
            .map(|entry| (entry.from, entry.to))
            .collect(),
        ..Default::default()
    };
    for file in plan.files {
        preview.add_edits(file.path, file.edits);
    }
    Ok(preview)
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

        // Minimal tsconfig covering the complete project source set
        tokio::fs::write(
            root.join("tsconfig.json"),
            r#"{"compilerOptions":{"target":"es2020","module":"commonjs"},"include":["src/**/*"]}"#,
        )
        .await?;

        tokio::fs::create_dir_all(root.join("src/utils")).await?;

        // src/utils/format.ts — inside the folder being moved
        tokio::fs::write(
            root.join("src/utils/format.ts"),
            "export function fmt(s: string) { return s.trim(); }\n",
        )
        .await?;

        // src/app.ts — outside, imports from utils/
        tokio::fs::write(
            root.join("src/app.ts"),
            "import { fmt } from \"./utils/format\";\nconsole.log(fmt(\"hi\"));\n",
        )
        .await?;

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
        assert!(
            root.join("src/helpers").exists(),
            "src/helpers should exist"
        );
        assert!(
            root.join("src/helpers/format.ts").exists(),
            "file inside moved dir should exist"
        );

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
