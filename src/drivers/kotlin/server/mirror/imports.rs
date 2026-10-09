//! Undoing what the mirror's missing libraries make the server do to imports.
//!
//! A move or a rename makes the server tidy the imports of the files it
//! edits. In the mirror an import of Compose or Android is unresolved, and so
//! is anything used through it: `Modifier.padding(...)` is not a use of
//! `androidx.compose.foundation.layout.padding` for a server that cannot find
//! `Modifier`, and `by remember { ... }` reaches `getValue` through its import
//! alone. The import then looks unused and is deleted, and the real project no
//! longer compiles. So the answer of the server is checked against the file
//! before: an import it removed stays, unless the change calls for it. A
//! move calls for the lines that point into the package a moved file left, a
//! rename for the lines that name the symbol it renamed.

use crate::drivers::kotlin::android::moved::join_relative;
use crate::drivers::kotlin::declarations::declared_package;
use crate::drivers::lsp::text::{TextIndex, apply_text_edits};
use anyhow::{Context, Result, bail};
use lsp_types::{Range, TextEdit};
use serde_json::Value;
use similar::{Algorithm, DiffTag, capture_diff_slices};
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
        /// The packages the moved files declared before the move.
        packages: Vec<String>,
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
        let mut packages = Vec::new();
        for file in request["files"].as_array().into_iter().flatten() {
            let old = file_path(file["oldUri"].as_str().context("A move without oldUri")?)?;
            let new = file_path(file["newUri"].as_str().context("A move without newUri")?)?;
            for entry in WalkDir::new(&old) {
                let path = entry?.into_path();
                if path.extension().is_some_and(|extension| extension == "kt")
                    && let Some(package) = declared_package(&std::fs::read_to_string(&path)?)
                    && !packages.contains(&package)
                {
                    packages.push(package);
                }
            }
            pairs.push((old, new));
        }
        Ok(Self::Move { pairs, packages })
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
            // It points into the package of a file that moved away.
            Self::Move { packages, .. } => packages.iter().any(|package| {
                fqn.strip_prefix(package.as_str())
                    .is_some_and(|rest| rest.starts_with('.'))
            }),
            // It names the renamed symbol, or something inside it.
            Self::Rename { name } => fqn.split('.').any(|part| part == name),
        }
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
        let kept = restore(&before, &after, &|fqn| self.accounts_for(fqn));
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

/// The imported name of an `import` line, without alias or wildcard.
fn import_fqn(line: &str) -> Option<&str> {
    let rest = line.trim().strip_prefix("import")?;
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let rest = rest.split("//").next()?.trim().trim_end_matches(';');
    rest.split_whitespace().next()
}

/// `after`, plus every import line of `before` that it lost without
/// `accounted` calling for the loss. A restored line goes after the import
/// that stood before it, so the block keeps its order.
fn restore(before: &str, after: &str, accounted: &dyn Fn(&str) -> bool) -> String {
    let mut result: Vec<String> = after.split_inclusive('\n').map(str::to_string).collect();
    let mut anchor: Option<usize> = None;
    for line in before.split_inclusive('\n') {
        let Some(fqn) = import_fqn(line) else {
            continue;
        };
        let key = line.trim();
        if let Some(at) = result.iter().position(|kept| kept.trim() == key) {
            anchor = Some(at);
            continue;
        }
        if accounted(fqn) {
            continue;
        }
        let at = anchor.map_or_else(|| first_import_place(&result), |anchor| anchor + 1);
        let mut restored = line.to_string();
        if !restored.ends_with('\n') {
            restored.push('\n');
        }
        result.insert(at, restored);
        anchor = Some(at);
    }
    result.concat()
}

/// Where the import block starts: at its first import, else after the
/// `package` line, else at the top.
fn first_import_place(lines: &[String]) -> usize {
    lines
        .iter()
        .position(|line| import_fqn(line).is_some())
        .or_else(|| {
            lines
                .iter()
                .position(|line| line.trim_start().starts_with("package "))
                .map(|at| at + 1)
        })
        .unwrap_or(0)
}

/// The edits that turn `before` into `after`, one per changed run of tokens
/// (names, punctuation, blanks): as fine as the server's own, so a rename
/// that changes a name changes just the name.
fn edits_between(before: &str, after: &str) -> Vec<TextEdit> {
    let (old, new) = (tokens(before), tokens(after));
    let (old_starts, new_starts) = (offsets(&old), offsets(&new));
    let index = TextIndex::new(before);
    capture_diff_slices(Algorithm::Myers, &old, &new)
        .iter()
        .filter_map(|op| {
            let (tag, old_range, new_range) = op.as_tag_tuple();
            (tag != DiffTag::Equal).then(|| TextEdit {
                range: Range::new(
                    index.position(old_starts[old_range.start]),
                    index.position(old_starts[old_range.end]),
                ),
                new_text: after[new_starts[new_range.start]..new_starts[new_range.end]].to_string(),
            })
        })
        .collect()
}

/// Names (and numbers) and runs of blanks are one token; every other
/// character, newlines included, is one by itself.
fn tokens(text: &str) -> Vec<&str> {
    let class = |ch: char| match ch {
        _ if ch.is_alphanumeric() || ch == '_' => 1,
        ' ' | '\t' => 2,
        _ => 0,
    };
    let mut tokens = Vec::new();
    let (mut start, mut current) = (0, None);
    for (at, ch) in text.char_indices() {
        let kind = class(ch);
        if let Some(previous) = current {
            if kind != 0 && kind == previous {
                continue;
            }
            tokens.push(&text[start..at]);
            start = at;
        }
        current = Some(kind);
    }
    if start < text.len() {
        tokens.push(&text[start..]);
    }
    tokens
}

/// Where each token starts, and the end of the last one.
fn offsets(tokens: &[&str]) -> Vec<usize> {
    let mut offsets = vec![0];
    for token in tokens {
        offsets.push(offsets.last().copied().unwrap_or(0) + token.len());
    }
    offsets
}

#[cfg(test)]
mod tests;
