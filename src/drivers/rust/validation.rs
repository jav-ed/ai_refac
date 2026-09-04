use super::{module_graph, workspace::SemanticWorkspace};
use anyhow::{Context, Result, bail};
use std::path::Path;

pub fn validate_paths(source: &[String], target: &[String]) -> Result<()> {
    if source == target {
        bail!("Source and target Rust module paths are identical");
    }
    if target.starts_with(source) {
        bail!(
            "Cannot move `crate::{}` into its own subtree `crate::{}`",
            source.join("::"),
            target.join("::")
        );
    }
    Ok(())
}

pub fn validate_result(root: &Path, source: &[String], target: &[String]) -> Result<()> {
    let workspace =
        SemanticWorkspace::load(root).context("Could not reload the moved Cargo workspace")?;
    module_graph::resolve_source(&workspace, target).with_context(|| {
        format!(
            "The target module `crate::{}` did not resolve after the move",
            target.join("::")
        )
    })?;
    if module_graph::resolve_source(&workspace, source).is_ok() {
        bail!(
            "The old module path `crate::{}` still resolves after the move",
            source.join("::")
        );
    }

    let output = std::process::Command::new("cargo")
        .args(["check", "--workspace", "--all-targets"])
        .current_dir(root)
        .output()
        .context("Could not execute cargo check after the Rust module move")?;
    if !output.status.success() {
        bail!(
            "cargo check --workspace --all-targets failed after the planned move:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}
