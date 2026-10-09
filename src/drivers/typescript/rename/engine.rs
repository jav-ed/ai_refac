use crate::drivers::resolve_resource_path;
use crate::drivers::typescript::dependencies;
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::process::Command;

/// Folder name of the per-platform native TypeScript 7 package, which ships
/// the compiler and language server as one executable.
fn platform_package() -> Result<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => Ok("typescript-linux-x64"),
        ("linux", "aarch64") => Ok("typescript-linux-arm64"),
        ("macos", "x86_64") => Ok("typescript-darwin-x64"),
        ("macos", "aarch64") => Ok("typescript-darwin-arm64"),
        (os, arch) => bail!("The TypeScript native engine is unsupported on {os}/{arch}"),
    }
}

/// Install (when needed) and return the TypeScript 7 native executable that the
/// helper's locked `typescript-native` dependency provides. It runs beside the
/// TypeScript 6 helper because TypeScript 7 removed the JavaScript API that
/// file moves use.
pub async fn locate() -> Result<PathBuf> {
    let script_path = resolve_resource_path("scripts/ts_refactor.ts")?;
    let script_dir = script_path
        .parent()
        .context("Could not determine scripts directory")?;
    dependencies::ensure_installed(
        &dependencies::bun_command(),
        script_dir,
        &["typescript-native"],
    )
    .await?;
    let executable = script_dir
        .join("node_modules/@typescript")
        .join(platform_package()?)
        .join("lib/tsc");
    if !executable.exists() {
        bail!(
            "TypeScript native executable missing: {}. Run `bun install --frozen-lockfile` in {}",
            executable.display(),
            script_dir.display()
        );
    }
    Ok(executable)
}

/// The engine silently ignores some options TypeScript 7 removed (`baseUrl`
/// breaks import resolution without any error), which would turn a rename into
/// a partial one. Its own project loader reports those as config errors, so a
/// failed file listing is the authoritative guard: hard-fail instead of
/// guessing. The listing must also contain the target, otherwise the engine
/// would rename inside a different tsconfig than the one the caller named.
/// A batch checks every file it renames in, with the one listing.
pub async fn preflight(
    executable: &Path,
    project_root: &Path,
    targets: &[PathBuf],
    timeout: Duration,
) -> Result<()> {
    let output = tokio::time::timeout(
        timeout,
        Command::new(executable)
            .args(["-p", ".", "--listFilesOnly"])
            .current_dir(project_root)
            .kill_on_drop(true)
            .output(),
    )
    .await
    .context("The TypeScript engine timed out while loading the project")?
    .context("Could not run the TypeScript native engine")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let errors: Vec<&str> = stdout
            .lines()
            .filter(|line| line.contains("error TS"))
            .collect();
        bail!(
            "The TypeScript 7 native engine rejects this project's tsconfig:\n{}\nSymbol rename needs a configuration TypeScript 7 accepts. Nothing was changed.",
            if errors.is_empty() {
                stdout.trim().to_string()
            } else {
                errors.join("\n")
            }
        );
    }
    for target in targets {
        let name = target.file_name();
        let listed = stdout
            .lines()
            .map(Path::new)
            .filter(|path| path.file_name() == name)
            .any(|path| {
                path == target
                    || path
                        .canonicalize()
                        .is_ok_and(|canonical| canonical == *target)
            });
        if !listed {
            bail!(
                "{} is not part of the tsconfig in {}. Point --project-path at the package whose tsconfig includes the file.",
                target.display(),
                project_root.display()
            );
        }
    }
    Ok(())
}
