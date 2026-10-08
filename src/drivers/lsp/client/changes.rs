//! Turning a server's `WorkspaceEdit` into ordered changes and applying them.
//! Planning (`collect_pending_changes`) touches nothing, so a caller can check
//! the plan before `apply_pending_changes` writes it.

use super::resource_ops::apply_resource_op;
use crate::drivers::lsp::text::apply_text_edits;
use anyhow::Result;
use lsp_types::{
    AnnotatedTextEdit, DocumentChangeOperation, DocumentChanges, OneOf, ResourceOp,
    TextDocumentEdit, TextEdit, Uri, WorkspaceEdit,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use url::Url;

/// Collect and apply a workspace edit in one go.
pub async fn apply_workspace_edit(
    edit: WorkspaceEdit,
    pending_moves: &HashMap<PathBuf, PathBuf>,
) -> Result<(Vec<(PathBuf, PathBuf)>, Vec<PathBuf>)> {
    // Phase 1: Collect all changes without touching the filesystem.
    let changes = collect_pending_changes(edit, pending_moves)?;
    apply_pending_changes(changes).await
}

/// Write collected changes: the file renames the server asked for (old path,
/// new path) and the paths whose text changed.
pub async fn apply_pending_changes(
    changes: Vec<PendingChange>,
) -> Result<(Vec<(PathBuf, PathBuf)>, Vec<PathBuf>)> {
    // Phase 2: Validate — every text-edit target must exist before we write
    // anything. Fail fast so no partial edits are applied.
    for change in &changes {
        if let PendingChange::TextEdit { path, .. } = change
            && !path.exists()
        {
            anyhow::bail!(
                "LSP returned a text edit for a file that does not exist: {:?}",
                path
            );
        }
    }

    // Phase 3: Execute all changes in their original order.
    let mut file_renames: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut text_edited_paths: Vec<PathBuf> = Vec::new();

    for change in changes {
        match change {
            PendingChange::TextEdit { path, edits } => {
                text_edited_paths.push(path.clone());
                apply_text_edits_to_path(&path, edits).await?;
            }
            PendingChange::ResourceOp(op) => {
                if let Some(rename) = apply_resource_op(op).await? {
                    file_renames.push(rename);
                }
            }
        }
    }

    Ok((file_renames, text_edited_paths))
}

/// One step of a server's workspace edit, in the order the server sent them.
#[derive(Debug)]
pub enum PendingChange {
    TextEdit { path: PathBuf, edits: Vec<TextEdit> },
    ResourceOp(ResourceOp),
}

pub fn collect_pending_changes(
    edit: WorkspaceEdit,
    pending_moves: &HashMap<PathBuf, PathBuf>,
) -> Result<Vec<PendingChange>> {
    let mut changes = Vec::new();

    if let Some(document_changes) = edit.document_changes {
        match document_changes {
            DocumentChanges::Edits(edits) => {
                for edit in edits {
                    collect_text_document_edit(edit, pending_moves, &mut changes)?;
                }
            }
            DocumentChanges::Operations(ops) => {
                for op in ops {
                    match op {
                        DocumentChangeOperation::Edit(edit) => {
                            collect_text_document_edit(edit, pending_moves, &mut changes)?;
                        }
                        DocumentChangeOperation::Op(resource_op) => {
                            changes.push(PendingChange::ResourceOp(resource_op));
                        }
                    }
                }
            }
        }
    } else if let Some(text_changes) = edit.changes {
        for (uri, edits) in text_changes {
            let path = uri_to_path(&uri)?;
            let effective = redirect_if_pending(&path, pending_moves);
            changes.push(PendingChange::TextEdit {
                path: effective,
                edits,
            });
        }
    }

    Ok(changes)
}

fn collect_text_document_edit(
    edit: TextDocumentEdit,
    pending_moves: &HashMap<PathBuf, PathBuf>,
    out: &mut Vec<PendingChange>,
) -> Result<()> {
    let path = uri_to_path(&edit.text_document.uri)?;
    let effective = redirect_if_pending(&path, pending_moves);
    let edits = edit
        .edits
        .into_iter()
        .map(|e| match e {
            OneOf::Left(edit) => edit,
            OneOf::Right(AnnotatedTextEdit { text_edit, .. }) => text_edit,
        })
        .collect();
    out.push(PendingChange::TextEdit {
        path: effective,
        edits,
    });
    Ok(())
}

fn redirect_if_pending(path: &Path, pending_moves: &HashMap<PathBuf, PathBuf>) -> PathBuf {
    if !path.exists()
        && let Some(source) = pending_moves.get(path)
    {
        return source.clone();
    }
    path.to_path_buf()
}

async fn apply_text_edits_to_path(path: &Path, edits: Vec<TextEdit>) -> Result<()> {
    let original = tokio::fs::read_to_string(path).await?;
    let updated = apply_text_edits(&original, edits)?;

    if updated != original {
        tokio::fs::write(path, updated).await?;
    }

    Ok(())
}

pub fn uri_to_path(uri: &Uri) -> Result<PathBuf> {
    let url = Url::parse(&uri.to_string()).map_err(|e| anyhow::anyhow!("Invalid URI: {}", e))?;
    url.to_file_path()
        .map_err(|_| anyhow::anyhow!("Cannot convert URI to file path"))
}
