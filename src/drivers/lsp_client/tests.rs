use super::changes::apply_workspace_edit;
use super::*;
use lsp_types::{
    DocumentChanges, OneOf, OptionalVersionedTextDocumentIdentifier, Position, Range,
    TextDocumentEdit, TextEdit, Uri, WorkspaceEdit,
};
use std::str::FromStr;
use tempfile::tempdir;
use url::Url;

#[tokio::test]
async fn test_apply_workspace_edit_writes_changes_map() -> Result<()> {
    let dir = tempdir()?;
    let file_path = dir.path().join("main.rs");
    tokio::fs::write(&file_path, "mod a;\n").await?;

    let uri = Uri::from_str(Url::from_file_path(&file_path).unwrap().as_str()).unwrap();

    let mut changes = HashMap::new();
    changes.insert(
        uri,
        vec![TextEdit {
            range: Range {
                start: Position::new(0, 4),
                end: Position::new(0, 5),
            },
            new_text: "b".to_string(),
        }],
    );

    let _ = apply_workspace_edit(
        WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        },
        &HashMap::new(),
    )
    .await?;

    assert_eq!(tokio::fs::read_to_string(file_path).await?, "mod b;\n");
    Ok(())
}

#[tokio::test]
async fn test_apply_workspace_edit_writes_document_changes() -> Result<()> {
    let dir = tempdir()?;
    let file_path = dir.path().join("main.rs");
    tokio::fs::write(&file_path, "mod a;\n").await?;

    let uri = Uri::from_str(Url::from_file_path(&file_path).unwrap().as_str()).unwrap();

    let edit = TextDocumentEdit {
        text_document: OptionalVersionedTextDocumentIdentifier { uri, version: None },
        edits: vec![OneOf::Left(TextEdit {
            range: Range {
                start: Position::new(0, 4),
                end: Position::new(0, 5),
            },
            new_text: "b".to_string(),
        })],
    };

    let _ = apply_workspace_edit(
        WorkspaceEdit {
            changes: None,
            document_changes: Some(DocumentChanges::Edits(vec![edit])),
            change_annotations: None,
        },
        &HashMap::new(),
    )
    .await?;

    assert_eq!(tokio::fs::read_to_string(file_path).await?, "mod b;\n");
    Ok(())
}
