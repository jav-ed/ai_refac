use super::*;
use async_trait::async_trait;
use lsp_types::{Position, Range, TextEdit};
use std::path::Path;

fn edit(line: u32, from: u32, to: u32, text: &str) -> TextEdit {
    TextEdit {
        range: Range {
            start: Position::new(line, from),
            end: Position::new(line, to),
        },
        new_text: text.to_string(),
    }
}

fn file_edits(path: &str, edits: Vec<TextEdit>) -> Change {
    Change::Edit(FileEdits {
        path: PathBuf::from(path),
        edits,
    })
}

fn edits_of<'a>(changes: &'a [Change], path: &str) -> &'a [TextEdit] {
    changes
        .iter()
        .find_map(|change| match change {
            Change::Edit(file) if file.path == Path::new(path) => Some(file.edits.as_slice()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no edits for {path}"))
}

#[test]
fn edits_of_another_rename_join_the_file_they_belong_to() {
    let mut changes = vec![file_edits("/p/a.py", vec![edit(0, 4, 8, "surface")])];
    merge(
        &mut changes,
        vec![
            file_edits("/p/a.py", vec![edit(5, 8, 12, "surface")]),
            file_edits("/p/b.py", vec![edit(1, 0, 4, "surface")]),
        ],
    );
    assert_eq!(changes.len(), 2);
    assert_eq!(edits_of(&changes, "/p/a.py").len(), 2);
    assert_eq!(edits_of(&changes, "/p/b.py").len(), 1);
}

#[test]
fn an_edit_both_renames_make_counts_once() {
    let mut changes = vec![file_edits("/p/a.py", vec![edit(0, 4, 8, "surface")])];
    merge(
        &mut changes,
        vec![file_edits(
            "/p/a.py",
            vec![edit(0, 4, 8, "surface"), edit(3, 4, 8, "surface")],
        )],
    );
    assert_eq!(edits_of(&changes, "/p/a.py").len(), 2);
}

#[test]
fn a_file_operation_is_kept_so_the_plan_can_refuse_it() {
    let mut changes = Vec::new();
    merge(
        &mut changes,
        vec![Change::Other {
            kind: "delete".to_string(),
            path: PathBuf::from("/p/a.py"),
        }],
    );
    assert!(matches!(changes[0], Change::Other { .. }));
}

/// A server with a scripted answer to the implementation request, and one
/// rename answer per position that is renamed.
struct Overrides {
    implementations: Value,
    renames: Vec<(u32, Value)>,
    asked: Vec<u32>,
}

#[async_trait]
impl RenameServer for Overrides {
    async fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        match method {
            "textDocument/implementation" => Ok(self.implementations.clone()),
            "textDocument/rename" => {
                let line = params["position"]["line"].as_u64().unwrap() as u32;
                self.asked.push(line);
                Ok(self
                    .renames
                    .iter()
                    .find(|(at, _)| *at == line)
                    .map(|(_, answer)| answer.clone())
                    .unwrap_or(Value::Null))
            }
            other => panic!("unexpected request {other}"),
        }
    }

    async fn sync_document(&mut self, _path: &Path, _text: &str) -> Result<()> {
        Ok(())
    }

    async fn shutdown(self: Box<Self>) {}
}

fn location(path: &Path, line: u32, from: u32, to: u32) -> Value {
    json!({
        "uri": file_uri(path).unwrap(),
        "range": {
            "start": { "line": line, "character": from },
            "end": { "line": line, "character": to },
        },
    })
}

fn rename_answer(path: &Path, line: u32, from: u32, to: u32) -> Value {
    json!({ "changes": { file_uri(path).unwrap(): [{
        "range": {
            "start": { "line": line, "character": from },
            "end": { "line": line, "character": to },
        },
        "newText": "surface",
    }] } })
}

fn reference(path: &Path, line: u32, from: u32, to: u32) -> Reference {
    Reference {
        path: path.to_path_buf(),
        range: Range {
            start: Position::new(line, from),
            end: Position::new(line, to),
        },
    }
}

const SOURCE: &str =
    "class Shape:\n    def area(self): ...\n\n\nclass Rect(Shape):\n    def area(self): ...\n";

#[tokio::test]
async fn the_overrides_a_server_lists_are_renamed_too() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("shapes.py");
    std::fs::write(&path, SOURCE).unwrap();
    let mut server = Overrides {
        // The base method itself, the override, and a subclass named Rect.
        implementations: json!([
            location(&path, 1, 8, 12),
            location(&path, 5, 8, 12),
            location(&path, 4, 6, 10),
        ]),
        renames: vec![(5, rename_answer(&path, 5, 8, 12))],
        asked: Vec::new(),
    };
    let base = vec![file_edits(
        path.to_str().unwrap(),
        vec![edit(1, 8, 12, "surface")],
    )];
    let known = vec![reference(&path, 1, 8, 12)];
    let at = json!({ "textDocument": { "uri": "x" }, "position": { "line": 1, "character": 8 } });

    let changes = with_overrides(&mut server, &at, &known, "area", "surface", base)
        .await
        .unwrap();

    // Only the override was renamed: the symbol itself is known, and the
    // class `Rect` does not spell `area`.
    assert_eq!(server.asked, vec![5]);
    assert_eq!(edits_of(&changes, path.to_str().unwrap()).len(), 2);
}

#[tokio::test]
async fn a_symbol_without_implementations_is_renamed_alone() {
    let mut server = Overrides {
        implementations: Value::Null,
        renames: Vec::new(),
        asked: Vec::new(),
    };
    let base = vec![file_edits("/p/a.py", vec![edit(0, 4, 8, "surface")])];
    let at = json!({});
    let changes = with_overrides(&mut server, &at, &[], "area", "surface", base)
        .await
        .unwrap();
    assert_eq!(changes.len(), 1);
    assert!(server.asked.is_empty());
}

#[tokio::test]
async fn an_override_the_server_will_not_rename_stops_the_whole_rename() {
    struct Refusing(Value);

    #[async_trait]
    impl RenameServer for Refusing {
        async fn request(&mut self, method: &str, _params: Value) -> Result<Value> {
            if method == "textDocument/implementation" {
                return Ok(self.0.clone());
            }
            Err(crate::drivers::lsp_session::RpcError {
                code: -32600,
                message: "cannot rename here".to_string(),
            }
            .into())
        }

        async fn sync_document(&mut self, _path: &Path, _text: &str) -> Result<()> {
            Ok(())
        }

        async fn shutdown(self: Box<Self>) {}
    }

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("shapes.py");
    std::fs::write(&path, SOURCE).unwrap();
    let mut server = Refusing(json!([location(&path, 5, 8, 12)]));
    let error = with_overrides(&mut server, &json!({}), &[], "area", "surface", Vec::new())
        .await
        .err()
        .expect("the refusal is an error")
        .to_string();
    assert!(error.contains("refuses to rename the override"), "{error}");
    assert!(error.contains("shapes.py:6"), "{error}");
    assert!(error.contains("cannot rename here"), "{error}");
}
