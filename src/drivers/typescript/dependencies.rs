use anyhow::{Result, bail};
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

/// Install the locked helper dependencies when this checkout is new. A package
/// counts as present when its folder exists under `scripts/node_modules`.
pub(super) async fn ensure_installed(
    bun: &str,
    script_dir: &Path,
    packages: &[&str],
) -> Result<()> {
    if packages
        .iter()
        .all(|name| script_dir.join("node_modules").join(name).exists())
    {
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
    if !install.status.success() {
        bail!(
            "bun install failed in {:?}: {}",
            script_dir,
            String::from_utf8_lossy(&install.stderr)
        );
    }
    Ok(())
}
