use super::*;
use crate::drivers::lsp_rename::discover::{EditedFile, RenamePlan};
use crate::drivers::lsp_rename::edits::PlannedFile;
use crate::drivers::lsp_rename::test_language::Plain;

fn project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.pl"), "let total = 1\ntotals = 2\n").unwrap();
    std::fs::write(dir.path().join("b.pl"), "print(total)\n# total again\n").unwrap();
    std::fs::write(dir.path().join("notes.txt"), "total\n").unwrap();
    dir
}

fn empty_plan() -> RenamePlan {
    RenamePlan {
        files: Vec::new(),
        moves: Vec::new(),
    }
}

#[test]
fn counts_whole_words_in_source_files_only() {
    let dir = project();
    let notes = scan(dir.path(), &Plain, "total", &empty_plan()).unwrap();
    assert_eq!(notes.len(), 1);
    // a.pl line 1 (not `totals`), b.pl lines 1 and 2; notes.txt is not source.
    assert!(notes[0].contains("3 lines in 2 files"), "{}", notes[0]);
    assert!(notes[0].contains("a.pl:1"), "{}", notes[0]);
    assert!(!notes[0].contains("notes.txt"), "{}", notes[0]);
}

#[test]
fn an_edited_file_is_read_as_it_will_be_after_the_rename() {
    let dir = project();
    let a = dir.path().join("a.pl");
    let b = dir.path().join("b.pl");
    let edited = |path: &std::path::Path, text: &str| EditedFile {
        file: PlannedFile {
            path: path.to_path_buf(),
            bytes: text.as_bytes().to_vec(),
            before: std::fs::read_to_string(path).unwrap(),
            text: text.to_string(),
        },
        edits: Vec::new(),
    };
    let plan = RenamePlan {
        files: vec![
            edited(&a, "let sum = 1\ntotals = 2\n"),
            edited(&b, "print(sum)\n# sum again\n"),
        ],
        moves: Vec::new(),
    };
    assert!(scan(dir.path(), &Plain, "total", &plan).unwrap().is_empty());
}

#[test]
fn generated_and_ignored_folders_are_not_scanned() {
    let dir = project();
    std::fs::create_dir_all(dir.path().join("node_modules/x")).unwrap();
    std::fs::write(dir.path().join("node_modules/x/c.pl"), "total\n").unwrap();
    std::fs::write(dir.path().join(".gitignore"), "ignored/\n").unwrap();
    std::fs::create_dir_all(dir.path().join("ignored")).unwrap();
    std::fs::write(dir.path().join("ignored/d.pl"), "total\n").unwrap();
    let notes = scan(dir.path(), &Plain, "total", &empty_plan()).unwrap();
    assert!(notes[0].contains("3 lines in 2 files"), "{}", notes[0]);
}

#[test]
fn only_the_first_places_are_listed() {
    let dir = tempfile::tempdir().unwrap();
    let lines: String = (0..20).map(|_| "total\n").collect();
    std::fs::write(dir.path().join("a.pl"), lines).unwrap();
    let notes = scan(dir.path(), &Plain, "total", &empty_plan()).unwrap();
    assert!(notes[0].contains("20 lines in 1 file"), "{}", notes[0]);
    assert!(notes[0].contains("a.pl:6"), "{}", notes[0]);
    assert!(!notes[0].contains("a.pl:7"), "{}", notes[0]);
}

#[test]
fn a_leftover_inside_a_place_the_server_never_renames_is_an_attention_note() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("a.pl"),
        "x = 1\n<<\n  use total\n>>\ntotal = 2\n",
    )
    .unwrap();
    let notes = scan(dir.path(), &Plain, "total", &empty_plan()).unwrap();
    assert_eq!(notes.len(), 2, "{notes:?}");
    assert!(notes[0].starts_with("ATTENTION: `total`"), "{}", notes[0]);
    assert!(
        notes[0].contains("inside a template (a.pl:3)"),
        "{}",
        notes[0]
    );
    assert!(notes[0].contains("change it yourself"), "{}", notes[0]);
    // The general note still counts both lines.
    assert!(notes[1].contains("2 lines in 1 file"), "{}", notes[1]);
}
