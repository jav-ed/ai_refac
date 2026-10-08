use super::*;

#[test]
fn python_names_the_tools_that_move_files() {
    let text = message("python", None);
    assert!(text.contains("Rope"), "{text}");
    assert!(text.contains("Pyrefly"), "{text}");
}

#[test]
fn an_unknown_language_gets_the_plain_message() {
    assert_eq!(
        message("cobol", None),
        "Driver for 'cobol' is not available."
    );
}
