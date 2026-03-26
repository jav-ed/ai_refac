use anyhow::{Result, bail};
use std::collections::HashSet;
use std::path::Path;

/// Validates the initial request parameters.
///
/// # Checks
/// 1. Operation must be supported (currently only "move").
/// 2. Source and target counts must match.
/// 3. No duplicate source paths.
/// 4. No duplicate target paths.
/// 5. No source == target pairs.
/// 6. Source paths must exist (files or directories).
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

    // 3. No duplicate source paths
    let mut seen_sources: HashSet<&str> = HashSet::new();
    for path_str in source_paths {
        if !seen_sources.insert(path_str.as_str()) {
            bail!("Duplicate source path: {}", path_str);
        }
    }

    // 4. No duplicate target paths; 5. No source == target
    if let Some(targets) = target_path {
        let mut seen_targets: HashSet<&str> = HashSet::new();
        for tgt in targets {
            if !seen_targets.insert(tgt.as_str()) {
                bail!("Duplicate target path: {}", tgt);
            }
        }
        for (src, tgt) in source_paths.iter().zip(targets.iter()) {
            if src == tgt {
                bail!("Source and target are identical: {}", src);
            }
        }
    }

    // 6. Validate Source Paths exist
    for path_str in source_paths {
        let path = Path::new(path_str);

        let final_path = if path.is_absolute() {
            path.to_path_buf()
        } else if let Some(root) = project_path {
            Path::new(root).join(path)
        } else {
            path.to_path_buf()
        };

        if !final_path.exists() {
            bail!(
                "Source path does not exist: {} (Resolved to: {:?})",
                path_str,
                final_path
            );
        }

        // Directories are allowed — they are routed to the TypeScript driver.
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

    #[test]
    fn test_directory_source_passes_validation() {
        // Directories are now valid source paths — they route to the TypeScript driver.
        let dir = tempdir().unwrap();
        let source = vec![dir.path().to_str().unwrap().to_string()];
        let target = vec!["anywhere/foo".to_string()];

        let result = initial_sanity_check(&source, "move", Some(&target), None);
        assert!(result.is_ok(), "directory source should pass validation: {:?}", result.err());
    }

    #[test]
    fn test_relative_path_resolved_against_project_path() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("sub").join("file.ts");
        std::fs::create_dir_all(file_path.parent().unwrap())?;
        File::create(&file_path)?;

        let source = vec!["sub/file.ts".to_string()];
        let target = vec!["sub/renamed.ts".to_string()];
        let project_root = dir.path().to_str().unwrap();

        initial_sanity_check(&source, "move", Some(&target), Some(project_root))?;
        Ok(())
    }

    #[test]
    fn test_duplicate_source_path_is_rejected() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("a.ts");
        File::create(&file_path)?;
        let path = file_path.to_str().unwrap().to_string();

        let result = initial_sanity_check(
            &[path.clone(), path.clone()],
            "move",
            Some(&vec!["b.ts".to_string(), "c.ts".to_string()]),
            None,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate source path"));
        Ok(())
    }

    #[test]
    fn test_duplicate_target_path_is_rejected() -> Result<()> {
        let dir = tempdir()?;
        let a = dir.path().join("a.ts");
        let b = dir.path().join("b.ts");
        File::create(&a)?;
        File::create(&b)?;

        let result = initial_sanity_check(
            &[a.to_str().unwrap().to_string(), b.to_str().unwrap().to_string()],
            "move",
            Some(&vec!["out.ts".to_string(), "out.ts".to_string()]),
            None,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate target path"));
        Ok(())
    }

    #[test]
    fn test_source_equals_target_is_rejected() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("a.ts");
        File::create(&file_path)?;
        let path = file_path.to_str().unwrap().to_string();

        let result = initial_sanity_check(
            &[path.clone()],
            "move",
            Some(&vec![path.clone()]),
            None,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("identical"));
        Ok(())
    }
}
