//! Taking one item out of a grouped import.

use super::*;
use ra_ap_syntax::{Edition, SourceFile};

/// Applies the split of the item named `leaf` and returns the new text.
fn split(content: &str, leaf: &str, new_path: &str) -> Option<String> {
    let parse = SourceFile::parse(content, Edition::CURRENT);
    let path = parse
        .tree()
        .syntax()
        .descendants()
        .filter_map(ast::Path::cast)
        .find(|path| {
            path.syntax().text() == leaf
                && path
                    .syntax()
                    .parent()
                    .is_some_and(|p| ast::UseTree::can_cast(p.kind()))
        })?;
    let mut edits = split_leaf_from_group(Path::new("a.rs"), content, &path, new_path).unwrap()?;
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.start));
    let mut text = content.to_string();
    for edit in edits {
        text.replace_range(edit.start..edit.end, &edit.replacement);
    }
    Some(text)
}

#[test]
fn a_middle_item_leaves_the_group_and_is_imported_on_the_next_line() {
    assert_eq!(
        split(
            "use super::{apply, names, report};\n",
            "names",
            "crate::plan::names"
        )
        .unwrap(),
        "use super::{apply, report};\nuse crate::plan::names;\n"
    );
}

#[test]
fn the_last_item_takes_the_comma_before_it() {
    assert_eq!(
        split(
            "use super::{apply, names};\n",
            "names",
            "crate::plan::names"
        )
        .unwrap(),
        "use super::apply;\nuse crate::plan::names;\n"
    );
}

#[test]
fn a_multi_line_group_keeps_its_layout_and_an_alias_and_visibility_move_along() {
    let content = "pub use super::{\n    apply,\n    names as n,\n    report,\n};\n";
    assert_eq!(
        split(content, "names", "crate::plan::names").unwrap(),
        "pub use super::{\n    apply,\n    report,\n};\npub use crate::plan::names as n;\n"
    );
}

#[test]
fn an_indented_import_keeps_its_indent() {
    let content = "mod tests {\n    use super::{apply, names};\n}\n";
    assert_eq!(
        split(content, "names", "crate::plan::names").unwrap(),
        "mod tests {\n    use super::apply;\n    use crate::plan::names;\n}\n"
    );
}

#[test]
fn a_plain_import_is_not_a_group() {
    assert_eq!(
        split("use super::names;\n", "names", "crate::plan::names"),
        None
    );
}

#[test]
fn an_item_with_a_list_of_its_own_takes_the_list_along() {
    assert_eq!(
        split(
            "use super::{apply, names::{self, Name}, report};\n",
            "names",
            "crate::plan::names"
        )
        .unwrap(),
        "use super::{apply, report};\nuse crate::plan::names::{self, Name};\n"
    );
}

#[test]
fn a_group_of_one_becomes_a_plain_import_of_the_new_path() {
    assert_eq!(
        split("use super::{names};\n", "names", "crate::plan::names").unwrap(),
        "use crate::plan::names;\n"
    );
}
