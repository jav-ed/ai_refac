use anyhow::{Context, Result, bail};
use std::path::Path;

/// Locate the Bun executable that runs the helper scripts and installs their
/// locked dependencies.
pub(super) fn bun_command() -> String {
    // 1. Try generic "bun"
    if std::process::Command::new("bun")
        .arg("--version")
        .output()
        .is_ok()
    {
        return "bun".to_string();
    }

    // 2. Try User's Home (Linux/macOS)
    if let Ok(home) = std::env::var("HOME") {
        let path = Path::new(&home).join(".bun/bin/bun");
        if path.exists() {
            return path.to_string_lossy().to_string();
        }
    }

    // 3. Fallback to generic
    "bun".to_string()
}

/// Whether every package folder is under `scripts/node_modules`.
fn all_present(script_dir: &Path, packages: &[&str]) -> bool {
    packages
        .iter()
        .all(|name| script_dir.join("node_modules").join(name).exists())
}

/// Install the locked helper dependencies when this checkout is new. A package
/// counts as present when its folder exists under `scripts/node_modules`.
///
/// Two processes that both find the packages missing (two refac commands, or
/// the parallel tests of a fresh checkout) would run `bun install` into the
/// same folder and fail on each other's half-linked files. The install runs
/// under a file lock; the one that waits finds the packages present afterwards.
pub(super) async fn ensure_installed(
    bun: &str,
    script_dir: &Path,
    packages: &[&str],
) -> Result<()> {
    if all_present(script_dir, packages) {
        return Ok(());
    }

    let lock_path = script_dir.join(".install.lock");
    let lock = tokio::task::spawn_blocking(move || -> Result<std::fs::File> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&lock_path)
            .with_context(|| format!("Could not open the install lock {}", lock_path.display()))?;
        file.lock()?;
        Ok(file)
    })
    .await??;
    if all_present(script_dir, packages) {
        return Ok(());
    }

    tracing::info!(
        "TypeScript dependencies missing in {:?}, running bun install...",
        script_dir
    );
    let install = tokio::process::Command::new(bun)
        .arg("install")
        .arg("--frozen-lockfile")
        .current_dir(script_dir)
        .output()
        .await?;
    drop(lock);
    if !install.status.success() {
        bail!(
            "bun install failed in {:?}: {}",
            script_dir,
            String::from_utf8_lossy(&install.stderr)
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests;
