use super::*;
use crate::drivers::lsp_rename::test_language::Plain;

#[test]
fn accepts_identifiers_and_rejects_everything_else() {
    assert!(validate(&Plain, "total", "grandTotal").is_ok());
    assert!(validate(&Plain, "total", "_élan2").is_ok());
    assert!(validate(&Plain, "total", "2fast").is_err());
    assert!(validate(&Plain, "total", "has space").is_err());
    assert!(validate(&Plain, "total", "$total").is_err());
    assert!(validate(&Plain, "total", "`quoted`").is_err());
    assert!(validate(&Plain, "total", "").is_err());
    assert!(validate(&Plain, "total", "class").is_err());
    assert!(validate(&Plain, "total", "total").is_err());
    // Words that are only reserved in other languages are fine.
    assert!(validate(&Plain, "total", "value").is_ok());
}

#[test]
fn the_error_names_the_language_and_the_problem() {
    let error = validate(&Plain, "total", "def").unwrap_err().to_string();
    assert!(error.contains("Plain keyword"), "{error}");
    let error = validate(&Plain, "total", "a b").unwrap_err().to_string();
    assert!(error.contains("plain Plain identifier"), "{error}");
    let error = validate(&Plain, "total", "total").unwrap_err().to_string();
    assert!(error.contains("equals the current name"), "{error}");
}

#[test]
fn the_file_must_exist_lie_in_the_project_and_have_a_known_extension() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(root.join("a.pl"), "x").unwrap();
    std::fs::write(root.join("b.txt"), "x").unwrap();

    let found = resolve_file(&Plain, Path::new("a.pl"), &root).unwrap();
    assert_eq!(found, root.join("a.pl"));

    let error = resolve_file(&Plain, Path::new("missing.pl"), &root)
        .unwrap_err()
        .to_string();
    assert!(error.contains("does not exist"), "{error}");
    let error = resolve_file(&Plain, Path::new("b.txt"), &root)
        .unwrap_err()
        .to_string();
    assert!(error.contains(".pl or .plx"), "{error}");

    let elsewhere = tempfile::tempdir().unwrap();
    std::fs::write(elsewhere.path().join("c.pl"), "x").unwrap();
    let error = resolve_file(&Plain, &elsewhere.path().join("c.pl"), &root)
        .unwrap_err()
        .to_string();
    assert!(error.contains("outside the project"), "{error}");
}
