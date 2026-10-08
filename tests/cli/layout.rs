//! The shape of the code, kept by a test so that a stray or oversized file
//! fails the next run instead of waiting for a review:
//!
//! - `tests/` holds group folders only. Every file directly in it is its own
//!   test program that links the whole library (about 90 MB each, see
//!   `Project_Manag/Docs/Setup/resource_Use.md`).
//! - Every code file is declared by its parent module. A file nobody declares
//!   is never compiled, so its tests would silently never run.
//! - A code file has at most 300 lines of code; blank lines and comments do not
//!   count.
//! - A folder holds at most 9 code files; beyond that, group them by subject.

use std::fs;
use std::path::{Path, PathBuf};

const MAX_CODE_LINES: usize = 300;
const MAX_FILES_PER_FOLDER: usize = 9;

fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn relative(path: &Path) -> String {
    path.strip_prefix(repository())
        .unwrap()
        .display()
        .to_string()
}

fn is_rust(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "rs")
}

/// Every `.rs` file below `dir`. Fixtures are inputs of the tests, not code of
/// this repository, so they are left out.
fn rust_files(dir: &Path, found: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", dir.display()))
        .map(|entry| entry.unwrap().path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == "fixtures") {
                continue;
            }
            rust_files(&path, found);
        } else if is_rust(&path) {
            found.push(path);
        }
    }
}

fn code_files() -> Vec<PathBuf> {
    let mut found = Vec::new();
    for folder in ["src", "tests"] {
        rust_files(&repository().join(folder), &mut found);
    }
    found
}

/// Lines that are neither blank nor a comment.
fn code_lines(text: &str) -> usize {
    let mut in_block_comment = false;
    let mut count = 0;
    for line in text.lines() {
        let line = line.trim();
        if in_block_comment {
            in_block_comment = !line.contains("*/");
        } else if line.is_empty() || line.starts_with("//") {
            continue;
        } else if line.starts_with("/*") {
            in_block_comment = !line.contains("*/");
        } else {
            count += 1;
        }
    }
    count
}

/// Whether `parent` has a line `mod <name>;`, with or without a visibility.
fn declares(parent: &str, name: &str) -> bool {
    parent.lines().any(|line| {
        let mut rest = line.trim_start();
        if let Some(after) = rest.strip_prefix("pub") {
            rest = after;
            if rest.starts_with('(') {
                rest = &rest[rest.find(')').unwrap() + 1..];
            }
            rest = rest.trim_start();
        }
        rest.strip_prefix("mod ")
            .is_some_and(|after| after.trim_end() == format!("{name};"))
    })
}

/// The files that may declare a module living in `dir`: the `mod.rs`, `lib.rs`
/// or `main.rs` inside it, or the `<dir>.rs` next to it.
fn declaring_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = vec![dir.join("mod.rs"), dir.join("lib.rs"), dir.join("main.rs")];
    if let (Some(parent), Some(name)) = (dir.parent(), dir.file_name()) {
        files.push(parent.join(name).with_extension("rs"));
    }
    files
}

fn is_declared(file: &Path) -> bool {
    let dir = file.parent().unwrap();
    let stem = file.file_stem().unwrap().to_str().unwrap();
    // Crate roots, binaries, and the helper module the test groups include
    // with `#[path]` are entry points, not children.
    if matches!(stem, "lib" | "main") || relative(file).starts_with("src/bin/") {
        return true;
    }
    if relative(file) == "tests/common/mod.rs" {
        return true;
    }
    let (name, owners) = if stem == "mod" {
        let name = dir.file_name().unwrap().to_str().unwrap();
        (name, declaring_files(dir.parent().unwrap()))
    } else {
        (stem, declaring_files(dir))
    };
    owners
        .iter()
        .filter(|owner| owner.exists())
        .any(|owner| declares(&fs::read_to_string(owner).unwrap(), name))
}

#[test]
fn tests_holds_group_folders_only() {
    let strays: Vec<String> = fs::read_dir(repository().join("tests"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_file() && is_rust(path))
        .map(|path| relative(&path))
        .collect();
    assert!(
        strays.is_empty(),
        "each file directly in tests/ becomes a test program of about 90 MB; make it a module of the group it belongs to (and `mod name;` in that group's main.rs): {strays:?}"
    );
}

#[test]
fn every_code_file_is_declared_by_its_parent_module() {
    let orphans: Vec<String> = code_files()
        .iter()
        .filter(|file| !is_declared(file))
        .map(|file| relative(file))
        .collect();
    assert!(
        orphans.is_empty(),
        "these files are compiled by nobody, so their code and tests never run; add `mod name;` to the parent: {orphans:?}"
    );
}

#[test]
fn no_code_file_has_more_than_300_lines_of_code() {
    let long: Vec<String> = code_files()
        .iter()
        .filter_map(|file| {
            let lines = code_lines(&fs::read_to_string(file).unwrap());
            (lines > MAX_CODE_LINES).then(|| format!("{} ({lines} lines)", relative(file)))
        })
        .collect();
    assert!(
        long.is_empty(),
        "split these by responsibility (a file for the helpers, a child module per subject): {long:?}"
    );
}

#[test]
fn no_folder_holds_more_than_nine_code_files() {
    let mut crowded = Vec::new();
    let files = code_files();
    let mut folders: Vec<&Path> = files.iter().map(|file| file.parent().unwrap()).collect();
    folders.sort();
    folders.dedup();
    for folder in folders {
        let count = files
            .iter()
            .filter(|file| file.parent() == Some(folder))
            .count();
        if count > MAX_FILES_PER_FOLDER {
            crowded.push(format!("{} ({count} files)", relative(folder)));
        }
    }
    assert!(
        crowded.is_empty(),
        "group the files of these folders into sub-folders by responsibility: {crowded:?}"
    );
}

#[test]
fn the_line_counter_skips_blank_lines_and_comments() {
    let text =
        "//! doc\n\nfn a() {}\n// note\n/* block\n still block */\nfn b() {}\n/* one line */\n";
    assert_eq!(code_lines(text), 2);
}

#[test]
fn a_module_declaration_is_found_with_or_without_visibility() {
    let parent =
        "mod one;\npub mod two;\npub(super) mod three;\n#[cfg(test)]\nmod four;\nmod five { }\n";
    for name in ["one", "two", "three", "four"] {
        assert!(declares(parent, name), "{name}");
    }
    assert!(!declares(parent, "five"));
    assert!(!declares(parent, "six"));
}
