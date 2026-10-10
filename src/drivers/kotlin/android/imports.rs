//! `R` and `BuildConfig` are generated into the module's namespace package, so
//! code in that package uses them without an import. A file that moves out of
//! it loses them ("Unresolved reference R"), and the language server does not
//! add the import, so this layer does.

const GENERATED: &[&str] = &["R", "BuildConfig"];

/// The text with an `import <namespace>.<Name>` added for every generated
/// class the file uses unqualified and does not import. `None` when nothing
/// had to be added.
pub fn add_generated_imports(text: &str, namespace: &str) -> Option<String> {
    let mut updated = text.to_string();
    let mut changed = false;
    for name in GENERATED {
        if uses_unqualified(&updated, name) && !imports(&updated, name) {
            updated = insert_import(&updated, &format!("import {namespace}.{name}"));
            changed = true;
        }
    }
    changed.then_some(updated)
}

/// `R.string.x` yes; `android.R.string.x`, `Other.R` and `SR.x` no. Lines that
/// are comments do not count.
fn uses_unqualified(text: &str, name: &str) -> bool {
    text.lines()
        .filter(|line| {
            let line = line.trim_start();
            !(line.starts_with("//") || line.starts_with('*') || line.starts_with("/*"))
        })
        .any(|line| {
            line.match_indices(name).any(|(at, _)| {
                let before = line[..at].chars().next_back();
                let after = line[at + name.len()..].chars().next();
                let own_word =
                    before.is_none_or(|ch| !(ch.is_alphanumeric() || ch == '_' || ch == '.'));
                own_word && after == Some('.')
            })
        })
}

/// Any import that ends in `.<name>`, whatever package it comes from: an
/// explicit import wins over the implicit one, so adding another would clash.
fn imports(text: &str, name: &str) -> bool {
    text.lines().any(|line| {
        let Some(path) = line.trim().strip_prefix("import ") else {
            return false;
        };
        let path = path
            .split(" as ")
            .next()
            .unwrap_or(path)
            .trim()
            .trim_end_matches(';');
        path.rsplit('.').next() == Some(name)
    })
}

/// Put the import in sorted position when the existing imports are sorted,
/// else after the last one; with no imports, below the package line.
pub(in crate::drivers::kotlin) fn insert_import(text: &str, import: &str) -> String {
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let import_lines: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.trim_start().starts_with("import "))
        .map(|(index, _)| index)
        .collect();
    let new_line = format!("{import}{newline}");
    let mut result: Vec<String> = lines.iter().map(|line| line.to_string()).collect();

    if import_lines.is_empty() {
        let package = lines
            .iter()
            .position(|line| line.trim_start().starts_with("package "));
        let after = package.map_or(0, |index| index + 1);
        // One blank line between package, imports and code.
        if lines.get(after).is_some_and(|line| line.trim().is_empty()) {
            result.splice(after + 1..after + 1, [new_line, newline.to_string()]);
        } else {
            result.splice(
                after..after,
                [newline.to_string(), new_line, newline.to_string()],
            );
        }
        return result.concat();
    }

    let keys: Vec<&str> = import_lines
        .iter()
        .map(|index| lines[*index].trim())
        .collect();
    let sorted = keys.windows(2).all(|pair| pair[0] <= pair[1]);
    let position = if sorted {
        import_lines
            .iter()
            .find(|index| lines[**index].trim() > import)
            .copied()
    } else {
        None
    };
    match position {
        Some(index) => result.insert(index, new_line),
        None => {
            let last = *import_lines.last().expect("checked above");
            if !result[last].ends_with('\n') {
                result[last].push_str(newline);
            }
            result.insert(last + 1, new_line);
        }
    }
    result.concat()
}

#[cfg(test)]
mod tests;
