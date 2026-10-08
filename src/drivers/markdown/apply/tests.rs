use super::*;
use crate::drivers::markdown::moves::{Move, MoveSet};
use crate::drivers::markdown::plan::{FileWrite, LinkPlan};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn file_move(root: &Path, from: &str, to: &str) -> Move {
    Move {
        from: root.join(from),
        to: root.join(to),
        is_dir: false,
    }
}

fn write(root: &Path, at: &str, before: &str, after: &str) -> FileWrite {
    FileWrite {
        original: root.join(at),
        destination: root.join(at),
        links: 1,
        before: before.to_string(),
        after: after.to_string(),
    }
}

fn plan(when: When, moves: Vec<Move>, writes: Vec<FileWrite>) -> LinkPlan {
    LinkPlan {
        when,
        moves: MoveSet::new(moves).unwrap(),
        writes,
        links_updated: 0,
        checked: 0,
        unreadable: Vec::new(),
    }
}

#[tokio::test]
async fn the_files_are_moved_and_the_changed_text_is_written() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.md"), "old a").unwrap();
    fs::write(dir.path().join("b.md"), "old b").unwrap();

    let plan = plan(
        When::BeforeMoving,
        vec![file_move(dir.path(), "a.md", "sub/a.md")],
        vec![
            write(dir.path(), "sub/a.md", "old a", "new a"),
            write(dir.path(), "b.md", "old b", "new b"),
        ],
    );
    apply(&plan).await.unwrap();

    assert!(!dir.path().join("a.md").exists());
    assert_eq!(
        fs::read_to_string(dir.path().join("sub/a.md")).unwrap(),
        "new a"
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("b.md")).unwrap(),
        "new b"
    );
}

#[tokio::test]
async fn after_other_backends_moved_files_only_the_text_is_written() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("README.md"), "[x](old.py)").unwrap();

    let plan = plan(
        When::AfterMoving,
        vec![file_move(dir.path(), "old.py", "new.py")],
        vec![write(dir.path(), "README.md", "[x](old.py)", "[x](new.py)")],
    );
    apply(&plan).await.unwrap();

    assert_eq!(
        fs::read_to_string(dir.path().join("README.md")).unwrap(),
        "[x](new.py)"
    );
}

#[tokio::test]
async fn a_write_that_fails_puts_back_the_text_and_the_moved_files() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.md"), "old a").unwrap();
    fs::write(dir.path().join("b.md"), "old b").unwrap();

    // The second write goes into a folder that does not exist.
    let plan = plan(
        When::BeforeMoving,
        vec![file_move(dir.path(), "a.md", "sub/a.md")],
        vec![
            write(dir.path(), "b.md", "old b", "new b"),
            write(dir.path(), "missing/c.md", "old c", "new c"),
        ],
    );
    let error = apply(&plan).await.unwrap_err();

    assert!(format!("{error:#}").contains("rolled back"), "{error:#}");
    assert_eq!(
        fs::read_to_string(dir.path().join("a.md")).unwrap(),
        "old a"
    );
    assert!(!dir.path().join("sub/a.md").exists());
    assert_eq!(
        fs::read_to_string(dir.path().join("b.md")).unwrap(),
        "old b"
    );
}

#[tokio::test]
async fn a_target_that_appeared_after_planning_is_refused() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.md"), "a").unwrap();
    fs::write(dir.path().join("taken.md"), "taken").unwrap();

    let plan = plan(
        When::BeforeMoving,
        vec![file_move(dir.path(), "a.md", "taken.md")],
        Vec::new(),
    );
    let error = apply(&plan).await.unwrap_err();

    assert!(format!("{error:#}").contains("already exists"), "{error:#}");
    assert_eq!(
        fs::read_to_string(dir.path().join("taken.md")).unwrap(),
        "taken"
    );
    assert_eq!(fs::read_to_string(dir.path().join("a.md")).unwrap(), "a");
}
