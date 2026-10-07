//! The Android namespace of a module, read from its Gradle build script. The
//! namespace decides where `R` and `BuildConfig` live and what a leading dot
//! in a manifest class name stands for.

use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub struct Module {
    pub dir: PathBuf,
    pub namespace: String,
}

/// The module a build script describes, or `None` when it is not an Android
/// module. A module counts as Android when its build script sets a namespace
/// or its directory holds a manifest (`has_manifest`); the latter without a
/// readable namespace is an error, because its resources could not be updated
/// correctly. A plugin mentioned in a root script does not make a module.
pub fn read_module(build_file: &Path, has_manifest: bool) -> Result<Option<Module>> {
    let text = std::fs::read_to_string(build_file)?;
    let dir = build_file
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    if let Some(namespace) = parse_namespace(&text) {
        return Ok(Some(Module { dir, namespace }));
    }
    if has_manifest {
        bail!(
            "{} builds an Android module (it has a manifest) but sets no `namespace = \"...\"` there, so refac cannot tell where R, BuildConfig and relative class names in XML point. Set the namespace in this file.",
            build_file.display()
        );
    }
    Ok(None)
}

/// `namespace = "x.y"` (Kotlin DSL) or `namespace 'x.y'` (Groovy).
fn parse_namespace(text: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let rest = line.trim_start().strip_prefix("namespace")?;
        let rest = rest.strip_prefix(|ch: char| ch == '=' || ch.is_whitespace())?;
        let rest = rest.trim_start_matches(|ch: char| ch == '=' || ch.is_whitespace());
        let quote = rest.chars().next().filter(|ch| *ch == '"' || *ch == '\'')?;
        let value = &rest[1..];
        Some(value[..value.find(quote)?].to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_kotlin_and_groovy_namespaces() {
        assert_eq!(
            parse_namespace("android {\n    namespace = \"com.example.droid\"\n}\n"),
            Some("com.example.droid".to_string())
        );
        assert_eq!(
            parse_namespace("android {\n  namespace 'com.example.old'\n}"),
            Some("com.example.old".to_string())
        );
        assert_eq!(parse_namespace("namespaceFoo = \"x\""), None);
        assert_eq!(parse_namespace("plugins { kotlin(\"jvm\") }"), None);
    }

    #[test]
    fn a_plain_jvm_module_is_not_android() {
        let dir = tempfile::tempdir().unwrap();
        let build = dir.path().join("build.gradle.kts");
        std::fs::write(&build, "plugins { kotlin(\"jvm\") version \"2.4.20\" }\n").unwrap();
        assert_eq!(read_module(&build, false).unwrap(), None);
    }

    #[test]
    fn a_module_with_a_manifest_but_no_namespace_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let build = dir.path().join("build.gradle.kts");
        std::fs::write(&build, "plugins { id(\"com.android.application\") }\n").unwrap();
        let error = read_module(&build, true).unwrap_err().to_string();
        assert!(error.contains("namespace"), "{error}");
        // A root script that only declares the plugin is not a module.
        assert_eq!(read_module(&build, false).unwrap(), None);
    }

    #[test]
    fn a_module_reports_its_directory_and_namespace() {
        let dir = tempfile::tempdir().unwrap();
        let build = dir.path().join("build.gradle");
        std::fs::write(
            &build,
            "android { namespace 'a.b' }\nandroid {\n  namespace \"a.b\"\n}\n",
        )
        .unwrap();
        let module = read_module(&build, false).unwrap().unwrap();
        assert_eq!(
            module,
            Module {
                dir: dir.path().to_path_buf(),
                namespace: "a.b".to_string()
            }
        );
    }
}
