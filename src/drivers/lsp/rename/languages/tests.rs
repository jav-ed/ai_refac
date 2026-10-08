//! What each language says about names and projects, with no server.

use super::*;
use crate::drivers::lsp::rename::language::Language;
use crate::drivers::lsp::rename::plan::names::validate;

fn error_of<T>(result: anyhow::Result<T>) -> String {
    match result {
        Ok(_) => panic!("expected an error"),
        Err(error) => format!("{error:#}"),
    }
}

#[test]
fn keywords_cannot_name_a_symbol() {
    assert!(validate(&Go, "area", "func").is_err());
    assert!(validate(&Rust, "area", "match").is_err());
    assert!(validate(&Python, "area", "lambda").is_err());
    assert!(validate(&Python, "area", "None").is_err());
    assert!(validate(&Dart, "area", "extends").is_err());
}

#[test]
fn a_word_that_is_a_keyword_elsewhere_is_a_fine_name() {
    // `type` is a Go keyword but only a soft keyword in Python and a plain
    // name in Rust and Dart.
    assert!(validate(&Go, "area", "type").is_err());
    assert!(validate(&Python, "area", "type").is_ok());
    assert!(validate(&Rust, "area", "type").is_err());
    assert!(validate(&Dart, "area", "type").is_ok());
    assert!(validate(&Python, "area", "match").is_ok());
}

#[test]
fn dart_names_may_contain_a_dollar_sign() {
    assert!(Dart.is_identifier_char('$'));
    assert!(!Go.is_identifier_char('$'));
    assert!(!Python.is_identifier_char('$'));
}

#[test]
fn python_special_methods_are_refused_by_either_name() {
    let error = error_of(validate(&Python, "__init__", "setup"));
    assert!(error.contains("special method"), "{error}");
    let error = error_of(validate(&Python, "setup", "__call__"));
    assert!(error.contains("double underscores"), "{error}");
    // Private names and a lone leading pair are ordinary.
    assert!(validate(&Python, "_cache", "__cache").is_ok());
    assert!(validate(&Python, "__", "_").is_ok());
    assert!(validate(&Go, "__init__", "setup").is_ok());
}

#[test]
fn go_needs_a_module_and_rust_a_cargo_manifest() {
    let dir = tempfile::tempdir().unwrap();
    let error = error_of(Go.project_root(dir.path()));
    assert!(error.contains("go.mod"), "{error}");
    std::fs::write(dir.path().join("go.mod"), "module x\n").unwrap();
    assert!(Go.project_root(dir.path()).is_ok());

    let error = error_of(Rust.project_root(dir.path()));
    assert!(error.contains("Cargo.toml"), "{error}");
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\n").unwrap();
    assert!(Rust.project_root(dir.path()).is_ok());
}

#[test]
fn dart_needs_a_pubspec_and_the_resolved_packages() {
    let dir = tempfile::tempdir().unwrap();
    let error = error_of(Dart.project_root(dir.path()));
    assert!(error.contains("pubspec.yaml"), "{error}");

    std::fs::write(dir.path().join("pubspec.yaml"), "name: x\n").unwrap();
    let error = error_of(Dart.project_root(dir.path()));
    assert!(error.contains("dart pub get"), "{error}");

    std::fs::create_dir(dir.path().join(".dart_tool")).unwrap();
    std::fs::write(dir.path().join(".dart_tool/package_config.json"), "{}").unwrap();
    assert!(Dart.project_root(dir.path()).is_ok());
}

#[test]
fn python_takes_any_folder_and_refuses_a_missing_one() {
    let dir = tempfile::tempdir().unwrap();
    assert!(Python.project_root(dir.path()).is_ok());
    let error = error_of(Python.project_root(&dir.path().join("missing")));
    assert!(error.contains("Cannot read the project path"), "{error}");
}

#[test]
fn only_python_asks_for_the_overrides_of_a_method() {
    assert!(Python.renames_overrides());
    assert!(!Go.renames_overrides());
    assert!(!Rust.renames_overrides());
    assert!(!Dart.renames_overrides());
}

#[test]
fn go_asks_again_for_a_partial_answer_and_the_others_do_not() {
    assert_eq!(Go.rename_attempts(), 4);
    assert_eq!(Rust.rename_attempts(), 1);
    assert_eq!(Python.rename_attempts(), 1);
    assert_eq!(Dart.rename_attempts(), 1);
}

#[test]
fn a_name_that_is_not_a_symbol_gets_the_advice_of_its_language() {
    let python = Python.not_a_symbol_hint().expect("Python has advice");
    assert!(python.contains("refac move"), "{python}");
    assert!(Dart.not_a_symbol_hint().is_some());
    assert!(Go.not_a_symbol_hint().is_none());
}
