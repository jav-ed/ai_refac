//! Turning a `WorkspaceEdit` from a language server into the new content of
//! each file, without writing anything. Writing goes through the journal.

use crate::drivers::lsp::text::apply_text_edits;
use anyhow::{Context, Result, bail};
use lsp_types::TextEdit;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use url::Url;

const BOM: char = '\u{FEFF}';

/// The edits of one file, as the server addressed it.
pub struct FileEdits {
    pub path: PathBuf,
    pub edits: Vec<TextEdit>,
}

/// The content a file gets once the edits are applied. `before` and `text`
/// are without the byte order mark, which `bytes` restores.
pub struct PlannedFile {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
    pub before: String,
    pub text: String,
}

/// One step of a `WorkspaceEdit`, in the order the server sent them.
pub enum Change {
    Edit(FileEdits),
    /// A file the server wants moved, for example the file of a renamed class.
    Rename {
        from: PathBuf,
        to: PathBuf,
    },
    /// A file the server wants created or deleted ("create" or "delete"),
    /// which no move or symbol rename of refac applies.
    Other {
        kind: String,
        path: PathBuf,
    },
}

/// Read the steps of a `WorkspaceEdit`, in the order the server sent them.
pub fn parse_changes(edit: &Value) -> Result<Vec<Change>> {
    let mut changes = Vec::new();
    if let Some(by_uri) = edit.get("changes").and_then(Value::as_object) {
        for (uri, edits) in by_uri {
            let mut per_file = BTreeMap::new();
            collect(&mut per_file, uri, edits)?;
            changes.extend(
                per_file
                    .into_iter()
                    .map(|(path, edits)| Change::Edit(FileEdits { path, edits })),
            );
        }
    }
    for change in edit
        .get("documentChanges")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        match change.get("kind").and_then(Value::as_str) {
            Some("rename") => changes.push(Change::Rename {
                from: file_path(
                    change["oldUri"]
                        .as_str()
                        .context("A rename without oldUri")?,
                )?,
                to: file_path(
                    change["newUri"]
                        .as_str()
                        .context("A rename without newUri")?,
                )?,
            }),
            Some(kind) => changes.push(Change::Other {
                kind: kind.to_string(),
                path: file_path(
                    change["uri"]
                        .as_str()
                        .context("A file operation without uri")?,
                )?,
            }),
            None => {
                let uri = change["textDocument"]["uri"]
                    .as_str()
                    .context("A text document edit without a uri")?;
                let mut per_file = BTreeMap::new();
                collect(&mut per_file, uri, &change["edits"])?;
                changes.extend(
                    per_file
                        .into_iter()
                        .map(|(path, edits)| Change::Edit(FileEdits { path, edits })),
                );
            }
        }
    }
    Ok(changes)
}

/// Read the text edits of a `WorkspaceEdit` that must not move files (the
/// answer to `willRenameFiles`: the client moves the files itself), merged
/// per file.
pub fn parse_workspace_edit(edit: &Value) -> Result<Vec<FileEdits>> {
    let mut per_file: BTreeMap<PathBuf, Vec<TextEdit>> = BTreeMap::new();
    for change in parse_changes(edit)? {
        match change {
            Change::Edit(file) => per_file.entry(file.path).or_default().extend(file.edits),
            Change::Rename { from, to } => bail!(
                "The language server asked for a rename file operation ({} to {}), which refac does not apply here",
                from.display(),
                to.display()
            ),
            Change::Other { kind, path } => bail!(
                "The language server asked for a {kind} file operation ({}), which refac does not apply",
                path.display()
            ),
        }
    }
    Ok(per_file
        .into_iter()
        .map(|(path, edits)| FileEdits { path, edits })
        .collect())
}

fn file_path(uri: &str) -> Result<PathBuf> {
    Url::parse(uri)
        .ok()
        .and_then(|url| url.to_file_path().ok())
        .with_context(|| format!("The server addressed a file that is not a local path: {uri}"))
}

fn collect(
    per_file: &mut BTreeMap<PathBuf, Vec<TextEdit>>,
    uri: &str,
    edits: &Value,
) -> Result<()> {
    let path = file_path(uri)?;
    let edits: Vec<TextEdit> = serde_json::from_value(edits.clone())
        .with_context(|| format!("Cannot read the text edits for {}", path.display()))?;
    per_file.entry(path).or_default().extend(edits);
    Ok(())
}

/// Compute the new content of every edited file. `locate` maps the path the
/// server used to the file that exists on disk now (an edit may address a
/// file under its destination before it has been moved there).
pub fn plan_files(
    edits: Vec<FileEdits>,
    locate: impl Fn(&Path) -> PathBuf,
) -> Result<Vec<PlannedFile>> {
    let mut planned = Vec::new();
    for file in edits {
        let path = locate(&file.path);
        let raw = crate::drivers::symbol::view::read_to_string(&path).with_context(|| {
            format!("The server edited {}, which cannot be read", path.display())
        })?;
        // The server counts positions from the first real character.
        let (bom, content) = match raw.strip_prefix(BOM) {
            Some(rest) => (true, rest),
            None => (false, raw.as_str()),
        };
        let updated = apply_text_edits(content, file.edits)
            .with_context(|| format!("Cannot apply the server's edits to {}", path.display()))?;
        let bytes = if bom {
            format!("{BOM}{updated}").into_bytes()
        } else {
            updated.clone().into_bytes()
        };
        planned.push(PlannedFile {
            path,
            bytes,
            before: content.to_string(),
            text: updated,
        });
    }
    Ok(planned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn uri(path: &Path) -> String {
        Url::from_file_path(path).unwrap().to_string()
    }

    fn edit(line: u32, from: u32, to: u32, text: &str) -> Value {
        json!({ "range": { "start": { "line": line, "character": from }, "end": { "line": line, "character": to } }, "newText": text })
    }

    #[test]
    fn reads_changes_and_document_changes_per_file() {
        let (a, b) = (Path::new("/p/A.kt"), Path::new("/p/B.kt"));
        let answer = json!({
            "changes": { uri(a): [edit(0, 0, 1, "x")] },
            "documentChanges": [
                { "textDocument": { "uri": uri(b), "version": null }, "edits": [edit(1, 0, 1, "y")] },
                { "textDocument": { "uri": uri(a), "version": null }, "edits": [edit(2, 0, 1, "z")] },
            ],
        });
        let files = parse_workspace_edit(&answer).unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, a);
        assert_eq!(files[0].edits.len(), 2);
        assert_eq!(files[1].path, b);
    }

    #[test]
    fn a_file_operation_is_refused() {
        let answer = json!({ "documentChanges": [{ "kind": "rename", "oldUri": "file:///a", "newUri": "file:///b" }] });
        let error = parse_workspace_edit(&answer).err().unwrap().to_string();
        assert!(error.contains("rename"), "{error}");
    }

    #[test]
    fn parse_changes_keeps_the_order_of_edits_and_file_renames() {
        let (a, b) = (Path::new("/p/A.kt"), Path::new("/p/B.kt"));
        let answer = json!({
            "documentChanges": [
                { "textDocument": { "uri": uri(a), "version": null }, "edits": [edit(0, 0, 1, "x")] },
                { "kind": "rename", "oldUri": uri(a), "newUri": uri(b) },
            ],
        });
        let changes = parse_changes(&answer).unwrap();
        assert_eq!(changes.len(), 2);
        assert!(matches!(&changes[0], Change::Edit(file) if file.path == a));
        assert!(matches!(&changes[1], Change::Rename { from, to } if from == a && to == b));
    }

    #[test]
    fn creating_and_deleting_files_are_reported_as_other_changes() {
        for kind in ["create", "delete"] {
            let answer = json!({ "documentChanges": [{ "kind": kind, "uri": "file:///p/A.kt" }] });
            let changes = parse_changes(&answer).unwrap();
            assert!(
                matches!(&changes[0], Change::Other { kind: found, path } if found == kind && path == Path::new("/p/A.kt"))
            );
            let error = parse_workspace_edit(&answer).err().unwrap().to_string();
            assert!(error.contains(kind), "{error}");
        }
    }

    #[test]
    fn a_rename_without_both_uris_is_an_error() {
        let answer =
            json!({ "documentChanges": [{ "kind": "rename", "oldUri": "file:///p/A.kt" }] });
        assert!(parse_changes(&answer).is_err());
    }

    #[test]
    fn a_remote_uri_is_refused() {
        let answer = json!({ "changes": { "https://example.com/A.kt": [] } });
        assert!(parse_workspace_edit(&answer).is_err());
    }

    #[test]
    fn plans_new_content_and_keeps_a_byte_order_mark() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("A.kt");
        std::fs::write(&file, format!("{BOM}package a.b\r\nclass A\r\n")).unwrap();
        let edits = vec![FileEdits {
            path: file.clone(),
            edits: vec![serde_json::from_value(edit(0, 8, 11, "c.d")).unwrap()],
        }];

        let planned = plan_files(edits, Path::to_path_buf).unwrap();
        assert_eq!(planned[0].text, "package c.d\r\nclass A\r\n");
        assert_eq!(
            String::from_utf8(planned[0].bytes.clone()).unwrap(),
            format!("{BOM}package c.d\r\nclass A\r\n")
        );
    }

    #[test]
    fn an_edit_of_a_missing_file_is_an_error() {
        let edits = vec![FileEdits {
            path: PathBuf::from("/definitely/not/here/A.kt"),
            edits: Vec::new(),
        }];
        let error = plan_files(edits, Path::to_path_buf)
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("cannot be read"), "{error}");
    }
}
