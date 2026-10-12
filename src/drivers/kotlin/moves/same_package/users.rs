//! Files that stayed behind and use an operator function that moved away.
//!
//! The same case from the other side. A moved file that declares
//! `operator fun Qty.get(i: Int)` takes it out of its package, and the files
//! left in that package (or importing it with a star) still write `qty[0]`.
//! The server imports the operators written as symbols for them (`compareTo`
//! for `<`) but not `get`, `set`, `componentN`, `iterator`, `getValue` or
//! `setValue` (found on the real server, 2026-10-12), and the build fails with
//! "No 'get' operator method providing array access". This layer adds
//! `import <new package>.<name>` to every such file that uses the syntax.

use super::convention::{is_convention, reaches};
use super::dependency;
use super::imports::Imports;
use super::used::used_names;
use super::{left_package, module_of, source_set, visible, write_imports};
use crate::drivers::kotlin::android::moved::MovedFile;
use crate::drivers::kotlin::android::survey::Survey;
use crate::drivers::kotlin::declarations::declared_package;
use crate::drivers::kotlin::declarations::top_level::{Declaration, declarations};
use crate::drivers::lsp::rename::write::journal::FileWrite;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// An operator function that left its package with a moved file.
struct Gone {
    declaration: Declaration,
    old: String,
    now: String,
    module: Option<PathBuf>,
    set: Option<String>,
}

/// Imports for the files that stayed; a note for each one that got some.
pub(super) fn plan(
    root: &Path,
    survey: &Survey,
    moved: &[MovedFile],
    writes: &mut Vec<FileWrite>,
) -> Vec<String> {
    let gone = gone(survey, moved);
    if gone.is_empty() {
        return Vec::new();
    }
    let mut notes = Vec::new();
    for path in &survey.sources {
        if moved.iter().any(|file| &file.to == path) {
            continue;
        }
        let text = match writes.iter().find(|write| &write.path == path) {
            Some(write) => String::from_utf8_lossy(&write.bytes).into_owned(),
            None => match std::fs::read(path) {
                Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                Err(_) => continue,
            },
        };
        let module = module_of(survey, path);
        let set = source_set(module, path);
        let (package, imports, used) = (
            declared_package(&text),
            Imports::parse(&text),
            used_names(&text),
        );
        let wanted: BTreeSet<String> = gone
            .iter()
            .filter(|function| {
                (package.as_deref() == Some(function.old.as_str()) || imports.star(&function.old))
                    && package.as_deref() != Some(function.now.as_str())
                    && (module == function.module.as_deref()
                        || module.zip(function.module.as_deref()).is_some_and(
                            |(this, declaring)| dependency::names(root, this, declaring),
                        ))
                    && visible(set.as_deref(), function.set.as_deref())
                    && reaches(&function.declaration, &used)
                    && !imports.binds(&function.declaration.name)
            })
            .map(|function| format!("{}.{}", function.now, function.declaration.name))
            .collect();
        if wanted.is_empty() {
            continue;
        }
        let wanted: Vec<String> = wanted.into_iter().collect();
        write_imports(writes, path, text, &wanted);
        notes.push(format!(
            "Added imports to {}: {}. The file calls these operator functions through its syntax (an index, a loop, a destructuring or a delegate) without writing their names, and the file that declares them moved to another package.",
            path.strip_prefix(root).unwrap_or(path).display(),
            wanted.join(", ")
        ));
    }
    notes
}

/// The operator extensions that every moved file took out of its package.
fn gone(survey: &Survey, moved: &[MovedFile]) -> Vec<Gone> {
    let mut found = Vec::new();
    for file in moved {
        let (Some(old), Some(now)) = (left_package(file), declared_package(&file.after)) else {
            continue;
        };
        let module = module_of(survey, &file.to);
        let set = source_set(module, &file.to);
        found.extend(
            declarations(&file.after)
                .into_iter()
                .filter(|declaration| {
                    declaration.operator
                        && declaration.extension
                        && is_convention(&declaration.name)
                })
                .map(|declaration| Gone {
                    declaration,
                    old: old.clone(),
                    now: now.clone(),
                    module: module.map(Path::to_path_buf),
                    set: set.clone(),
                }),
        );
    }
    found
}
