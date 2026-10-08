use super::*;
use crate::drivers::lsp::rename::plan::discover::EditedFile;

fn plan_for(path: &std::path::Path, before: &str) -> RenamePlan {
    RenamePlan {
        files: vec![EditedFile {
            file: crate::drivers::lsp::rename::plan::edits::PlannedFile {
                path: path.to_path_buf(),
                bytes: b"new".to_vec(),
                before: before.to_string(),
                text: "new".to_string(),
            },
            edits: Vec::new(),
        }],
        moves: Vec::new(),
    }
}

#[test]
fn a_file_edited_during_planning_blocks_the_write() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("A.kt");
    std::fs::write(&file, "class A\n").unwrap();
    let plan = plan_for(&file, "class A\n");
    assert!(ensure_unchanged(&plan).is_ok());

    std::fs::write(&file, "class A // edited meanwhile\n").unwrap();
    let error = ensure_unchanged(&plan).unwrap_err().to_string();
    assert!(
        error.contains("changed while the rename was being planned"),
        "{error}"
    );
}

#[test]
fn a_byte_order_mark_is_not_a_change() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("A.kt");
    std::fs::write(&file, format!("{BOM}class A\n")).unwrap();
    assert!(ensure_unchanged(&plan_for(&file, "class A\n")).is_ok());
}
