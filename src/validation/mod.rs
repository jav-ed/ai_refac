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
pub fn initial_sanity_check(
    source_paths: &[String],
    operation: &str,
    target_path: Option<&Vec<String>>,
    project_path: Option<&str>,
) -> Result<()> {
    // 1. Validate Operation
    if operation != "move" {
        bail!(
            "Operation '{}' is not supported. Only 'move' is currently implemented.",
            operation
        );
    }

    // 2. Validate Targets
    if let Some(targets) = target_path {
        if targets.len() != source_paths.len() {
            bail!(
                "Mismatch check: Source count ({}) != Target count ({})",
                source_paths.len(),
                targets.len()
            );
        }
    }

    // 3. Validate Source Paths
    for path_str in source_paths {
        let path = Path::new(path_str);

        // If absolute, check directly. If relative, join with project_path if available.
        let final_path = if path.is_absolute() {
            path.to_path_buf()
        } else if let Some(root) = project_path {
            Path::new(root).join(path)
        } else {
            // If relative and no project_path, we can only check relative to CWD, which might be wrong but it's the legacy behavior.
            path.to_path_buf()
        };

        if !final_path.exists() {
            // Improve error message to show what we checked
            bail!(
                "Source path does not exist: {} (Resolved to: {:?})",
                path_str,
                final_path
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_valid_move_operation() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("source.txt");
        File::create(&file_path)?;

        let source = vec![file_path.to_str().unwrap().to_string()];
        let target = vec!["target.txt".to_string()];

        initial_sanity_check(&source, "move", Some(&target), None)?;
        Ok(())
    }

    #[test]
    fn test_invalid_operation() {
        let source = vec!["any.txt".to_string()];
        let result = initial_sanity_check(&source, "delete", None, None);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Operation 'delete' is not supported. Only 'move' is currently implemented."
        );
    }

    #[test]
    fn test_missing_source_file() {
        let source = vec!["non_existent.txt".to_string()];
        let result = initial_sanity_check(&source, "move", None, None);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Source path does not exist")
        );
    }

    #[test]
    fn test_mismatched_target_count() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("source.txt");
        File::create(&file_path)?;

        let source = vec![file_path.to_str().unwrap().to_string()];
        let target = vec!["t1.txt".to_string(), "t2.txt".to_string()];

        let result = initial_sanity_check(&source, "move", Some(&target), None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Mismatch check"));
        Ok(())
    }
}
