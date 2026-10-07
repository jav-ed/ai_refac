//! The Android layer. The Kotlin server moves and renames the code but never
//! touches Android XML, and it leaves a moved file without the implicit `R`
//! and `BuildConfig` of its old package. This layer repairs both after the
//! server is done. Projects without Android modules pass through untouched.

use super::declarations::declared_package;
use super::journal::Journal;
use super::moved::MovedFile;
use super::survey::Survey;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

mod imports;
mod namespace;
mod xml;

use namespace::Module;

#[derive(Debug, Default)]
pub struct AndroidReport {
    pub edited: Vec<PathBuf>,
}

/// Update XML and imports for the classes in `renames`. Every write goes
/// through the journal.
pub fn update(
    survey: &Survey,
    moved: &[MovedFile],
    renames: &[(String, String)],
    journal: &mut Journal,
) -> Result<AndroidReport> {
    let mut modules = Vec::new();
    for build_file in &survey.build_files {
        let dir = build_file.parent().unwrap_or(Path::new(""));
        modules.extend(namespace::read_module(
            build_file,
            has_manifest(survey, dir),
        )?);
    }
    let mut report = AndroidReport::default();
    if modules.is_empty() {
        return Ok(report);
    }
    rewrite_xml(survey, &modules, renames, journal, &mut report)?;
    add_imports(&modules, moved, journal, &mut report)?;
    Ok(report)
}

fn rewrite_xml(
    survey: &Survey,
    modules: &[Module],
    renames: &[(String, String)],
    journal: &mut Journal,
    report: &mut AndroidReport,
) -> Result<()> {
    if renames.is_empty() {
        return Ok(());
    }
    for path in &survey.android_xml {
        let Some(module) = owner(modules, path) else {
            continue;
        };
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("Cannot read {}", path.display()))?;
        let names = xml::Names {
            namespace: &module.namespace,
            renames,
            is_manifest: path.file_name().and_then(|name| name.to_str())
                == Some("AndroidManifest.xml"),
        };
        let updated = xml::rewrite(&text, &names)
            .with_context(|| format!("Cannot update the class names in {}", path.display()))?;
        if let Some(updated) = updated {
            journal.write_file(path, updated.as_bytes())?;
            report.edited.push(path.clone());
        }
    }
    Ok(())
}

/// A file that left its module's namespace package no longer sees `R` and
/// `BuildConfig` without an import.
fn add_imports(
    modules: &[Module],
    moved: &[MovedFile],
    journal: &mut Journal,
    report: &mut AndroidReport,
) -> Result<()> {
    for file in moved {
        let Some(module) = owner(modules, &file.to) else {
            continue;
        };
        let was_in_namespace =
            declared_package(&file.before).as_deref() == Some(module.namespace.as_str());
        let still_in_namespace =
            declared_package(&file.after).as_deref() == Some(module.namespace.as_str());
        if !was_in_namespace || still_in_namespace {
            continue;
        }
        if let Some(updated) = imports::add_generated_imports(&file.after, &module.namespace) {
            journal.write_file(&file.to, updated.as_bytes())?;
            report.edited.push(file.to.clone());
        }
    }
    Ok(())
}

/// Whether `<module>/src/<source set>/AndroidManifest.xml` exists.
fn has_manifest(survey: &Survey, module_dir: &Path) -> bool {
    survey.android_xml.iter().any(|xml| {
        xml.file_name().and_then(|name| name.to_str()) == Some("AndroidManifest.xml")
            && xml.ancestors().nth(3) == Some(module_dir)
            && xml
                .ancestors()
                .nth(2)
                .and_then(|src| src.file_name())
                .and_then(|name| name.to_str())
                == Some("src")
    })
}

/// The module whose directory holds the file (the deepest one wins).
fn owner<'a>(modules: &'a [Module], path: &Path) -> Option<&'a Module> {
    modules
        .iter()
        .filter(|module| path.starts_with(&module.dir))
        .max_by_key(|module| module.dir.components().count())
}

#[cfg(test)]
mod tests;
