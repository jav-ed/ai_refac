use super::{MovePreview, RefactorDriver};
use anyhow::{Context, Ok, Result};
use async_trait::async_trait;
use helper_plan::{parse_helper_plan, parse_plan};
use std::path::PathBuf;

mod dependencies;
pub mod files;
mod helper_plan;
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
        self.move_alongside(file_map, Vec::new(), root_path).await?;
        Ok(())
    }

    /// The notes of the helper: what the move changes about the files a glob
    /// selects, which no edit can repair.
    async fn move_files_with_notes(
        &self,
        file_map: Vec<(String, String)>,
        root_path: Option<&std::path::Path>,
    ) -> Result<Vec<String>> {
        self.move_alongside(file_map, Vec::new(), root_path).await
    }

    /// The helper plans the whole move before it writes anything; with
    /// `--dry-run` it prints that plan instead of applying it. It also checks
    /// that each rewritten specifier resolves, against a model of the files as
    /// they will be after the move, so a dry run refuses what the real move
    /// would roll back.
    async fn plan_move(
        &self,
        file_map: Vec<(String, String)>,
        root_path: Option<&std::path::Path>,
    ) -> Result<MovePreview> {
        self.plan_alongside(file_map, Vec::new(), root_path).await
    }
}

impl TypeScriptDriver {
    /// Moves `file_map` and fixes the code that imports `alongside` as well: the
    /// pages and pictures another driver moves right after this returns. The
    /// files of `alongside` are not renamed here. Returns a note naming the
    /// files whose imports changed (when there are pages or pictures) and the
    /// notes of the helper.
    pub async fn move_alongside(
        &self,
        file_map: Vec<(String, String)>,
        alongside: Vec<(String, String)>,
        root_path: Option<&std::path::Path>,
    ) -> Result<Vec<String>> {
        let stdout = run_helper(&file_map, &alongside, root_path, false).await?;
        tracing::info!("TypeScript/Oxc batch output: {}", stdout);
        let plan = parse_helper_plan(&stdout)?;
        let mut notes = Vec::new();
        if !alongside.is_empty() && !plan.files.is_empty() {
            notes.push(format!(
                "Updated the imports in {} code file(s) for the moved pages and pictures.",
                plan.files.len()
            ));
        }
        notes.extend(plan.notes);
        Ok(notes)
    }

    /// The plan of `move_alongside`.
    pub async fn plan_alongside(
        &self,
        file_map: Vec<(String, String)>,
        alongside: Vec<(String, String)>,
        root_path: Option<&std::path::Path>,
    ) -> Result<MovePreview> {
        let stdout = run_helper(&file_map, &alongside, root_path, true).await?;
        parse_plan(&stdout)
    }
}

/// `bun scripts/ts_refactor.ts`, with the locked parser and resolver installed
/// when this checkout is new.
async fn helper_command() -> Result<tokio::process::Command> {
    let script_path = super::resolve_resource_path("scripts/ts_refactor.ts")?;
    let bun_cmd = dependencies::bun_command();
    let script_dir = script_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Could not determine scripts directory"))?;
    dependencies::ensure_installed(
        &bun_cmd,
        script_dir,
        &[
            "oxc-parser",
            "oxc-resolver",
            "typescript",
            "@astrojs/compiler-rs",
        ],
    )
    .await?;
    let mut cmd = tokio::process::Command::new(&bun_cmd);
    cmd.arg(script_path);
    Ok(cmd)
}

/// Runs the helper and returns what it printed. A failure carries the
/// helper's stderr to the caller; logging it as well would put a second copy
/// in front of the JSON error (`--json`).
async fn run_command(mut cmd: tokio::process::Command, what: &str) -> Result<String> {
    let output = process::run(&mut cmd, process::Limits::from_env()?).await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("{what} failed: {}", stderr);
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Runs `scripts/ts_refactor.ts` on the pairs and returns what it printed.
/// `alongside` are moves another tool carries out: the helper plans them but
/// does not rename them.
async fn run_helper(
    file_map: &[(String, String)],
    alongside: &[(String, String)],
    root_path: Option<&std::path::Path>,
    dry_run: bool,
) -> Result<String> {
    let mut cmd = helper_command().await?;
    cmd.arg("batch").arg(serde_json::to_string(file_map)?);
    if let Some(r) = root_path {
        cmd.arg(r.to_string_lossy().to_string());
    }
    if !alongside.is_empty() {
        cmd.arg("--external").arg(serde_json::to_string(alongside)?);
    }
    if dry_run {
        cmd.arg("--dry-run");
    }
    run_command(cmd, "TypeScript/Oxc batch").await
}

/// The component files of the project (`extensions`: `astro`, `mdx`) that
/// import from one of `defining` and write `name`: the files a symbol rename
/// would leave behind, because the language server cannot read inside them.
pub async fn component_users(
    project: &std::path::Path,
    name: &str,
    defining: &[PathBuf],
    extensions: &[&str],
) -> Result<Vec<PathBuf>> {
    let mut cmd = helper_command().await?;
    cmd.arg("users")
        .arg(project)
        .arg(name)
        .arg(serde_json::to_string(defining)?)
        .arg(serde_json::to_string(extensions)?);
    let stdout = run_command(cmd, "Reading the component files").await?;
    let line = stdout
        .lines()
        .rev()
        .find(|line| line.trim_start().starts_with('['))
        .ok_or_else(|| anyhow::anyhow!("The TypeScript helper printed no file list: {stdout:?}"))?;
    serde_json::from_str(line)
        .context("The TypeScript helper printed a file list refac cannot read")
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
