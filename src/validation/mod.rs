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

        initial_sanity_check(&source, "move", Some(&target))?;
        Ok(())
    }

    #[test]
    fn test_invalid_operation() {
        let source = vec!["any.txt".to_string()];
        let result = initial_sanity_check(&source, "delete", None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Operation 'delete' is not supported. Only 'move' is currently implemented.");
    }

    #[test]
    fn test_missing_source_file() {
        let source = vec!["non_existent.txt".to_string()];
        let result = initial_sanity_check(&source, "move", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Source path does not exist"));
    }

    #[test]
    fn test_mismatched_target_count() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("source.txt");
        File::create(&file_path)?;
        
        let source = vec![file_path.to_str().unwrap().to_string()];
        let target = vec!["t1.txt".to_string(), "t2.txt".to_string()];

        let result = initial_sanity_check(&source, "move", Some(&target));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Mismatch check"));
        Ok(())
    }
}
