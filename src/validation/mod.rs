use anyhow::{Result, bail};
use std::path::Path;

/// Validates the initial request parameters.
/// 
/// # Checks
/// 1. Operation must be supported (currently only "move").
/// 2. Source paths must exist.
/// 3. Source and target counts must match (if target is provided).
/// 
/// # Internal Docs
/// We use this function to fail fast before doing any heavy lifting or spawning processes.
/// This prevents partial states where one file is moved and another fails.
pub fn initial_sanity_check(source_paths: &[String], operation: &str, target_path: Option<&Vec<String>>) -> Result<()> {
    // 1. Validate Operation
    if operation != "move" {
        bail!("Operation '{}' is not supported. Only 'move' is currently implemented.", operation);
    }

    // 2. Validate Source Paths
    for path_str in source_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            bail!("Source path does not exist: {}", path_str);
        }
    }

    // 3. Validate Targets
    if let Some(targets) = target_path {
        if targets.len() != source_paths.len() {
            bail!(
                "Mismatch check: Source count ({}) != Target count ({})",
                source_paths.len(),
                targets.len()
            );
        }
    }

    Ok(())
}
