use super::*;
use std::ffi::OsStr;

fn request(source: &[&str], target: &[&str], root: &Path) -> RefactorRequest {
    RefactorRequest {
        source_path: source.iter().map(|path| path.to_string()).collect(),
        target_path: Some(target.iter().map(|path| path.to_string()).collect()),
        operation: "move".to_string(),
        project_path: Some(root.to_string_lossy().into_owned()),
    }
}

fn rename(file: &str) -> RenameRequest {
    RenameRequest {
        project_path: ".".into(),
        file: file.into(),
        symbol: "old".to_string(),
        new_name: "new".to_string(),
        line: None,
        column: None,
        dry_run: false,
    }
}

#[test]
fn the_variable_is_one_zero_or_empty_and_anything_else_is_an_error() {
    assert!(!parse(None).unwrap());
    assert!(!parse(Some(OsStr::new(""))).unwrap());
    assert!(!parse(Some(OsStr::new("0"))).unwrap());
    assert!(parse(Some(OsStr::new("1"))).unwrap());
    let error = parse(Some(OsStr::new("yes"))).unwrap_err().to_string();
    assert!(
        error.contains(BATCH_ONLY_ENV) && error.contains("yes"),
        "{error}"
    );
}

#[test]
fn a_single_kotlin_file_is_refused_with_the_batch_command_and_the_way_through() {
    let dir = tempfile::tempdir().unwrap();
    let req = request(&["a.kt"], &["pkg/a.kt"], dir.path());

    let message = check_move(&req, false).unwrap_err().to_string();

    assert!(message.contains("single Kotlin move"), "{message}");
    assert!(message.contains(BATCH_ONLY_ENV), "{message}");
    assert!(
        message.contains("--source-path a.kt --source-path <next.kt>"),
        "{message}"
    );
    assert!(message.contains("--target-path pkg/a.kt"), "{message}");
    assert!(message.contains("--allow-single"), "{message}");
    assert!(message.contains("Nothing was changed"), "{message}");
    assert!(check_move(&req, true).is_ok());
}

#[test]
fn several_kotlin_files_a_folder_and_other_languages_are_not_single() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("old")).unwrap();
    std::fs::write(dir.path().join("old/A.kt"), "").unwrap();

    let two = request(&["a.kt", "b.kt"], &["p/a.kt", "p/b.kt"], dir.path());
    assert!(check_move(&two, false).is_ok());

    let folder = request(&["old"], &["new"], dir.path());
    assert!(check_move(&folder, false).is_ok());

    let other = request(&["a.go"], &["p/a.go"], dir.path());
    assert!(check_move(&other, false).is_ok());

    // One Kotlin file next to a Go file is still one Kotlin server start.
    let mixed = request(&["a.kt", "b.go"], &["p/a.kt", "p/b.go"], dir.path());
    assert!(check_move(&mixed, false).is_err());
}

#[test]
fn a_single_kotlin_rename_is_refused_with_its_own_entry_in_the_batch_command() {
    let message = check_rename(&rename("src/A.kt"), false)
        .unwrap_err()
        .to_string();

    assert!(message.contains("single Kotlin rename"), "{message}");
    assert!(
        message.contains(r#"{"file": "src/A.kt", "symbol": "old", "new_name": "new"}"#),
        "{message}"
    );
    assert!(message.contains("refac rename --batch -"), "{message}");
    assert!(message.contains("--allow-single"), "{message}");
    assert!(check_rename(&rename("src/A.kt"), true).is_ok());
    assert!(check_rename(&rename("src/a.go"), false).is_ok());
}

#[test]
fn the_note_says_what_it_cost_and_how_to_batch() {
    let note = note("rename", Duration::from_secs(41), false);

    assert!(note.starts_with("This Kotlin rename took 41 s."), "{note}");
    assert!(note.contains("refac rename --batch renames.json"), "{note}");
    assert!(
        note.contains("--source-path a.kt --source-path b.kt"),
        "{note}"
    );
    assert!(note.contains(BATCH_ONLY_ENV), "{note}");
    assert!(!note.contains("dry run"), "{note}");
    assert!(super::note("move", Duration::from_secs(3), true).contains("This was a dry run"),);
}
