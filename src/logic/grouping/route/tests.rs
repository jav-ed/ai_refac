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
        ("Card.astro", "typescript"),
        ("global.css", "typescript"),
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
        "style.scss",
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

/// A component next to its image is code with an asset, not a document folder:
/// treating it as one moved the folder and left the importer broken.
#[test]
fn a_folder_with_astro_components_is_never_a_document_folder() {
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join("hero")).unwrap();
    fs::write(dir.path().join("hero/Hero.astro"), "").unwrap();
    fs::write(dir.path().join("hero/hero.svg"), "").unwrap();
    // The component sits one level below: no direct TypeScript file, but code.
    fs::create_dir_all(dir.path().join("sections/hero")).unwrap();
    fs::write(dir.path().join("sections/hero/Hero.astro"), "").unwrap();
    fs::write(dir.path().join("sections/hero/hero.svg"), "").unwrap();

    let root = Some(dir.path());
    assert_eq!(language_of("hero", root).unwrap(), Some("typescript"));
    let error = language_of("sections", root).err().unwrap().to_string();
    assert!(error.contains("Markdown and asset files"), "{error}");
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

/// A stylesheet is moved by the TypeScript engine, so a folder of them is too;
/// with pictures next to them it is a document folder, and the engine fixes
/// the stylesheets' paths alongside. A folder that also holds code of another
/// language is still refused.
#[test]
fn a_folder_of_stylesheets_goes_to_typescript() {
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join("styles/pages")).unwrap();
    fs::write(dir.path().join("styles/pages/posts.css"), "").unwrap();
    fs::create_dir_all(dir.path().join("styles/with_pictures")).unwrap();
    fs::write(dir.path().join("styles/with_pictures/posts.css"), "").unwrap();
    fs::write(dir.path().join("styles/with_pictures/hero.svg"), "").unwrap();
    fs::create_dir_all(dir.path().join("mixed")).unwrap();
    fs::write(dir.path().join("mixed/site.css"), "").unwrap();
    fs::write(dir.path().join("mixed/run.sh"), "").unwrap();

    let root = Some(dir.path());
    assert_eq!(
        language_of("styles/pages", root).unwrap(),
        Some("typescript")
    );
    assert_eq!(
        language_of("styles/with_pictures", root).unwrap(),
        Some("markdown")
    );
    assert!(language_of("mixed", root).is_err());
}
