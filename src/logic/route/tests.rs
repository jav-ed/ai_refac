use super::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn files_go_to_the_language_of_their_extension() {
    for (file, language) in [
        ("a.md", "markdown"),
        ("a.py", "python"),
        ("a.ts", "typescript"),
        ("a.cjs", "typescript"),
        ("a.rs", "rust"),
        ("a.go", "go"),
        ("a.dart", "dart"),
        ("a.kt", "kotlin"),
    ] {
        assert_eq!(language_of(file, None).unwrap(), Some(language), "{file}");
    }
    assert_eq!(language_of("a.java", None).unwrap(), None);
    assert_eq!(language_of("Makefile", None).unwrap(), None);
}

#[test]
fn a_directory_is_routed_by_what_it_contains() {
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join("ts")).unwrap();
    fs::write(dir.path().join("ts/a.ts"), "").unwrap();
    fs::create_dir_all(dir.path().join("kt/com/example")).unwrap();
    fs::write(dir.path().join("kt/com/example/A.kt"), "").unwrap();
    fs::create_dir_all(dir.path().join("empty")).unwrap();

    let root = Some(dir.path());
    assert_eq!(language_of("ts", root).unwrap(), Some("typescript"));
    // The Kotlin file is two levels below the directory that is moved.
    assert_eq!(language_of("kt", root).unwrap(), Some("kotlin"));
    let error = language_of("empty", root).err().unwrap().to_string();
    assert!(error.contains("Kotlin"), "{error}");
}
