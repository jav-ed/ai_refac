//! The plain-JVM mirror of a Kotlin Multiplatform build.
//!
//! The Kotlin server cannot read a multiplatform Gradle build: its import
//! prints `Failed to find 'target' in Kotlin extension`, no source set is
//! known afterwards, and every move is refused with "Destination directory was
//! not found". It reads a plain Kotlin/JVM build well, so refac gives it one:
//! a throw-away project with the same relative layout, whose single module
//! takes every `src/<source set>/kotlin` and `java` folder of the real project
//! as a source folder. The server works on that copy; refac keeps speaking in
//! the real project's paths, and `Mirror` translates at the server boundary.
//!
//! What the mirror cannot do: it has no dependencies, so Compose, Android and
//! platform APIs do not resolve in it. Package lines and the references to the
//! project's own classes, which is what a move changes, do not need them.
//! What the server decides from the missing symbols (an import "unused"
//! because its receiver is unresolved) is undone by `imports::keep_imports`.
//! Its copies hide the `expect` and `actual` modifiers from the server
//! (`modifiers`), because a plain module has no place for an expect
//! declaration; for the same reason a file that declares one is not moved.

mod imports;
mod layout;
mod modifiers;
#[cfg(test)]
mod tests;

pub use imports::keep_imports;
use modifiers::{blank_expect_actual, first_declaration_of, first_expect_actual_line};

use crate::drivers::kotlin::android::moved::join_relative;
use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use url::Url;
use walkdir::WalkDir;

/// `FileChangeType` of `workspace/didChangeWatchedFiles`.
const CREATED: u64 = 1;
const CHANGED: u64 = 2;
const DELETED: u64 = 3;

/// Files of the real project that the Gradle wrapper needs, so that the mirror
/// imports with the project's own Gradle version.
const WRAPPER: &[&str] = &[
    "gradlew",
    "gradlew.bat",
    "gradle/wrapper/gradle-wrapper.properties",
    "gradle/wrapper/gradle-wrapper.jar",
];

pub struct Mirror {
    real: PathBuf,
    dir: TempDir,
}

impl Mirror {
    /// The mirror of a Gradle root, or `None` when the project is not
    /// multiplatform and the server can read it as it is.
    pub fn for_project(real: &Path) -> Result<Option<Self>> {
        let roots = layout::source_roots(real)?;
        if !layout::is_multiplatform(&roots) {
            return Ok(None);
        }
        let version = layout::kotlin_version(real, &roots)?;
        let dir = tempfile::Builder::new()
            .prefix("refac-kotlin-mirror-")
            .tempdir()?;
        let mirror = Self {
            real: real.to_path_buf(),
            dir,
        };
        mirror.write_build(&roots, &version)?;
        for root in &roots {
            mirror.copy_sources(&real.join(root))?;
        }
        Ok(Some(mirror))
    }

    /// Whether the server works on a mirror for this Gradle root.
    pub fn applies_to(real: &Path) -> Result<bool> {
        Ok(layout::is_multiplatform(&layout::source_roots(real)?))
    }

    pub fn root(&self) -> &Path {
        self.dir.path()
    }

    fn write_build(&self, roots: &[PathBuf], version: &str) -> Result<()> {
        let root = self.root();
        std::fs::write(
            root.join("settings.gradle.kts"),
            "// Written by refac: a plain Kotlin/JVM view of a Kotlin Multiplatform project.\nrootProject.name = \"refac-mirror\"\n",
        )?;
        std::fs::write(root.join("build.gradle.kts"), build_script(roots, version))?;
        for file in WRAPPER {
            let from = self.real.join(file);
            if from.is_file() {
                let to = root.join(file);
                std::fs::create_dir_all(to.parent().context("A wrapper file without a folder")?)?;
                std::fs::copy(&from, &to)
                    .with_context(|| format!("Cannot copy {}", from.display()))?;
            }
        }
        Ok(())
    }

    /// The mirror's place for a path of the real project; other paths stay.
    pub fn to_mirror(&self, path: &Path) -> PathBuf {
        match path.strip_prefix(&self.real) {
            Ok(rest) => join_relative(self.root(), rest),
            Err(_) => path.to_path_buf(),
        }
    }

    /// The real project's place for a path of the mirror; other paths stay.
    pub fn to_real(&self, path: &Path) -> PathBuf {
        match path.strip_prefix(self.root()) {
            Ok(rest) => join_relative(&self.real, rest),
            Err(_) => path.to_path_buf(),
        }
    }

    /// Every `file:` URI in the message, in the mirror's namespace.
    pub fn uris_to_mirror(&self, message: &mut Value) {
        map_uris(message, &|path| self.to_mirror(path));
    }

    /// Every `file:` URI in the message, in the real project's namespace.
    pub fn uris_to_real(&self, message: &mut Value) {
        map_uris(message, &|path| self.to_real(path));
    }

    /// The real files and folders to mirror: sources only, since the server
    /// reads nothing else.
    fn copy_sources(&self, from: &Path) -> Result<()> {
        for entry in WalkDir::new(from) {
            let path = entry?.into_path();
            if is_source(&path) {
                self.copy_file(&path)?;
            }
        }
        Ok(())
    }

    fn copy_file(&self, real: &Path) -> Result<()> {
        let to = self.to_mirror(real);
        std::fs::create_dir_all(to.parent().context("A source without a folder")?)?;
        if real.extension().is_some_and(|extension| extension == "kt") {
            let text = std::fs::read_to_string(real)
                .with_context(|| format!("Cannot read {}", real.display()))?;
            std::fs::write(&to, blank_expect_actual(&text))
                .with_context(|| format!("Cannot write {}", to.display()))?;
        } else {
            std::fs::copy(real, &to).with_context(|| format!("Cannot copy {}", real.display()))?;
        }
        Ok(())
    }

    /// Writes the text of a document the caller wrote to the real project and
    /// returns the text the server is to see: the mirror's version of it.
    pub fn write_document(&self, real: &Path, text: &str) -> Result<String> {
        // A document outside the project has no place in the mirror.
        if !real.starts_with(&self.real) {
            return Ok(text.to_string());
        }
        let seen = if real.extension().is_some_and(|extension| extension == "kt") {
            blank_expect_actual(text)
        } else {
            text.to_string()
        };
        let to = self.to_mirror(real);
        std::fs::create_dir_all(to.parent().context("A document without a folder")?)?;
        std::fs::write(&to, &seen).with_context(|| format!("Cannot write {}", to.display()))?;
        Ok(seen)
    }

    /// Brings the mirror up to date with what the caller did to the real
    /// project, as the `workspace/didChangeWatchedFiles` event says, and
    /// returns the event for the server: events about files the mirror does
    /// not hold (resources, manifests) are dropped.
    pub fn follow_changes(&self, params: &Value) -> Result<Value> {
        let mut forwarded = Vec::new();
        for change in params["changes"].as_array().into_iter().flatten() {
            let uri = change["uri"].as_str().context("A change without a uri")?;
            let real = Url::parse(uri)
                .ok()
                .and_then(|url| url.to_file_path().ok())
                .with_context(|| format!("A change of {uri}, which is not a file"))?;
            let mirrored = self.to_mirror(&real);
            match change["type"].as_u64() {
                Some(DELETED) => remove(&mirrored)?,
                Some(CREATED) | Some(CHANGED) if real.is_dir() => self.copy_sources(&real)?,
                Some(CREATED) | Some(CHANGED) if is_source(&real) => {
                    if !real.is_file() {
                        bail!(
                            "The server was told about {}, which is not there",
                            real.display()
                        );
                    }
                    self.copy_file(&real)?;
                }
                Some(CREATED) | Some(CHANGED) => continue,
                other => bail!("Unknown file change type {other:?} for {uri}"),
            }
            let mut change = change.clone();
            map_uris(&mut change, &|path| self.to_mirror(path));
            forwarded.push(change);
        }
        Ok(serde_json::json!({ "changes": forwarded }))
    }

    /// A line for the user: this change was planned on the mirror.
    pub fn note() -> String {
        "Kotlin Multiplatform: the Kotlin server cannot read this build, so it planned the change on a plain-JVM copy of every source set. Package lines and references to the project's own classes follow and imports of libraries it cannot see are kept. Compile every target afterwards.".to_string()
    }
}

/// Refuses to move a file that declares `expect` or `actual` in a build the
/// server is served a mirror of. In the mirror an expect declaration and its
/// actual are two declarations of one name in one module: the server refuses
/// the second move with "Following declarations would clash", and the move of
/// either one alone leaves the other in a package it no longer matches. Nothing
/// has been started or changed.
pub fn refuse_expect_actual(root: &Path, files: &[PathBuf]) -> Result<()> {
    if !Mirror::applies_to(root)? {
        return Ok(());
    }
    for file in files {
        let text = std::fs::read_to_string(file)
            .with_context(|| format!("Cannot read {}", file.display()))?;
        if let Some(line) = first_expect_actual_line(&text) {
            bail!(
                "{} declares something `expect` or `actual` (line {line}). In a Kotlin Multiplatform build refac serves the Kotlin server a plain-JVM copy, in which an expect declaration and its actual are two declarations of one name: moving them clashes, and moving one leaves the other in a package it must share. Move the files that declare expect or actual by hand (package line, then the compiler lists the imports that follow) and everything else with refac.",
                file.display()
            );
        }
    }
    Ok(())
}

/// Refuses to rename a symbol that `file` declares `expect` or `actual`: the
/// server sees an expect declaration and its actual as two declarations of
/// one name and renames one of them, which leaves the build broken. Only the
/// file named in the request is read, so a symbol declared elsewhere is not
/// caught. Nothing has been started or changed.
pub fn refuse_expect_actual_symbol(root: &Path, file: &Path, symbol: &str) -> Result<()> {
    if !Mirror::applies_to(root)? {
        return Ok(());
    }
    let text =
        std::fs::read_to_string(file).with_context(|| format!("Cannot read {}", file.display()))?;
    if let Some(line) = first_declaration_of(&text, symbol) {
        bail!(
            "`{symbol}` is declared `expect` or `actual` in {} (line {line}). In a Kotlin Multiplatform build refac serves the Kotlin server a plain-JVM copy, in which an expect declaration and its actual are two declarations of one name, and a rename would change only one of them. Rename it by hand in every source set and let the compiler list what is left.",
            file.display()
        );
    }
    Ok(())
}

fn build_script(roots: &[PathBuf], version: &str) -> String {
    let dirs = |kind: &str| -> String {
        roots
            .iter()
            .filter(|root| root.file_name().is_some_and(|name| name == kind))
            .map(|root| format!("            {kind}.srcDir(\"{}\")\n", slashed(root)))
            .collect()
    };
    format!(
        "// Written by refac: a plain Kotlin/JVM view of a Kotlin Multiplatform project.\nplugins {{\n    kotlin(\"jvm\") version \"{version}\"\n}}\n\nrepositories {{\n    mavenCentral()\n}}\n\nkotlin {{\n    sourceSets {{\n        main {{\n{kotlin}        }}\n    }}\n}}\n\nsourceSets {{\n    main {{\n{java}    }}\n}}\n",
        kotlin = dirs("kotlin"),
        java = dirs("java"),
    )
}

fn slashed(path: &Path) -> String {
    path.components()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn is_source(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("kt" | "java")
    )
}

fn remove(path: &Path) -> Result<()> {
    let result = if path.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };
    match result {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            Err(error).with_context(|| format!("Cannot remove {}", path.display()))
        }
        _ => Ok(()),
    }
}

/// Rewrites every string that is a `file:` URI of a path `map` changes. The
/// path is parsed and written again, so the encoding the server chose does not
/// matter.
fn map_uris(value: &mut Value, map: &dyn Fn(&Path) -> PathBuf) {
    match value {
        Value::String(text) if text.starts_with("file:") => {
            let Some(path) = Url::parse(text)
                .ok()
                .and_then(|url| url.to_file_path().ok())
            else {
                return;
            };
            let mapped = map(&path);
            if mapped != path
                && let Ok(url) = Url::from_file_path(&mapped)
            {
                *text = url.to_string();
            }
        }
        Value::Array(items) => items.iter_mut().for_each(|item| map_uris(item, map)),
        // `WorkspaceEdit.changes` is keyed by URI.
        Value::Object(fields) => {
            for (key, mut item) in std::mem::take(fields) {
                map_uris(&mut item, map);
                let mut key = Value::String(key);
                map_uris(&mut key, map);
                if let Value::String(key) = key {
                    fields.insert(key, item);
                }
            }
        }
        _ => {}
    }
}
