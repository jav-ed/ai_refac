//! What a Kotlin or Android project looks like on disk: the Gradle root the
//! server imports, and the source roots a package is derived from.

use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

/// The Gradle root: the directory holding `settings.gradle(.kts)`. The server
/// imports the whole build from there, so a module directory is not enough.
pub fn gradle_root(project_path: Option<&Path>) -> Result<PathBuf> {
    let given = match project_path {
        Some(path) => path.to_path_buf(),
        None => std::env::current_dir().context("Cannot read the current directory")?,
    };
    let root = given
        .canonicalize()
        .with_context(|| format!("Project path does not exist: {}", given.display()))?;
    if !root.join("settings.gradle.kts").is_file() && !root.join("settings.gradle").is_file() {
        bail!(
            "No settings.gradle(.kts) in {}. Kotlin refactors need the Gradle project root, the folder that holds settings.gradle.kts.",
            root.display()
        );
    }
    Ok(root)
}

/// The source root a file lives in: `<module>/src/<source set>/kotlin` or
/// `.../java`. The server derives a moved file's new package from the
/// directory's position under this root, so a path outside one is refused.
pub fn source_root(path: &Path) -> Option<PathBuf> {
    path.ancestors().find_map(|candidate| {
        let name = candidate.file_name()?.to_str()?;
        if name != "kotlin" && name != "java" {
            return None;
        }
        let set_dir = candidate.parent()?;
        let src_dir = set_dir.parent()?;
        (src_dir.file_name()?.to_str()? == "src").then(|| candidate.to_path_buf())
    })
}

/// The package a directory under a source root stands for, for example
/// `com.example.util`. `None` for the source root itself.
pub fn package_of_dir(root: &Path, dir: &Path) -> Option<String> {
    let relative = dir.strip_prefix(root).ok()?;
    let parts: Vec<&str> = relative.iter().filter_map(|part| part.to_str()).collect();
    (!parts.is_empty()).then(|| parts.join("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_source_root_of_kotlin_and_java_sets() {
        let file = Path::new("/p/app/src/main/kotlin/com/example/Main.kt");
        assert_eq!(
            source_root(file),
            Some(PathBuf::from("/p/app/src/main/kotlin"))
        );
        let java = Path::new("/p/app/src/debug/java/com/x/A.kt");
        assert_eq!(
            source_root(java),
            Some(PathBuf::from("/p/app/src/debug/java"))
        );
        assert_eq!(source_root(Path::new("/p/app/docs/kotlin/A.kt")), None);
        assert_eq!(source_root(Path::new("/p/app/src/kotlin/A.kt")), None);
    }

    #[test]
    fn derives_the_package_from_the_directory() {
        let root = Path::new("/p/src/main/kotlin");
        assert_eq!(
            package_of_dir(root, &root.join("com/example/util")),
            Some("com.example.util".to_string())
        );
        assert_eq!(package_of_dir(root, root), None);
        assert_eq!(package_of_dir(root, Path::new("/elsewhere")), None);
    }

    #[test]
    fn a_folder_without_settings_is_not_a_gradle_root() {
        let dir = tempfile::tempdir().unwrap();
        let error = gradle_root(Some(dir.path())).unwrap_err().to_string();
        assert!(error.contains("settings.gradle"), "{error}");
        std::fs::write(dir.path().join("settings.gradle.kts"), "").unwrap();
        assert!(gradle_root(Some(dir.path())).is_ok());
    }
}
