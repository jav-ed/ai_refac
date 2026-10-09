//! What the mirror is built from: the source roots of a Gradle project, the
//! decision that it is multiplatform, and the Kotlin version the mirror's own
//! build file asks for.

use crate::drivers::kotlin::project::source_root;
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// The Kotlin version of the mirror's build, for projects whose own build
/// files do not state one refac can read.
pub const VERSION_ENV: &str = "REFAC_KOTLIN_MIRROR_VERSION";

/// Folders that hold output or tooling, never sources.
const SKIPPED: &[&str] = &["build", "node_modules", "Pods"];

/// Every `<module>/src/<source set>/kotlin` and `.../java` folder below the
/// Gradle root, relative to it and sorted.
pub fn source_roots(root: &Path) -> Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    let mut walker = WalkDir::new(root).min_depth(1).into_iter();
    while let Some(entry) = walker.next() {
        let entry = entry.with_context(|| format!("Cannot read {}", root.display()))?;
        if !entry.file_type().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy();
        if name.starts_with('.') || SKIPPED.contains(&name.as_ref()) {
            walker.skip_current_dir();
            continue;
        }
        if source_root(entry.path()).as_deref() == Some(entry.path()) {
            found.push(entry.path().strip_prefix(root)?.to_path_buf());
            walker.skip_current_dir();
        }
    }
    found.sort();
    Ok(found)
}

/// Whether a source set belongs to a Kotlin Multiplatform module: `commonMain`
/// and `commonTest`, and the `<target>Main` sets (`jvmMain`, `androidMain`,
/// `iosMain`). Android and plain JVM modules use `main`, `test`, `androidTest`
/// and the names of their variants and flavors.
pub fn is_multiplatform_set(name: &str) -> bool {
    name == "commonTest" || name.ends_with("Main")
}

/// Whether any source root belongs to a multiplatform module.
pub fn is_multiplatform(roots: &[PathBuf]) -> bool {
    roots.iter().any(|root| {
        root.parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .is_some_and(is_multiplatform_set)
    })
}

/// The Kotlin version for the mirror's build: the one in
/// `REFAC_KOTLIN_MIRROR_VERSION`, else the one the project's version catalog or
/// its Kotlin Multiplatform plugin line states. A version that cannot be read
/// is an error, not a guess: the wrong one would not import.
pub fn kotlin_version(root: &Path, roots: &[PathBuf]) -> Result<String> {
    kotlin_version_in(std::env::var(VERSION_ENV).ok().as_deref(), root, roots)
}

pub(super) fn kotlin_version_in(
    configured: Option<&str>,
    root: &Path,
    roots: &[PathBuf],
) -> Result<String> {
    if let Some(value) = configured {
        return match version_like(value.trim()) {
            true => Ok(value.trim().to_string()),
            false => bail!("{VERSION_ENV} must be a Kotlin version such as 2.1.20, got `{value}`"),
        };
    }
    let mut looked_at = Vec::new();
    let catalog = root.join("gradle/libs.versions.toml");
    if let Ok(text) = std::fs::read_to_string(&catalog) {
        if let Some(version) = catalog_version(&text) {
            return Ok(version);
        }
        looked_at.push(catalog);
    }
    for script in build_scripts(root, roots) {
        let Ok(text) = std::fs::read_to_string(&script) else {
            continue;
        };
        if let Some(version) = plugin_version(&text) {
            return Ok(version);
        }
        looked_at.push(script);
    }
    bail!(
        "refac needs the Kotlin version of this multiplatform build to import its sources with the Kotlin server, and found none in {}. Set {VERSION_ENV}=<version> (for example 2.1.20).",
        if looked_at.is_empty() {
            "gradle/libs.versions.toml or any build.gradle(.kts)".to_string()
        } else {
            looked_at
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        }
    )
}

/// The root build script and the build script of every module that has sources.
fn build_scripts(root: &Path, roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut modules = vec![PathBuf::new()];
    for source in roots {
        // <module>/src/<set>/kotlin
        if let Some(module) = source.ancestors().nth(3) {
            modules.push(module.to_path_buf());
        }
    }
    modules.sort();
    modules.dedup();
    modules
        .iter()
        .flat_map(|module| {
            ["build.gradle.kts", "build.gradle"].map(|name| root.join(module).join(name))
        })
        .collect()
}

/// `kotlin = "2.1.20"` in the `[versions]` table of a version catalog.
fn catalog_version(text: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;
        let key = key.trim();
        if !matches!(
            key,
            "kotlin" | "kotlinVersion" | "kotlin-version" | "kotlin_version"
        ) {
            return None;
        }
        let value = value.trim().trim_matches('"');
        version_like(value).then(|| value.to_string())
    })
}

/// `kotlin("multiplatform") version "2.1.20"` and its spellings, or a
/// `kotlin-gradle-plugin:2.1.20` classpath entry.
fn plugin_version(text: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let lower = line.to_lowercase();
        if let Some(at) = lower.find("kotlin-gradle-plugin:") {
            let rest = &line[at + "kotlin-gradle-plugin:".len()..];
            let version: String = rest
                .chars()
                .take_while(|ch| ch.is_alphanumeric() || matches!(ch, '.' | '-'))
                .collect();
            return version_like(&version).then_some(version);
        }
        if !lower.contains("multiplatform") {
            return None;
        }
        let after = &line[lower.find(" version ")? + " version ".len()..];
        let version = after.trim().trim_start_matches(['"', '\'']);
        let version: String = version
            .chars()
            .take_while(|ch| ch.is_alphanumeric() || matches!(ch, '.' | '-'))
            .collect();
        version_like(&version).then_some(version)
    })
}

/// `2.1.20`, `2.2.0-RC2`: digits first, so `libs.versions.kotlin` and
/// `$kotlinVersion` are not taken for versions.
fn version_like(value: &str) -> bool {
    let mut parts = value.split('.');
    let numeric = |part: Option<&str>| {
        part.is_some_and(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_digit()))
    };
    numeric(parts.next())
        && numeric(parts.next())
        && parts.all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
        })
}
