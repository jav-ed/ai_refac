use super::*;

#[test]
fn accepts_identifiers_and_rejects_everything_else() {
    assert!(validate_names("total", "grandTotal").is_ok());
    assert!(validate_names("total", "_élan2").is_ok());
    assert!(validate_names("total", "2fast").is_err());
    assert!(validate_names("total", "has space").is_err());
    assert!(validate_names("total", "$total").is_err());
    assert!(validate_names("total", "`quoted`").is_err());
    assert!(validate_names("total", "").is_err());
    assert!(validate_names("total", "class").is_err());
    assert!(validate_names("total", "total").is_err());
    // Soft keywords are fine as names.
    assert!(validate_names("total", "value").is_ok());
}

fn plan_for(path: &std::path::Path, before: &str) -> RenamePlan {
    RenamePlan {
        files: vec![plan::EditedFile {
            file: crate::drivers::kotlin::edits::PlannedFile {
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
