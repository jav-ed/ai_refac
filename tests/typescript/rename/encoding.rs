//! Columns in wide characters, byte order marks and line endings.

use super::{assert_succeeded, rename};
use crate::common;

use std::fs;

#[test]
fn utf8_columns_and_multibyte_text_are_handled() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    assert_succeeded(&rename(project, "src/unicode.ts", "amount", "sum", &[]));
    let text = common::read_file(project, "src/unicode.ts");
    assert!(
        text.contains("const café = \"😀é\";"),
        "multibyte text intact:\n{text}"
    );
    assert!(
        text.contains("export const sum = café.length;") && text.contains("sum * 2"),
        "{text}"
    );
}

#[test]
fn bom_and_crlf_line_endings_are_preserved() {
    let temp = common::setup_fixture("typescript/rename_project");
    let project = temp.path();
    let original = "\u{FEFF}export const amount = 1;\r\nexport const twice = amount * 2;\r\n";
    fs::write(project.join("src/windows.ts"), original).unwrap();
    assert_succeeded(&rename(
        project,
        "src/windows.ts",
        "amount",
        "quantity",
        &[],
    ));
    let bytes = fs::read(project.join("src/windows.ts")).unwrap();
    let expected = "\u{FEFF}export const quantity = 1;\r\nexport const twice = quantity * 2;\r\n";
    assert_eq!(String::from_utf8(bytes).unwrap(), expected);
}
