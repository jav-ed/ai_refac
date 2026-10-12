//! Undoing what the server does to imports and layout that nobody asked for.
//!
//! A move or a rename makes the server tidy the imports of the files it
//! edits, and it reformats code there too. In the mirror an import of Compose
//! or Android is unresolved, and so is anything used through it:
//! `Modifier.padding(...)` is not a use of
//! `androidx.compose.foundation.layout.padding` for a server that cannot find
//! `Modifier`, and `by remember { ... }` reaches `getValue` through its import
//! alone. The import then looks unused and is deleted, and the real project no
//! longer compiles. A plain project loses unused imports as well, which does
//! not break the build but is not what a move is for. So the answer of the
//! server is checked against the file before: an import it removed stays,
//! unless the change calls for it, and a change of layout alone is undone
//! (`untouched`). A move calls for the imports of what a moved file declares,
//! which it rewrites; a rename for the lines that name the symbol it renamed.
//! The import of something that stayed in the old package is not the move's.

use super::edits::edits_between;
use super::operators::undo_operator_calls;
use super::restore::restore;
use super::untouched::undo_reformatting;
use crate::drivers::kotlin::android::moved::join_relative;
use crate::drivers::kotlin::declarations::declared_package;
use crate::drivers::kotlin::declarations::top_level::declarations;
use crate::drivers::kotlin::project::{package_of_dir, source_root};
use crate::drivers::lsp::text::{TextIndex, apply_text_edits};
use anyhow::{Context, Result, bail};
use lsp_types::TextEdit;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use url::Url;
use walkdir::WalkDir;

const BOM: char = '\u{FEFF}';

/// Rewrites the `WorkspaceEdit` the server answered to `request`, in the
/// mirror's paths: every file keeps the imports the change did not call for,
/// and the edits that remain are the smallest ones. Only the answers to
/// `workspace/willRenameFiles` and `textDocument/rename` are touched. `real`
/// names the file of the project that a file of the mirror stands for.
pub fn keep_imports(
    method: &str,
    answer: &mut Value,
    request: &Value,
    real: &dyn Fn(&Path) -> PathBuf,
) -> Result<()> {
    let Some(cause) = Cause::read(method, request)? else {
        return Ok(());
    };
    if let Some(by_uri) = answer.get_mut("changes").and_then(Value::as_object_mut) {
        for (uri, edits) in std::mem::take(by_uri) {
            let edits = cause.rewrite(&uri, edits, real)?;
            if !edits.is_empty() {
                by_uri.insert(uri, Value::Array(edits));
            }
        }
    }
    if let Some(changes) = answer
        .get_mut("documentChanges")
        .and_then(Value::as_array_mut)
    {
        let mut kept = Vec::new();
        for mut change in std::mem::take(changes) {
            let uri = change["textDocument"]["uri"].as_str().map(str::to_string);
            let Some(uri) = uri.filter(|_| change.get("edits").is_some()) else {
                // A file operation, not a text edit.
                kept.push(change);
                continue;
            };
            let rewritten = cause.rewrite(&uri, change["edits"].take(), real)?;
            if !rewritten.is_empty() {
                change["edits"] = Value::Array(rewritten);
                kept.push(change);
            }
        }
        *changes = kept;
    }
    Ok(())
}

/// What the request asked the server to change.
enum Cause {
    Move {
        /// (old, new) of each file or folder.
        pairs: Vec<(PathBuf, PathBuf)>,
        /// What the moved files declare at the top level, by the package they
        /// declared it in before the move, with the package that holds it now
        /// when the move is only a change of place (a file that is renamed
        /// takes its class's name along, so its imports cannot be written
        /// down without the server).
        left: BTreeMap<String, BTreeMap<String, Option<String>>>,
    },
    Rename {
        /// The name the symbol had.
        name: String,
    },
}

impl Cause {
    fn read(method: &str, request: &Value) -> Result<Option<Self>> {
        match method {
            "workspace/willRenameFiles" => Self::read_move(request).map(Some),
            "textDocument/rename" => Self::read_rename(request).map(Some),
            _ => Ok(None),
        }
    }

    fn read_move(request: &Value) -> Result<Self> {
        let mut pairs = Vec::new();
        let mut left: BTreeMap<String, BTreeMap<String, Option<String>>> = BTreeMap::new();
        for file in request["files"].as_array().into_iter().flatten() {
            let old = file_path(file["oldUri"].as_str().context("A move without oldUri")?)?;
            let new = file_path(file["newUri"].as_str().context("A move without newUri")?)?;
            let renamed = old.is_file() && old.file_name() != new.file_name();
            for entry in WalkDir::new(&old) {
                let path = entry?.into_path();
                if path.extension().is_none_or(|extension| extension != "kt") {
                    continue;
                }
                let text = std::fs::read_to_string(&path)
                    .with_context(|| format!("Cannot read {}", path.display()))?;
                let Some(package) = declared_package(&text) else {
                    continue;
                };
                let target = match path.strip_prefix(&old) {
                    Ok(rest) if !rest.as_os_str().is_empty() => new.join(rest),
                    _ => new.clone(),
                };
                let now = if renamed { None } else { package_in(&target) };
                let names = left.entry(package).or_default();
                for declaration in declarations(&text) {
                    names.insert(declaration.name, now.clone());
                }
            }
            pairs.push((old, new));
        }
        Ok(Self::Move { pairs, left })
    }

    /// The symbol is named by the position of the request: the identifier
    /// under it in the file as it is.
    fn read_rename(request: &Value) -> Result<Self> {
        let path = file_path(
            request["textDocument"]["uri"]
                .as_str()
                .context("A rename without a document")?,
        )?;
        let position: lsp_types::Position = serde_json::from_value(request["position"].clone())
            .context("A rename without a position")?;
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("Cannot read {}", path.display()))?;
        let text = text.trim_start_matches(BOM);
        let at = TextIndex::new(text).offset(position)?;
        let word = |ch: char| ch.is_alphanumeric() || ch == '_';
        let start = text[..at].rfind(|ch| !word(ch)).map_or(0, |i| i + 1);
        let end = text[at..]
            .find(|ch| !word(ch))
            .map_or(text.len(), |i| at + i);
        let name = &text[start..end];
        if name.is_empty() {
            bail!(
                "The rename at {position:?} of {} names no identifier",
                path.display()
            );
        }
        Ok(Self::Rename {
            name: name.to_string(),
        })
    }

    /// An import the change rewrites or drops on purpose.
    fn accounts_for(&self, fqn: &str) -> bool {
        match self {
            // It names something a moved file declared (or all of its
            // package: `import old.*`), so the move rewrites it. An import of
            // the same package that names something that stayed is not the
            // move's to touch, used or not.
            Self::Move { left, .. } => left.iter().any(|(package, names)| {
                after_package(fqn, package)
                    .and_then(|rest| rest.split('.').next())
                    .is_some_and(|first| first == "*" || names.contains_key(first))
            }),
            // It names the renamed symbol, or something inside it.
            Self::Rename { name } => fqn.split('.').any(|part| part == name),
        }
    }

    /// What an import the server deleted becomes when the move rewrites it
    /// instead: a moved class that the file only imported, never used, is
    /// deleted as unused by the server, and the move would have pointed the
    /// import to the new package. Not in the package the file is in now,
    /// where it would be redundant.
    fn rewritten(&self, fqn: &str, own_package: Option<&str>) -> Option<String> {
        let Self::Move { left, .. } = self else {
            return None;
        };
        left.iter().find_map(|(package, names)| {
            let rest = after_package(fqn, package)?;
            let now = names.get(rest.split('.').next()?)?.as_deref()?;
            (Some(now) != own_package).then(|| format!("{now}.{rest}"))
        })
    }

    /// The file as the server saw it, and its place: an edit may address the
    /// file under its new name before it has been moved there.
    fn before(&self, path: &Path) -> Result<(PathBuf, String)> {
        let mut at = path.to_path_buf();
        if let Self::Move { pairs, .. } = self
            && !at.exists()
        {
            for (old, new) in pairs {
                if let Ok(rest) = path.strip_prefix(new) {
                    at = join_relative(old, rest);
                }
            }
        }
        let text = std::fs::read_to_string(&at).with_context(|| {
            format!(
                "The server edited {}, which refac cannot read",
                at.display()
            )
        })?;
        Ok((at, text.trim_start_matches(BOM).to_string()))
    }

    /// The edits of one file after its removed imports are put back. When no
    /// import was lost the server's own edits stay as they are: the rename
    /// engine checks each of them against the places that refer to the symbol.
    fn rewrite(
        &self,
        uri: &str,
        edits: Value,
        real: &dyn Fn(&Path) -> PathBuf,
    ) -> Result<Vec<Value>> {
        let (at, before) = self.before(&file_path(uri)?)?;
        let sent: Vec<TextEdit> = serde_json::from_value(edits)
            .with_context(|| format!("The server sent edits for {uri} that refac cannot read"))?;
        let after = apply_text_edits(&before, sent.clone())?;
        let own_package = declared_package(&after);
        let kept = restore(&before, &after, &|fqn| self.accounts_for(fqn), &|fqn| {
            self.rewritten(fqn, own_package.as_deref())
        });
        let kept = undo_operator_calls(&before, &kept);
        let kept = undo_reformatting(&before, &kept);
        let edits = if kept == after {
            sent
        } else {
            let edits = edits_between(&before, &kept);
            if apply_text_edits(&before, edits.clone())? != kept {
                bail!(
                    "The edits refac made for {} do not give the text it meant",
                    at.display()
                );
            }
            edits
        };
        check_against_real(&before, &edits, &real(&at))?;
        edits
            .into_iter()
            .map(|edit| Ok(serde_json::to_value(edit)?))
            .collect()
    }
}

/// The server saw the mirror's text, in which `expect` and `actual` are
/// blanked. Every edit is applied to the real file at the same position, so
/// the text it replaces must be the same in both.
fn check_against_real(before: &str, edits: &[TextEdit], real: &Path) -> Result<()> {
    let real_text =
        std::fs::read_to_string(real).with_context(|| format!("Cannot read {}", real.display()))?;
    let real_text = real_text.trim_start_matches(BOM);
    let (mirror_index, real_index) = (TextIndex::new(before), TextIndex::new(real_text));
    for edit in edits {
        let mirror_span =
            mirror_index.offset(edit.range.start)?..mirror_index.offset(edit.range.end)?;
        let real_span = real_index.offset(edit.range.start)?..real_index.offset(edit.range.end)?;
        if before.get(mirror_span) != real_text.get(real_span) {
            bail!(
                "The server's edit at line {} of {} replaces text with an `expect` or `actual` modifier, which the mirror hides from it; refac would overwrite the modifier. Make this change by hand.",
                edit.range.start.line + 1,
                real.display()
            );
        }
    }
    Ok(())
}

fn file_path(uri: &str) -> Result<PathBuf> {
    Url::parse(uri)
        .ok()
        .and_then(|url| url.to_file_path().ok())
        .with_context(|| format!("{uri} is not a file URI"))
}

/// What follows `package.` in an imported name, if it is in that package.
fn after_package<'a>(fqn: &'a str, package: &str) -> Option<&'a str> {
    fqn.strip_prefix(package)?.strip_prefix('.')
}

/// The package that the directory of a file under a source root stands for.
fn package_in(file: &Path) -> Option<String> {
    package_of_dir(&source_root(file)?, file.parent()?)
}

#[cfg(test)]
mod tests;
