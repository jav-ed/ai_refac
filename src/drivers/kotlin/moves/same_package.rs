//! Imports a moved file needs for what it saw in its old package.
//!
//! A Kotlin file sees every top-level declaration of its own package without an
//! import. When it moves to another package it loses that, and the language
//! server adds the imports it can resolve. It does not add all of them: in the
//! Multiplatform mirror an `expect fun` and its `actual fun` are two
//! declarations of one name, which the server takes for an ambiguous call and
//! leaves without an import (found with `platformName()`, 2026-10-10), and the
//! build of the real project then fails with "Unresolved reference". This
//! layer runs after the server, looks at names only (so overloads, `expect`
//! and `actual` are no problem), and adds `import <old package>.<name>` for
//! every name the moved file uses, that a file of the old package declares at
//! the top level, and that the file does not import already.
//!
//! Operators and delegates: the server imports the operators written as
//! symbols (`+`, `<`, `in`) but not `a[i]`, `for (x in y)`, `val (a, b) = p`
//! or `by`; `convention` finds those uses, and the extension has to be an
//! `operator` of the old package to be imported for them.
//!
//! The other direction (`users`): a file that stayed behind gets the imports of
//! the operator functions that moved away with their declaring file.
//!
//! Other Gradle modules: a plain name of another module is the server's job;
//! only an operator the syntax calls is taken from a module that the module's
//! build file names as a direct project dependency (`dependency`).

mod convention;
mod dependency;
#[cfg(test)]
mod dependency_tests;
mod imports;
#[cfg(test)]
mod scan_tests;
#[cfg(test)]
mod tests;
mod used;
mod users;

use crate::drivers::kotlin::android::imports::insert_import;
use crate::drivers::kotlin::android::moved::MovedFile;
use crate::drivers::kotlin::android::survey::Survey;
use crate::drivers::kotlin::declarations::declared_package;
use crate::drivers::kotlin::declarations::top_level::{Declaration, Kind, declarations};
use crate::drivers::lsp::rename::write::journal::FileWrite;
use anyhow::Result;
use imports::Imports;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use used::used_names;

/// A top-level name of a package that some moved file left.
struct Candidate {
    declaration: Declaration,
    /// The package the file declared when the name was visible to the moved
    /// file.
    package: String,
    /// The package it declares now: a moved file may have left too.
    home: Option<String>,
    path: PathBuf,
    module: Option<PathBuf>,
    set: Option<String>,
}

/// The imports to add, as changes to `writes` (the file's text there, when
/// the Android layer has written it already, is built upon). Returns a note
/// for every file that got some; `root` is the Gradle root the notes show
/// paths relative to.
pub fn plan(
    root: &Path,
    survey: &Survey,
    moved: &[MovedFile],
    writes: &mut Vec<FileWrite>,
) -> Result<Vec<String>> {
    let left: BTreeSet<String> = moved.iter().filter_map(left_package).collect();
    if left.is_empty() {
        return Ok(Vec::new());
    }
    let candidates = candidates(survey, moved, &left);
    let mut notes = Vec::new();
    for file in moved {
        let Some(old) = left_package(file) else {
            continue;
        };
        let existing = writes.iter().position(|write| write.path == file.to);
        let text = match existing {
            Some(index) => String::from_utf8_lossy(&writes[index].bytes).into_owned(),
            None => file.after.clone(),
        };
        let module = module_of(survey, &file.to);
        let set = source_set(module, &file.to);
        // A name of the same module is one the file could write; a declaration
        // of another module only counts for an operator the syntax calls, and
        // only when the file's module names that module as a dependency.
        let seen = candidates.iter().filter_map(|candidate| {
            if candidate.package != old
                || candidate.path == file.to
                || !visible(set.as_deref(), candidate.set.as_deref())
            {
                return None;
            }
            let own = candidate.module.as_deref() == module;
            let reachable = own
                || (candidate.declaration.operator
                    && candidate.declaration.extension
                    && module
                        .zip(candidate.module.as_deref())
                        .is_some_and(|(this, other)| dependency::names(root, this, other)));
            reachable.then_some((candidate, own))
        });
        let wanted = missing_imports(&text, declared_package(&file.after).as_deref(), seen);
        if wanted.is_empty() {
            continue;
        }
        write_imports(writes, &file.to, text, &wanted);
        notes.push(format!(
            "Added imports to {}: {}. The file used these names from its old package {old} without an import, and the server left them out.",
            file.to.strip_prefix(root).unwrap_or(&file.to).display(),
            wanted.join(", ")
        ));
    }
    notes.extend(users::plan(root, survey, moved, writes));
    Ok(notes)
}

/// Adds `wanted` (full names) as imports to `text`, the file's text now, and
/// records that as the file's write, replacing an earlier one of the same file.
fn write_imports(writes: &mut Vec<FileWrite>, path: &Path, text: String, wanted: &[String]) {
    let updated = wanted.iter().fold(text, |text, import| {
        insert_import(&text, &format!("import {import}"))
    });
    match writes.iter().position(|write| write.path == path) {
        Some(index) => {
            writes[index].bytes = updated.into_bytes();
            writes[index].changes += wanted.len();
        }
        None => writes.push(FileWrite {
            path: path.to_path_buf(),
            bytes: updated.into_bytes(),
            changes: wanted.len(),
        }),
    }
}

/// The package a moved file declared before and no longer declares.
fn left_package(file: &MovedFile) -> Option<String> {
    let old = declared_package(&file.before)?;
    (declared_package(&file.after).as_deref() != Some(old.as_str())).then_some(old)
}

/// The full names to import: used by `text`, declared in the old package by
/// one of `candidates`, not imported yet, not in the package the file is in
/// now.
fn missing_imports<'a>(
    text: &str,
    now: Option<&str>,
    candidates: impl Iterator<Item = (&'a Candidate, bool)>,
) -> Vec<String> {
    let used = used_names(text);
    let own = declarations(text);
    let imports = Imports::parse(text);
    let mut by_name: BTreeMap<&str, Vec<(&Candidate, bool)>> = BTreeMap::new();
    for (candidate, same_module) in candidates {
        by_name
            .entry(&candidate.declaration.name)
            .or_default()
            .push((candidate, same_module));
    }
    let mut wanted = BTreeSet::new();
    for (name, found) in by_name {
        let in_module = || found.iter().filter(|(_, same_module)| *same_module);
        let written = (used.plain.contains(name) && in_module().next().is_some())
            || (used.member.contains(name)
                && in_module().any(|(candidate, _)| candidate.declaration.extension))
            || found
                .iter()
                .any(|(candidate, _)| convention::reaches(&candidate.declaration, &used));
        // A type has one meaning: the file's own class of that name is the
        // one it meant, and two classes of one name in different homes
        // cannot both have been meant.
        let own_type = own
            .iter()
            .any(|declaration| declaration.name == name && declaration.kind == Kind::Type);
        let homes: BTreeSet<&str> = found
            .iter()
            .filter_map(|(candidate, _)| candidate.home.as_deref())
            .filter(|home| Some(*home) != now)
            .collect();
        let ambiguous_type = homes.len() > 1
            && found
                .iter()
                .any(|(candidate, _)| candidate.declaration.kind == Kind::Type);
        if !written || own_type || ambiguous_type || imports.binds(name) {
            continue;
        }
        wanted.extend(
            homes
                .into_iter()
                .filter(|home| !imports.star(home))
                .map(|home| format!("{home}.{name}")),
        );
    }
    wanted.into_iter().collect()
}

/// Every top-level name of the packages in `left`, from the sources of the
/// project as they are now. A moved file counts with the package it had
/// before and the one it has now.
fn candidates(survey: &Survey, moved: &[MovedFile], left: &BTreeSet<String>) -> Vec<Candidate> {
    let mut found = Vec::new();
    for path in &survey.sources {
        let (text, package, home) = match moved.iter().find(|file| &file.to == path) {
            Some(file) => (
                file.after.clone(),
                declared_package(&file.before),
                declared_package(&file.after),
            ),
            None => {
                let text = match std::fs::read(path) {
                    Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                    Err(_) => continue,
                };
                let package = declared_package(&text);
                (text, package.clone(), package)
            }
        };
        let Some(package) = package.filter(|package| left.contains(package)) else {
            continue;
        };
        let declared = if path
            .extension()
            .is_some_and(|extension| extension == "java")
        {
            java_class(path)
        } else {
            declarations(&text)
        };
        let module = module_of(survey, path).map(Path::to_path_buf);
        let set = source_set(module.as_deref(), path);
        found.extend(declared.into_iter().map(|declaration| Candidate {
            declaration,
            package: package.clone(),
            home: home.clone(),
            path: path.clone(),
            module: module.clone(),
            set: set.clone(),
        }));
    }
    found
}

/// A Java file declares the public class that carries its name.
fn java_class(path: &Path) -> Vec<Declaration> {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(|name| Declaration {
            name: name.to_string(),
            kind: Kind::Type,
            extension: false,
            operator: false,
            receiver: None,
        })
        .into_iter()
        .collect()
}

/// The Gradle module that holds the file: the deepest directory with a build
/// file above it.
fn module_of<'a>(survey: &'a Survey, path: &Path) -> Option<&'a Path> {
    survey
        .build_files
        .iter()
        .filter_map(|build_file| build_file.parent())
        .filter(|dir| path.starts_with(dir))
        .max_by_key(|dir| dir.components().count())
}

/// `<module>/src/<set>/...`: the name of the source set.
fn source_set(module: Option<&Path>, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(module?).ok()?;
    let mut parts = relative.components();
    if parts.next()?.as_os_str() != "src" {
        return None;
    }
    Some(parts.next()?.as_os_str().to_str()?.to_string())
}

/// Whether code in source set `from` can see the declarations of `declared`.
/// A test set sees its main set; `commonMain` is seen by all; a platform set
/// is not seen by the others. Unknown sets are given the benefit of the doubt:
/// a name that is used and declared in the old package is most likely meant.
fn visible(from: Option<&str>, declared: Option<&str>) -> bool {
    let (Some(from), Some(declared)) = (from, declared) else {
        return true;
    };
    if from == declared || declared == "main" || declared == "commonMain" {
        return true;
    }
    let test = from.eq_ignore_ascii_case("test") || from.ends_with("Test");
    test && (declared == "commonTest"
        || from
            .strip_suffix("Test")
            .is_some_and(|platform| declared == format!("{platform}Main")))
}
