use super::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn files_go_to_the_language_of_their_extension() {
    for (file, language) in [
        ("a.md", "markdown"),
        ("a.MD", "markdown"),
        ("a.markdown", "markdown"),
        ("a.mdx", "markdown"),
        ("logo.png", "markdown"),
        ("manual.PDF", "markdown"),
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
    // Formats that code reads are not assets: a move would leave them behind.
    for file in [
        "config.json",
        "style.css",
        "notes.txt",
        "data.csv",
        "page.html",
    ] {
        assert_eq!(language_of(file, None).unwrap(), None, "{file}");
    }
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

#[test]
fn a_folder_of_documents_and_assets_is_a_markdown_move() {
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join("docs/img")).unwrap();
    fs::write(dir.path().join("docs/guide.md"), "").unwrap();
    fs::write(dir.path().join("docs/img/logo.png"), "").unwrap();
    fs::create_dir_all(dir.path().join("pictures")).unwrap();
    fs::write(dir.path().join("pictures/a.JPG"), "").unwrap();

    let root = Some(dir.path());
    assert_eq!(language_of("docs", root).unwrap(), Some("markdown"));
    // No Markdown at all is fine when the folder holds assets.
    assert_eq!(language_of("pictures", root).unwrap(), Some("markdown"));
}

#[test]
fn a_folder_with_code_or_nothing_useful_is_refused() {
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join("mixed")).unwrap();
    fs::write(dir.path().join("mixed/guide.md"), "").unwrap();
    fs::write(dir.path().join("mixed/run.sh"), "").unwrap();
    fs::create_dir_all(dir.path().join("other")).unwrap();
    fs::write(dir.path().join("other/data.json"), "").unwrap();

    let root = Some(dir.path());
    for folder in ["mixed", "other"] {
        let error = language_of(folder, root).err().unwrap().to_string();
        assert!(
            error.contains("Markdown and asset files"),
            "{folder}: {error}"
        );
    }
}

#[test]
fn a_folder_with_code_of_a_supported_language_goes_to_that_language() {
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join("web")).unwrap();
    fs::write(dir.path().join("web/index.ts"), "").unwrap();
    fs::write(dir.path().join("web/README.md"), "").unwrap();

    assert_eq!(
        language_of("web", Some(dir.path())).unwrap(),
        Some("typescript")
    );
}
