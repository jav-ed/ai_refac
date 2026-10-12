//! One walk over the Gradle project that sorts its files for the layers that
//! run after the server: Android resources, other text that may name a class,
//! and sources.

use anyhow::{Context, Result};
use ignore::{DirEntry, WalkBuilder};
use std::path::{Path, PathBuf};

/// Directories that hold build output or tool state, never project files.
const SKIPPED: &[&str] = &[
    "build",
    ".gradle",
    ".git",
    ".idea",
    ".kotlin",
    "node_modules",
];

/// Extensions of text files that can name a class (ProGuard rules, build
/// scripts, service lists, configuration).
const NAMING_TEXT: &[&str] = &[
    "pro",
    "gradle",
    "kts",
    "properties",
    "json",
    "yml",
    "yaml",
    "toml",
    "cfg",
    "conf",
];

#[derive(Debug, Default)]
pub struct Survey {
    /// `build.gradle(.kts)` files: their directory is a module.
    pub build_files: Vec<PathBuf>,
    /// `AndroidManifest.xml` and every `src/<set>/res/**.xml`.
    pub android_xml: Vec<PathBuf>,
    /// Text files other than sources and Android XML that may name a class.
    pub naming_text: Vec<PathBuf>,
    /// Kotlin and Java sources.
    pub sources: Vec<PathBuf>,
}

/// What `.gitignore` leaves out is not the project: reference clones, scratch
/// folders and the output of other tools hold sources and configuration of
/// their own, which no move of this project may read, scan or add imports from
/// (same walk as the Markdown pass).
pub fn survey(root: &Path) -> Result<Survey> {
    let mut found = Survey::default();
    let entries = WalkBuilder::new(root)
        .hidden(false)
        .parents(false)
        .git_global(false)
        .require_git(false)
        .filter_entry(keep)
        .build();
    for entry in entries {
        let path = entry
            .with_context(|| format!("Cannot scan {} for project files", root.display()))?
            .into_path();
        if !path.is_file() {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        let extension = path.extension().and_then(|extension| extension.to_str());
        match (name, extension) {
            ("build.gradle" | "build.gradle.kts", _) => {
                found.build_files.push(path.clone());
                found.naming_text.push(path);
            }
            (_, Some("kt" | "java")) => found.sources.push(path),
            (_, Some("xml")) if is_android_xml(&path) => found.android_xml.push(path),
            (_, Some(extension)) if NAMING_TEXT.contains(&extension) => {
                found.naming_text.push(path)
            }
            _ if is_service_list(&path) => found.naming_text.push(path),
            _ => {}
        }
    }
    Ok(found)
}

fn keep(entry: &DirEntry) -> bool {
    entry.depth() == 0
        || !entry.file_type().is_some_and(|kind| kind.is_dir())
        || !SKIPPED.contains(&entry.file_name().to_str().unwrap_or(""))
}

/// The manifest, or an XML file inside a `res` folder of a source set
/// (`<module>/src/<set>/res/...`).
fn is_android_xml(path: &Path) -> bool {
    if path.file_name().and_then(|name| name.to_str()) == Some("AndroidManifest.xml") {
        return true;
    }
    path.ancestors().any(|dir| {
        dir.file_name().and_then(|name| name.to_str()) == Some("res")
            && dir
                .parent()
                .and_then(Path::parent)
                .and_then(|src| src.file_name())
                .and_then(|name| name.to_str())
                == Some("src")
    })
}

/// `META-INF/services/<interface>` lists implementation class names.
fn is_service_list(path: &Path) -> bool {
    path.parent()
        .and_then(|dir| dir.file_name())
        .and_then(|name| name.to_str())
        == Some("services")
        && path
            .parent()
            .and_then(Path::parent)
            .and_then(|dir| dir.file_name())
            .and_then(|name| name.to_str())
            == Some("META-INF")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(root: &Path, relative: &str) {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "").unwrap();
    }

    #[test]
    fn sorts_files_and_skips_build_output() {
        let dir = tempfile::tempdir().unwrap();
        for file in [
            "app/build.gradle.kts",
            "app/proguard-rules.pro",
            "app/src/main/AndroidManifest.xml",
            "app/src/main/res/layout/main.xml",
            "app/src/main/kotlin/com/x/A.kt",
            "app/src/main/resources/META-INF/services/com.x.Spi",
            "app/src/main/docs/notes.xml",
            "app/build/generated/B.kt",
            "app/build/intermediates/res/layout/main.xml",
            ".gradle/cache.properties",
        ] {
            touch(dir.path(), file);
        }

        let found = survey(dir.path()).unwrap();
        let relative = |paths: &[PathBuf]| {
            let mut names: Vec<String> = paths
                .iter()
                .map(|path| path.strip_prefix(dir.path()).unwrap().display().to_string())
                .collect();
            names.sort();
            names
        };
        assert_eq!(relative(&found.build_files), ["app/build.gradle.kts"]);
        assert_eq!(
            relative(&found.android_xml),
            [
                "app/src/main/AndroidManifest.xml",
                "app/src/main/res/layout/main.xml"
            ]
        );
        assert_eq!(relative(&found.sources), ["app/src/main/kotlin/com/x/A.kt"]);
        assert_eq!(
            relative(&found.naming_text),
            [
                "app/build.gradle.kts",
                "app/proguard-rules.pro",
                "app/src/main/resources/META-INF/services/com.x.Spi"
            ]
        );
    }

    #[test]
    fn what_gitignore_leaves_out_is_not_part_of_the_project() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(".gitignore"),
            "Repos/\nScratch/\n*.local.json\n",
        )
        .unwrap();
        for file in [
            "app/build.gradle.kts",
            "app/src/main/kotlin/com/x/A.kt",
            "app/settings.local.json",
            "Repos/compose/src/commonMain/kotlin/Foo.kt",
            "Repos/compose/build.gradle.kts",
            "Scratch/Agent_Tasks/copy/B.kt",
            "Scratch/Agent_Tasks/copy/config.json",
        ] {
            touch(dir.path(), file);
        }

        let found = survey(dir.path()).unwrap();

        let relative = |paths: &[PathBuf]| {
            let mut names: Vec<String> = paths
                .iter()
                .map(|path| path.strip_prefix(dir.path()).unwrap().display().to_string())
                .collect();
            names.sort();
            names
        };
        assert_eq!(relative(&found.sources), ["app/src/main/kotlin/com/x/A.kt"]);
        assert_eq!(relative(&found.build_files), ["app/build.gradle.kts"]);
        assert_eq!(relative(&found.naming_text), ["app/build.gradle.kts"]);
    }
}
